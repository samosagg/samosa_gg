use std::sync::Arc;

use crate::cache::Cache;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use anyhow::Context;
use teloxide::prelude::*;

pub struct Stoploss;

#[async_trait::async_trait]
impl CommandProcessor for Stoploss {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let from = msg.from.context("Missing from in message")?;
        let tg_id = from.id.0 as i64;
        Ok(())
    }
}
