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
            add_to_group::AddToGroup, create_trading_account::CreateTradingAccount, join_existing_clan::JoinExistingClan, order_leverage::OrderLeverage, place_order::PlaceOrder, CallbackQueryProcessor, UserAction
        },
        commands::{
            long::Long, mint::Mint, settings::Settings, short::Short, start::Start, wallet::Wallet, CommandProcessor, PrivateCommand
        },
        states::{
            long_pair::LongPair, place_order_quote::PlaceOrderQuote, short_pair::ShortPair, PendingState, StateProcessor
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
        PrivateCommand::Settings => Box::new(Settings)
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
        None => return Ok(()),
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
                order_type,
                market,
                leverage,
            } => Box::new(PlaceOrderQuote {
                market,
                order_type,
                leverage,
            }),
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
