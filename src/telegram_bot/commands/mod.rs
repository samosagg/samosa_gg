pub mod chart;
pub mod dashboard;
pub mod limit;
pub mod long;
pub mod mint;
pub mod positions;
pub mod settings;
pub mod short;
pub mod start;
pub mod stoploss;
pub mod takeprofit;
pub mod terminal;
pub mod wallet;

use std::sync::Arc;

use teloxide::utils::command::BotCommands;

use crate::{cache::Cache, telegram_bot::TelegramBot};

#[async_trait::async_trait]
pub trait CommandProcessor {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
    ) -> anyhow::Result<()>;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "The following commands are supported:"
)]
pub enum BotCommand {
    #[command(description = "Setup your account")]
    Start,
    #[command(description = "Mint usdc faucet")]
    Mint,
    #[command(description = "User dashboard")]
    Dashboard,
    #[command(description = "Place a long order")]
    Long,
    #[command(description = "Place a short order")]
    Short,
    #[command(description = "Place a limit order")]
    Limit,
    #[command(description = "Open your settings")]
    Settings,
    #[command(description = "See chart")]
    Chart,
    #[command(description = "Add take profit on a position")]
    Takeprofit,
    #[command(description = "Add stop loss on a position")]
    Stoploss,
}

impl BotCommand {
    pub fn allowed_in_group(&self) -> bool {
        matches!(
            self,
            BotCommand::Chart // | BotCommand::Dashboard
        )
    }
}
