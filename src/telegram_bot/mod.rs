pub mod actions;
pub mod commands;
pub mod states;

use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use futures_util::lock::Mutex;
use teloxide::{prelude::*, types::Me, utils::command::BotCommands};
use tokio::time::sleep;

use crate::{
    cache::{Cache, ICache},
    config::Config,
    telegram_bot::{
        actions::{
            CallbackQueryProcessor, UserAction, accounts::Accounts, add_to_group::AddToGroup,
            ask_order_amount::AskOrderAmount, balances::Balances, change_degen_mode::DegenMode,
            close::Close, create_trading_account::CreateTradingAccount,
            deposit_to_subaccount::DepositToSubAccount, export_pk::ExportPk,
            join_existing_clan::JoinExistingClan, order_leverage::OrderLeverage,
            place_order::PlaceOrder, slippage::Slippage, stats::Stats, transfer::Transfer,
            update_slippage::UpdateSlippage, withdraw::Withdraw,
        },
        commands::{
            CommandProcessor, PrivateCommand, chart::Chart, long::Long, mint::Mint,
            positions::Positions, settings::Settings, short::Short, start::Start,
            terminal::Terminal, wallet::Wallet,
        },
        states::{
            PendingState, StateProcessor, ask_slippage::AskSlippage,
            deposit_to_sub_amount::DepositToSubaccountAmount, long_pair::LongPair,
            order_margin::OrderMargin, order_pair::OrderPair, short_pair::ShortPair,
            withdraw_address::WithdrawAddress, withdraw_amount::WithdrawAmount,
        },
    },
    utils::{aptos_client::AptosClient, database_utils::ArcDbPool},
};

pub struct TelegramBot<TCache: ICache> {
    pool: ArcDbPool,
    config: Arc<Config>,
    aptos_client: Arc<AptosClient>,
    cache: Arc<TCache>,
    pub state: Arc<Mutex<HashMap<ChatId, PendingState>>>,
}

impl<TCache> TelegramBot<TCache>
where
    TCache: ICache + 'static,
{
    pub fn new(
        config: Arc<Config>,
        pool: ArcDbPool,
        aptos_client: Arc<AptosClient>,
        cache: Arc<TCache>,
    ) -> Self {
        Self {
            pool,
            config,
            aptos_client,
            cache,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        tracing::info!("Starting telegram bot...");
        let bot = Bot::new(&self.config.bot_config.token);

        bot.set_my_commands(PrivateCommand::bot_commands())
            .await
            .expect("Failed to set bot commands");

        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<PrivateCommand>()
                    .endpoint(private_commands_handler),
            )
            .branch(Update::filter_callback_query().endpoint(handle_callback_query))
            .branch(Update::filter_message().endpoint(input_handler));

        let arc_telegram_bot = Arc::new(self);
        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![arc_telegram_bot])
            .default_handler(|upd| async move { tracing::warn!("Unhandled update: {upd:?}") })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "An error has occured in the dispatcher",
            ))
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

async fn private_commands_handler(
    cfg: Arc<TelegramBot<Cache>>,
    bot: Bot,
    _me: Me,
    msg: Message,
    cmd: PrivateCommand,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;
    let command_processor: Box<dyn CommandProcessor + Send + Sync> = match cmd {
        PrivateCommand::Start => Box::new(Start),
        PrivateCommand::Mint => Box::new(Mint),
        PrivateCommand::Wallet => Box::new(Wallet),
        PrivateCommand::Long => Box::new(Long),
        PrivateCommand::Short => Box::new(Short),
        PrivateCommand::Settings => Box::new(Settings),
        PrivateCommand::Terminal => Box::new(Terminal),
        PrivateCommand::Chart => Box::new(Chart),
        PrivateCommand::Positions => Box::new(Positions),
    };
    if let Err(err) = command_processor.process(cfg, bot.clone(), msg).await {
        tracing::error!("Command failed: {:?}", err);
        send_temporary_message(&bot, chat_id, format!("{}", err), 15).await?;
    }
    Ok(())
}

// match UserAction::from_str(&data)
//     .or_else(|_| AdminAction::from_str(&data))
//     .or_else(|_| OtherAction::from_str(&data))
// {
//     Ok(UserAction::CreateTradingAccount) => println!("Create Trading Account"),
//     Ok(AdminAction::BanUser) => println!("Ban user"),
//     Ok(OtherAction::DoSomething) => println!("Other action"),
//     Err(_) => println!("Unknown callback"),
// }
async fn handle_callback_query(
    cfg: Arc<TelegramBot<Cache>>,
    bot: Bot,
    query: CallbackQuery,
) -> anyhow::Result<()> {
    if let Some(ref data) = query.data {
        let query_processor: Option<Box<dyn CallbackQueryProcessor + Send + Sync>> =
            match <UserAction as FromStr>::from_str(&data) {
                Ok(UserAction::CreateTradingAccount) => Some(Box::new(CreateTradingAccount)),
                Ok(UserAction::AddToGroup) => Some(Box::new(AddToGroup)),
                Ok(UserAction::JoinExistingClan) => Some(Box::new(JoinExistingClan)),
                Ok(UserAction::Order {
                    market,
                    order_type,
                    leverage,
                }) => {
                    tracing::info!(
                        "Order callback received: market={}, type={}, leverage={}",
                        market,
                        order_type,
                        leverage
                    );
                    Some(Box::new(OrderLeverage {
                        market,
                        order_type,
                        leverage,
                    }))
                }
                Ok(UserAction::ConfirmOrder {
                    market,
                    order_type,
                    leverage,
                    amount,
                }) => {
                    tracing::info!(
                        "Confirm Order callback received: market={}, type={}, leverage={}, amount={}",
                        market,
                        order_type,
                        leverage,
                        amount
                    );
                    Some(Box::new(PlaceOrder {
                        market,
                        order_type,
                        leverage,
                        amount,
                    }))
                }
                Ok(UserAction::ChangeDegenMode {
                    change_to,
                    user_id,
                    token,
                }) => {
                    tracing::info!("Degen mode change callback received: to={}", change_to);
                    Some(Box::new(DegenMode {
                        change_to,
                        user_id,
                        token,
                    }))
                }
                Ok(UserAction::ExportPk) => Some(Box::new(ExportPk)),
                Ok(UserAction::Accounts { user_id, token }) => {
                    tracing::info!(
                        "Accounts callback received: user_id={}, token={}",
                        user_id,
                        token
                    );
                    Some(Box::new(Accounts { user_id, token }))
                }
                Ok(UserAction::Slippage) => Some(Box::new(Slippage)),
                Ok(UserAction::Stats) => Some(Box::new(Stats)),
                Ok(UserAction::Withdraw { user_id, token }) => {
                    tracing::info!(
                        "Withdraw callback received: user_id={}, token={}",
                        user_id,
                        token
                    );
                    Some(Box::new(Withdraw { user_id, token }))
                }
                Ok(UserAction::Transfer { user_id }) => {
                    tracing::info!("Transfer callback received: user_id={}", user_id);
                    Some(Box::new(Transfer { user_id }))
                }
                Ok(UserAction::Balances { user_id }) => {
                    tracing::info!("Balances callback received: user_id={}", user_id);
                    Some(Box::new(Balances { user_id }))
                }
                Ok(UserAction::Close) => Some(Box::new(Close)),
                Ok(UserAction::UpdateSlippage) => Some(Box::new(UpdateSlippage)),
                Ok(UserAction::DepositToSubAccount { subaccount_id }) => {
                    Some(Box::new(DepositToSubAccount { subaccount_id }))
                }
                Ok(UserAction::MarketOrder {
                    is_long,
                    market_name,
                    leverage,
                }) => Some(Box::new(AskOrderAmount {
                    market_name,
                    is_long,
                    leverage,
                })),
                Err(_) => {
                    tracing::warn!("Unknown callback: {}", data);
                    None
                }
            };

        if let Some(processor) = query_processor {
            if let Err(err) = processor.process(cfg, bot.clone(), query.clone()).await {
                tracing::error!("Callback processing failed: {:?}", err);
                let msg = query
                    .message
                    .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
                send_temporary_message(&bot, msg.chat().id, format!("{}", err), 15).await?;
            }
        }
    }
    Ok(())
}

async fn input_handler(cfg: Arc<TelegramBot<Cache>>, bot: Bot, msg: Message) -> anyhow::Result<()> {
    let text = match msg.text() {
        Some(t) => t.to_string(),
        None => {
            return Ok(());
        }
    };

    let chat_id = msg.chat.id;

    let maybe_state = {
        let state = cfg.state.lock().await;
        state.get(&chat_id).cloned()
    };

    if let Some(state) = maybe_state {
        let state_processor: Box<dyn StateProcessor + Send + Sync> = match state {
            PendingState::WaitingForLongPair => Box::new(LongPair),
            PendingState::WaitingForShortPair => Box::new(ShortPair),
            PendingState::WaitingForOrderMargin {
                is_long,
                market_name,
                leverage,
            } => Box::new(OrderMargin {
                market_name,
                is_long,
                leverage,
            }),
            PendingState::WaitingForSlippage => Box::new(AskSlippage),
            PendingState::WaitingForWithdrawAddress { user_id, token } => {
                Box::new(WithdrawAddress { user_id, token })
            }
            PendingState::WaitingForWithdrawAmount {
                user_id,
                token,
                address,
            } => Box::new(WithdrawAmount {
                user_id,
                token,
                address,
            }),
            PendingState::WaitingForSubAccountDepositAmount {
                wallet_id,
                subaccount_id,
                token,
            } => Box::new(DepositToSubaccountAmount {
                wallet_id,
                subaccount_id,
                token,
            }),
            PendingState::WaitingForOrderPair { is_long } => Box::new(OrderPair { is_long }),
        };
        if let Err(err) = state_processor.process(cfg, bot.clone(), msg, text).await {
            tracing::error!("Command failed: {:?}", err);
            send_temporary_message(&bot, chat_id, format!("{}", err), 15).await?;
        }
    }
    Ok(())
}

pub async fn send_temporary_message(
    bot: &Bot,
    chat_id: ChatId,
    text: String,
    duration_secs: u64,
) -> anyhow::Result<()> {
    let sent_message = bot.send_message(chat_id, text).await?;

    let bot_clone = bot.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(duration_secs)).await;
        if let Err(e) = bot_clone.delete_message(chat_id, sent_message.id).await {
            tracing::error!("Failed to delete temporary message: {:?}", e);
        }
    });

    Ok(())
}

pub fn escape_markdown_v2(text: &str) -> String {
    let special = r#"_[]()~`>#+-=|{}.!""#;
    text.chars()
        .map(|c| {
            if special.contains(c) {
                format!(r"\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub fn build_text_for_contact_support() -> String {
    "An unexpected error has occured, Please contact support".to_string()
}
