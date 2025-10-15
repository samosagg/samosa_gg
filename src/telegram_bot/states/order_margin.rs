use crate::{
    cache::{Cache, ICache},
    models::db::users::User,
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
    utils::{
        database_connection::get_db_connection,
        view_requests::view_fa_balance_request,
    },
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

pub struct OrderMargin {
    pub market_name: String,
    pub is_long: bool,
    pub leverage: u8,
}

#[async_trait::async_trait]
impl StateProcessor for OrderMargin {
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

        let market = cfg
            .cache
            .get_market(&self.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market. Please try again"))?;
        let asset_context = cfg
            .cache
            .get_asset_context(&market.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market data. Please try again"))?;
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
        let order_type = if self.is_long { "long" } else { "short" };
        let text = format!(
            "You are opening a <b>{}</b> position for <b>{}</b> for <b>{} USDC</b> at price <b>${}</b> with Leverage <b>{}x</b>",
            order_type.to_uppercase(),
            market.market_name,
            amount,
            asset_context.mark_price,
            self.leverage
        );
        let kb = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback(
                "✅ Yes",
                UserAction::PlaceOrder {
                    market_name: self.market_name.clone(),
                    is_long: self.is_long,
                    leverage: self.leverage,
                    amount,
                }
                .to_string(),
            ),
            InlineKeyboardButton::callback("❌ Cancel", UserAction::Cancel.to_string()),
        ]]);
        bot.send_message(chat_id, text)
            .reply_markup(kb)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}
