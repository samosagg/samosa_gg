use std::sync::Arc;

use crate::cache::Cache;
use crate::telegram_bot::states::PendingState;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use teloxide::prelude::*;

pub struct Short;

#[async_trait::async_trait]
impl CommandProcessor for Short {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;

        {
            let mut state = cfg.state.lock().await;
            state.insert(msg.chat.id, PendingState::OrderPair { is_long: false });
        }

        bot.send_message(chat_id, "Write ticker e.g. APT/USD")
            .await?;

        Ok(())
    }
}
