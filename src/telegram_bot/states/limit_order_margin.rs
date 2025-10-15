use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
    utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

pub struct LimitOrderMargin {
    pub market_name: String,
    pub price: BigDecimal,
    pub leverage: u8,
}

#[async_trait::async_trait]
impl StateProcessor for LimitOrderMargin {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let from = msg.from.context("From is missing in message")?;
        let tg_id = from.id.0 as i64;

        let amount: BigDecimal = match text.parse::<BigDecimal>() {
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

        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Wallet not created yet. Type /start to create wallet")
            })?;
        let request = view_fa_balance_request(
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            &db_user.address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
        let balance_str = serde_json::from_value::<String>(balance_json)?;
        let balance = BigDecimal::from_str(&balance_str)?;
        let divisor = BigDecimal::from(10u64.pow(6));
        let usdc = balance / divisor;

        if usdc < amount {
            bot.send_message(
                chat_id,
                format!("Insufficient balance, Available: {}USDC", usdc),
            )
            .await?;
            return Ok(());
        };
        let text = format!(
            "You are placing <b>{}</b> limit order at price <b>{}</b> with margin <b>{} USDC</b>and Leverage <b>{}x</b>",
            self.market_name.clone(),
            self.price,
            amount,
            self.leverage
        );
        let kb = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback(
                    "🟢 Buy/Long",
                    UserAction::PlaceLimitOrder {
                        market_name: self.market_name.clone(),
                        is_long: true,
                        price: self.price.clone(),
                        leverage: self.leverage,
                        amount: amount.clone(),
                    }
                    .to_string(),
                ),
                InlineKeyboardButton::callback(
                    "🔴 Sell/Short",
                    UserAction::PlaceLimitOrder {
                        market_name: self.market_name.clone(),
                        is_long: false,
                        price: self.price.clone(),
                        leverage: self.leverage,
                        amount: amount.clone(),
                    }
                    .to_string(),
                ),
            ],
            vec![InlineKeyboardButton::callback(
                "❌ Cancel",
                UserAction::Cancel.to_string(),
            )],
        ]);
        bot.send_message(chat_id, text)
            .reply_markup(kb)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}
