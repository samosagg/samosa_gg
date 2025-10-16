use bigdecimal::BigDecimal;
use std::{str::FromStr, sync::Arc};
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor},
    utils::{
        database_connection::get_db_connection, decibel_transaction::deposit_to_subaccount_at,
        view_requests::view_primary_subaccount,
    },
};

pub struct ConfirmSubaccountDeposit {
    pub amount: BigDecimal,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for ConfirmSubaccountDeposit {
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
        let scaled_amount = &self.amount * BigDecimal::from_str("1000000")?;
        let amount_u64 = scaled_amount.with_scale(0).to_string().parse::<u64>()?;
        let payload = deposit_to_subaccount_at(
            &cfg.config.contract_address,
            subaccount,
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            amount_u64,
        )?;
        let txn = cfg
            .aptos_client
            .sign_txn_with_turnkey_and_fee_payer(&db_user.address, &db_user.public_key, payload)
            .await?;

        let txn_hash = cfg.aptos_client.submit_transaction_and_wait(txn).await?;

        tracing::info!(
            "{} deposited to subaccount {}: https://explorer.aptoslabs.com/txn/{}?network=decibel",
            db_user.address,
            subaccount,
            txn_hash.clone()
        );

        bot.send_message(
            chat_id,
            format!("✅ Txn completed! <a href='https://explorer.aptoslabs.com/txn/{}?network=decibel'>View Txn</a>", txn_hash),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        Ok(())
    }
}
