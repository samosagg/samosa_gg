use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use bigdecimal::BigDecimal;
use teloxide::{payloads::{EditMessageTextSetters, SendMessageSetters}, prelude::Requester, types::ParseMode};

use crate::{
    cache::Cache, models::db::users::User, telegram_bot::{
        states::StateProcessor, TelegramBot
    }, utils::{database_connection::get_db_connection, decibel_transaction::transfer_fungible_asset}
};

use aptos_sdk::types::account_address::AccountAddress;

pub struct ExternalWithdrawAddress {
    pub amount: BigDecimal
}

#[async_trait::async_trait]
impl StateProcessor for ExternalWithdrawAddress {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let address = AccountAddress::from_hex_literal(&text)?;
        let chat_id = msg.chat.id;
        let from = msg.from.context("From is missing in message")?;
        let tg_id = from.id.0 as i64;
        
        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }
        
        let processing_message = bot.send_message(chat_id, format!("Processing your request to send <b>{} USDC</b> to wallet <code>{}</code>", self.amount.clone(), text)).parse_mode(ParseMode::Html).await?;
        
        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("Wallet not created yet. Type /start to create wallet")
        })?;
        
        let scaled_amount = &self.amount * BigDecimal::from_str("1000000")?;
        let amount_u64 = scaled_amount.with_scale(0).to_string().parse::<u64>()?;
        let payload = transfer_fungible_asset("0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02", &address.to_string(), amount_u64)?;
        let txn = cfg
            .aptos_client
            .sign_txn_with_turnkey_and_fee_payer(&db_user.address, &db_user.public_key, payload)
            .await?;

        let txn_hash = cfg.aptos_client.submit_transaction_and_wait(txn).await?;

        tracing::info!(
            "{} transferred usdc to {}: https://explorer.aptoslabs.com/txn/{}?network=decibel",
            db_user.address,
            text,
            txn_hash.clone()
        );

        bot.edit_message_text(   
            chat_id,    
            processing_message.id,
            format!("✅ Txn completed! <a href='https://explorer.aptoslabs.com/txn/{}?network=decibel'>View Txn</a>", txn_hash),
        )
        .parse_mode(ParseMode::Html)
        .await?;

        Ok(())
    }
}
