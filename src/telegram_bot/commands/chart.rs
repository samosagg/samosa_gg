use std::sync::Arc;

use crate::cache::Cache;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use teloxide::prelude::*;

pub struct Chart;

#[async_trait::async_trait]
impl CommandProcessor for Chart {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let args = msg.text();
        // get pair and interval from args

        Ok(())
    }
}
