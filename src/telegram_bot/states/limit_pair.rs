use std::sync::Arc;

use anyhow::Context;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::ParseMode};

use crate::{
    cache::{Cache, ICache},
    telegram_bot::{
        TelegramBot,
        states::{PendingState, StateProcessor},
    },
};

pub struct LimitPair;

#[async_trait::async_trait]
impl StateProcessor for LimitPair {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;

        let similar_markets = cfg.cache.get_markets_ilike(&text).await;
        if similar_markets.len() == 0 {
            bot.send_message(chat_id, "Ticker not found, try again")
                .await?;
            return Ok(());
        }
        let market = similar_markets
            .first()
            .context("Ticker not found on first index")?;

        let asset_context = cfg
            .cache
            .get_asset_context(&market.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market data. Please try again"))?;
        {
            let mut state = cfg.state.lock().await;
            state.insert(
                chat_id,
                PendingState::LimitPrice {
                    market_name: market.market_name.clone(),
                },
            );
        }

        bot.send_message(
            chat_id,
            format!(
                "<b>{}</b> is currently at <b>${}</b>\nEnter your limit price",
                market.market_name, asset_context.mark_price
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;

        Ok(())
    }
}
