use std::sync::Arc;

use anyhow::Context;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::{Cache, ICache, Market},
    telegram_bot::{
        TelegramBot, escape_markdown_v2,
        states::{
            StateProcessor,
            long_pair::{build_leverage_keyboard, build_text_for_placing_order},
        },
    },
};

pub struct ShortPair;

#[async_trait::async_trait]
impl StateProcessor for ShortPair {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let similar_markets = cfg.cache.get_markets_ilike(&text).await;
        if similar_markets.len() == 0 {
            bot.send_message(msg.chat.id, "Pair not found, try apt/usd")
                .await?;
            return Ok(());
        }
        let market: &Market = similar_markets
            .first()
            .context("Failed to get first pair")?;
        let context = cfg.cache.get_asset_context(&market.market_name).await;
        let asset = if let Some(asset_context) = context {
            asset_context
        } else {
            bot.send_message(msg.chat.id, "Failed to get pair details")
                .await?;
            return Ok(());
        };
        let keyboard: InlineKeyboardMarkup =
            build_leverage_keyboard(&market.market_name, "short", market.max_leverage);
        bot.send_message(
            msg.chat.id,
            escape_markdown_v2(&build_text_for_placing_order(
                "short",
                market,
                asset.mark_price,
            )),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboard)
        .await?;
        {
            let mut state = cfg.state.lock().await;
            state.remove(&msg.chat.id);
        }
        Ok(())
    }
}
