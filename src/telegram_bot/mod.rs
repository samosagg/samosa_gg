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
            cancel::Cancel, change_degen_mode::ChangeDegenMode, change_notification::ChangeNotification, confirm_subaccount_deposit::ConfirmSubaccountDeposit, deposit_to_subaccount::DepositToSubaccount, export_pk::ExportPk, external_withdraw::ExternalWithdraw, limit_order_leverage::LimitOrderLeverage, order_leverage::OrderLeverage, place_limit_order::PlaceLimitOrder, place_order::PlaceOrder, show_pk::ShowPk, slippage::Slippage, update_slippage::UpdateSlippage, CallbackQueryProcessor, UserAction
        },
        commands::{
            dashboard::Dashboard, limit::Limit, long::Long, mint::Mint, settings::Settings, short::Short, start::Start, CommandProcessor, PrivateCommand
        },
        states::{
            custom_slippage::CustomSlippage, deposit_to_subaccount::DepositToSubaccount as DepositToSubaccountAmount, external_withdraw_address::ExternalWithdrawAddress, external_withdraw_amount::ExternalWithdrawAmount, limit_order_margin::LimitOrderMargin, limit_pair::LimitPair, limit_price::LimitPrice, order_margin::OrderMargin, order_pair::OrderPair, PendingState, StateProcessor
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
        PrivateCommand::Dashboard => Box::new(Dashboard),
        PrivateCommand::Long => Box::new(Long),
        PrivateCommand::Short => Box::new(Short),
        PrivateCommand::Limit => Box::new(Limit),
        PrivateCommand::Settings => Box::new(Settings),
        // PrivateCommand::Terminal => Box::new(Terminal),
        // PrivateCommand::Chart => Box::new(Chart),
        // PrivateCommand::Positions => Box::new(Positions),
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
                Ok(UserAction::OrderLeverage {
                    market_name,
                    is_long,
                    leverage,
                }) => Some(Box::new(OrderLeverage {
                    market_name,
                    is_long,
                    leverage,
                })),
                Ok(UserAction::PlaceOrder {
                    market_name,
                    is_long,
                    leverage,
                    amount,
                }) => Some(Box::new(PlaceOrder {
                    market_name,
                    is_long,
                    leverage,
                    amount,
                })),
                Ok(UserAction::Cancel) => Some(Box::new(Cancel)),
                Ok(UserAction::LimitOrderLeverage {
                    market_name,
                    price,
                    leverage,
                }) => Some(Box::new(LimitOrderLeverage {
                    market_name,
                    price,
                    leverage,
                })),
                Ok(UserAction::PlaceLimitOrder {
                    market_name,
                    price,
                    leverage,
                    amount,
                    is_long,
                }) => Some(Box::new(PlaceLimitOrder {
                    market_name,
                    price,
                    leverage,
                    amount,
                    is_long,
                })),
                Ok(UserAction::ExportPk) => Some(Box::new(ExportPk)),
                Ok(UserAction::ShowPk) => Some(Box::new(ShowPk)),
                Ok(UserAction::ChangeNotificationPreferences) => Some(Box::new(ChangeNotification)),
                Ok(UserAction::Slippage) => Some(Box::new(Slippage)),
                Ok(UserAction::UpdateSlippage) => Some(Box::new(UpdateSlippage)),
                Ok(UserAction::ChangeDegenMode { user_id, to }) => {
                    Some(Box::new(ChangeDegenMode { user_id, to }))
                }
                Ok(UserAction::DepositToSubaccount) => Some(Box::new(DepositToSubaccount)),
                Ok(UserAction::ConfirmSubaccountDeposit{ amount }) => Some(Box::new(ConfirmSubaccountDeposit{ amount })),
                Ok(UserAction::ExternalWithdraw) => Some(Box::new(ExternalWithdraw)),
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
            PendingState::OrderPair { is_long } => Box::new(OrderPair { is_long }),
            PendingState::OrderMargin {
                market_name,
                is_long,
                leverage,
            } => Box::new(OrderMargin {
                market_name,
                is_long,
                leverage,
            }),
            PendingState::LimitPair => Box::new(LimitPair),
            PendingState::LimitPrice { market_name } => Box::new(LimitPrice { market_name }),
            PendingState::LimitOrderMargin {
                market_name,
                price,
                leverage,
            } => Box::new(LimitOrderMargin {
                market_name,
                price,
                leverage,
            }),
            PendingState::UpdateSlippage => Box::new(CustomSlippage),
            PendingState::DepositToSubaccount { address, balance } => {
                Box::new(DepositToSubaccountAmount { address, balance })
            },
            PendingState::ExternalWithdrawAmount{ balance } => Box::new(ExternalWithdrawAmount{ balance }),
            PendingState::ExternalWithdrawAddress { amount } => Box::new(ExternalWithdrawAddress{ amount })
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
