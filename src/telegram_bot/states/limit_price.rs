use std::sync::Arc;

use bigdecimal::BigDecimal;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::{Cache, ICache},
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
};

pub struct LimitPrice {
    pub market_name: String,
}

#[async_trait::async_trait]
impl StateProcessor for LimitPrice {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let price: BigDecimal = match text.parse::<BigDecimal>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(chat_id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };
        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }
        let market = cfg
            .cache
            .get_market(&self.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market. Please try again"))?;
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
        let mut row: Vec<InlineKeyboardButton> = vec![];
        for leverage in 1..=market.max_leverage {
            let callback_data = UserAction::LimitOrderLeverage {
                market_name: market.market_name.clone(),
                price: price.clone(),
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
        let kb = InlineKeyboardMarkup::new(keyboard);
        bot.send_message(chat_id, "*Choose leverage*")
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(kb)
            .await?;
        Ok(())
    }
}
