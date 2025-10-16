use std::sync::Arc;

use crate::cache::Cache;
use crate::models::db::users::User;
use crate::telegram_bot::states::PendingState;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use crate::utils::database_connection::get_db_connection;
use crate::utils::view_requests::view_fa_balance_request;
use anyhow::Context;
use teloxide::prelude::*;

pub struct Short;

#[async_trait::async_trait]
impl CommandProcessor for Short {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let from = msg.from.context("Missing from in message")?;
        let tg_id = from.id.0 as i64;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Wallet not created yet. Type /start to create"))?;
        let request = view_fa_balance_request(
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            &db_user.address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;
        let usdc = (balance as f64) / 10f64.powi(6);
        let min_required = 10.0;

        if usdc < min_required {
            return Err(anyhow::anyhow!(
                "❌ Minimum {min_required} USDC required to trade.\nYour balance: {:.2} USDC",
                usdc
            ));
        }

        {
            let mut state = cfg.state.lock().await;
            state.insert(
                msg.chat.id,
                PendingState::OrderPair {
                    is_long: false,
                    balance: usdc,
                },
            );
        }

        bot.send_message(chat_id, "📈 Going <b>SHORT</b> — bullish move!\n\nNow, please type the trading pair you want to trade (e.g. `APT/USD`).").parse_mode(teloxide::types::ParseMode::Html).await?;

        Ok(())
    }
}
