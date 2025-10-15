use std::{str::FromStr, sync::Arc};
use bigdecimal::BigDecimal;
use teloxide::{
    prelude::*,
    types::{ForceReply, ParseMode},
};

use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{
        TelegramBot,
        actions::CallbackQueryProcessor,
        states::PendingState,
    },
    utils::{
        database_connection::get_db_connection,
        view_requests::{view_fa_balance_request, view_primary_subaccount},
    },
};

pub struct DepositToSubaccount;

#[async_trait::async_trait]
impl CallbackQueryProcessor for DepositToSubaccount {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
        let from = callback_query.from;
        let chat_id = msg.chat().id;
        let tg_id = from.id.0 as i64;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Wallet not created yet. Type /start to create wallet")
            })?;
        // request primary
        let request = view_primary_subaccount(&cfg.config.contract_address, &db_user.address)?;
        let response = cfg.aptos_client.view(&request).await?;
        let value = response
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Primary subaccount not found"))?;
        let subaccount = value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected primary subaccount as string"))?;
        // balance
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
            "💰 <b>Deposit to Subaccount</b>\n\n\
            Your main wallet balance: <b>{} USDC</b>\n\
            Primary subaccount: <code>{}</code>\n\n\
            Please enter the amount you want to deposit to your subaccount.\n\
            ⚠️ Make sure you have enough balance in your main wallet.\n\n\
            After entering the amount, click <b>Confirm Deposit</b> to proceed.",
            usdc, subaccount
        );

        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        bot.send_message(chat_id, "Reply with the amount in USDC")
            .reply_markup(ForceReply::new().selective())
            .await?;
        {
            let mut state = cfg.state.lock().await;
            state.insert(
                chat_id,
                PendingState::DepositToSubaccount {
                    address: subaccount.into(),
                    balance: usdc
                },
            );
        }
        Ok(())
    }
}
