use std::sync::Arc;

use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{TelegramBot, commands::CommandProcessor},
    utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
};
use anyhow::Context;
use teloxide::{prelude::*, types::ParseMode};
pub struct Dashboard;

#[async_trait::async_trait]
impl CommandProcessor for Dashboard {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let from = msg.from.context("From is missing in message")?;
        let chat_id = msg.chat.id;
        let tg_id = from.id.0 as i64;
        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;

        let maybe_existing_user = User::get_by_telegram_id(tg_id, &mut conn).await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(
                chat_id,
                "Wallet not created yet. Type /start to create wallet",
            )
            .await?;
            return Ok(());
        };
        let message = bot
            .send_message(chat_id, "Preparing your dashboard")
            .await?;
        let request = view_fa_balance_request(
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            &db_user.address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;
        let usdc = (balance as f64) / 10f64.powi(6);

        let text = format!(
            "📊 <b>TradeBot Dashboard</b>\n\n<code>{}</code>\n\n💵 Available Balance: <b>\\{} USDC</b>\n\n📂 Active Positions\\: {}",
            db_user.address, usdc, "No active positions"
        );
        bot.edit_message_text(chat_id, message.id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}
