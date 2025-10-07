use std::sync::Arc;

use anyhow::Context;
use bigdecimal::BigDecimal;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::{Cache, ICache, Market},
    telegram_bot::{TelegramBot, actions::UserAction, escape_markdown_v2, states::StateProcessor},
};

pub struct LongPair;

#[async_trait::async_trait]
impl StateProcessor for LongPair {
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
            build_leverage_keyboard(&market.market_name, "long", market.max_leverage);
        bot.send_message(
            msg.chat.id,
            escape_markdown_v2(&build_text_for_placing_order(
                "long",
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

pub fn build_leverage_keyboard(
    market: &str,
    order_type: &str,
    max_leverage: u64,
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut row: Vec<InlineKeyboardButton> = vec![];
    for lev in 1..=max_leverage {
        let callback_data = UserAction::Order {
            market: market.into(),
            order_type: order_type.into(),
            leverage: lev,
        }
        .to_string();
        row.push(InlineKeyboardButton::callback(
            format!("{}x", lev),
            callback_data,
        ));
        if row.len() == 5 {
            keyboard.push(row);
            row = vec![];
        }
    }
    // Push remaining buttons if any
    if !row.is_empty() {
        keyboard.push(row);
    }
    InlineKeyboardMarkup::new(keyboard)
}

pub fn build_text_for_placing_order(
    order_type: &str,
    market: &Market,
    price: BigDecimal,
) -> String {
    format!(
        "Placing {} order for {}\n\
        Current Price: ${}\n\n\
        Choose your leverage\n\
        Max leverage {}x
        ",
        order_type, market.market_name, price, market.max_leverage
    )
}
