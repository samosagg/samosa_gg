use std::sync::Arc;

use crate::{
    cache::Cache,
    telegram_bot::{commands::CommandProcessor, TelegramBot},
};
use anyhow::Context;
use teloxide::prelude::*;

pub struct Terminal;

#[async_trait::async_trait]
impl CommandProcessor for Terminal {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let from = msg.from.context("Message missing sender")?;
        
        Ok(())
    }
}