use std::sync::Arc;

use crate::{
    cache::Cache, models::db::users::User, telegram_bot::{commands::CommandProcessor, TelegramBot}, utils::{database_connection::get_db_connection, decibel_transaction::mint}
};
use anyhow::Context;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct Mint;

#[async_trait::async_trait]
impl CommandProcessor for Mint {
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
            bot.send_message(chat_id, "Wallet not created yet//. Type /start to create")
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };

        let message = bot
            .send_message(chat_id, "Processing your request...")
            .await?;

        let mint_amount = 100000000u64;
        let payload = mint(&cfg.config.contract_address, &db_user.address, mint_amount)?;
        let signed_txn = cfg
            .aptos_client
            .sign_txn_with_turnkey_and_fee_payer(&db_user.address, &db_user.public_key, payload)
            .await?;

        let hash = cfg
            .aptos_client
            .submit_transaction_and_wait(signed_txn)
            .await?;

        tracing::info!(
            "{} minted faucet: https://explorer.aptoslabs.com/txn/{}?network=decibel",
            db_user.address,
            hash
        );
        

        bot.edit_message_text(
            chat_id,
            message.id,
            "Faucet minted successfully",
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    }
}