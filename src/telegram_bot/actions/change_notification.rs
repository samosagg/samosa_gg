// use anyhow::Context;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor},
};

pub struct ChangeNotification;

#[async_trait::async_trait]
impl CallbackQueryProcessor for ChangeNotification {
    async fn process(
        &self,
        _cfg: Arc<TelegramBot<Cache>>,
        _bot: Bot,
        _callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
