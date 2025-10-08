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

pub struct OrderPair {
    pub is_long: bool,
}

#[async_trait::async_trait]
impl StateProcessor for OrderPair {
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
            let reply_text = format!("{} not found", text);
            bot.send_message(chat_id, reply_text).await?;
            return Ok(());
        }
        let market = similar_markets
            .first()
            .context("Failed to get first market in array of markets")?;

        let context = cfg.cache.get_asset_context(&market.market_name).await;
        let asset_context = if let Some(ctx) = context {
            ctx
        } else {
            let reply_text = format!("Missing asset context for {}", text);
            bot.send_message(chat_id, reply_text).await?;
            return Ok(());
        };

        let keyboard: InlineKeyboardMarkup =
            build_leverage_keyboard(&market.market_name, self.is_long, market.max_leverage);
        let reply_text =
            build_text_for_placing_order(self.is_long, market, asset_context.mark_price);
        bot.send_message(chat_id, escape_markdown_v2(&reply_text))
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
    market_name: &str,
    is_long: bool,
    max_leverage: u8,
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut row: Vec<InlineKeyboardButton> = vec![];
    for leverage in 1..=max_leverage {
        let callback_data = UserAction::MarketOrder {
            is_long,
            market_name: market_name.into(),
            leverage,
        }
        .to_string();
        row.push(InlineKeyboardButton::callback(
            format!("{}x", leverage),
            callback_data,
        ));
        if row.len() == 5 {
            keyboard.push(row);
            row = vec![];
        }
    }
    if !row.is_empty() {
        keyboard.push(row);
    }
    InlineKeyboardMarkup::new(keyboard)
}

pub fn build_text_for_placing_order(is_long: bool, market: &Market, price: BigDecimal) -> String {
    format!(
        "Placing **{}** order for **{}**\n\n\
        Current Price: ${}\n\n\
        Choose your leverage below\n\
        Max leverage {}x",
        if is_long { "long" } else { "short" },
        market.market_name,
        price,
        market.max_leverage
    )
}
