pub mod long;
pub mod mint;
pub mod start;
pub mod wallet;
pub mod short;
pub mod settings;
pub mod terminal;

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
pub enum PrivateCommand {
    #[command(aliases = ["help", "h"], description = "Setup your pace.trade account")]
    Start,
    #[command(description = "Mint usdc faucet")]
    Mint,
    #[command(description = "See wallet Info")]
    Wallet,
    #[command(description = "Place a long order")]
    Long,
    #[command(description = "Place a short order")]
    Short,
    #[command(description = "Open your settings")]
    Settings,
    #[command(description = "Open your account on Terminal")]
    Terminal
}
