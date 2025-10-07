// use anyhow::Context;
use std::sync::Arc;
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor},
};

pub struct Stats;

#[async_trait::async_trait]
impl CallbackQueryProcessor for Stats {
    async fn process(
        &self,
        _cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        let text = build_text_for_stats(0, 0, 0);
        bot.send_message(msg.chat().id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}

fn build_text_for_stats(volume: u64, trades: u64, invites: u64) -> String {
    format!(
        "**Samosagg stats**\n\n\
        Total volume: **${}**\n\n\
        Trades Made: **{}**\n\n\
        Invites: **{}**",
        volume, trades, invites
    )
}
