use std::{str::FromStr, sync::Arc};
use bigdecimal::BigDecimal;
use teloxide::{prelude::*, types::ForceReply};

use crate::{
    cache::Cache, models::db::users::User, telegram_bot::{actions::CallbackQueryProcessor, states::PendingState, TelegramBot}, utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request}
};

pub struct ExternalWithdraw;

#[async_trait::async_trait]
impl CallbackQueryProcessor for ExternalWithdraw {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        let chat_id = msg.chat().id;
        let from = callback_query.from;
        let tg_id = from.id.0 as i64;
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

        let text = format!(
            "🌐 <b>Withdraw to External Wallet</b>\n\n\
            Your main wallet balance: <b>{} USDC</b>\n\n\
            Address: <code>{}</code>",
            usdc,
            db_user.address
        );

        bot.send_message(chat_id, text).parse_mode(teloxide::types::ParseMode::Html).await?;
        bot.send_message(
            chat_id,
            "Reply with the amount in USDC you want to withdraw",
        )
        .reply_markup(ForceReply::new().selective())
        .await?;

        {
            let mut state = cfg.state.lock().await;
            state.insert(
                chat_id,
                PendingState::ExternalWithdrawAmount {
                    balance: usdc
                },
            );
        }
        Ok(())
    }
}
