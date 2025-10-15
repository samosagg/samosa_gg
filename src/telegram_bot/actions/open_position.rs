// use anyhow::Context;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor},
};

pub struct OpenPosition {
    pub market_name: String,
    pub is_long: bool,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for OpenPosition {
    async fn process(
        &self,
        _cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        bot.delete_message(msg.chat().id, msg.id()).await?;
        Ok(())
    }
}
