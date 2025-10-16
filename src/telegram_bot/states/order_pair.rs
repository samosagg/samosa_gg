use std::sync::Arc;

use anyhow::Context;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::{Cache, ICache},
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
};

pub struct OrderPair {
    pub is_long: bool,
    pub balance: f64,
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
            return Err(anyhow::anyhow!("Ticker not found, try again"));
        }
        let market = similar_markets
            .first()
            .context("Ticker not found on first index")?;

        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
        let mut row: Vec<InlineKeyboardButton> = vec![];
        for leverage in 1..=market.max_leverage {
            let callback_data = UserAction::OrderLeverage {
                market_name: market.market_name.clone(),
                is_long: self.is_long,
                leverage,
                balance: self.balance,
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
        let kb = InlineKeyboardMarkup::new(keyboard);
        bot.send_message(
            chat_id,
            "⚙️ <b>Choose your leverage</b>\n\nSelect how much risk you want to take:\n\
            • 1x — Safe & steady 🛡️\n\
            • 5x — Moderate risk ⚖️\n\
            • 10x — High risk ⚡\n\
            • 20x+ — Degens only 💀",
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;

        Ok(())
    }
}
