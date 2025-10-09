use std::sync::Arc;

use crate::cache::Cache;
use crate::telegram_bot::states::PendingState;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct Short;

#[async_trait::async_trait]
impl CommandProcessor for Short {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        // let from = msg.from.context("Message missing sender")?;
        // {
        //     let mut state = cfg.state.lock().await;
        //     state.insert(msg.chat.id, PendingState::WaitingForShortPair);
        // }

        // bot.send_message(msg.chat.id, build_text_for_asking_pair())
        //     .parse_mode(ParseMode::MarkdownV2)
        //     .await?;

        Ok(())
    }
}

fn build_text_for_asking_pair() -> String {
    "Enter the name of the token you want to place long order to, e\\.g\\. apt or apt/usd"
        .to_string()
}
