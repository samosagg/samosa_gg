use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::users::User,
    telegram_bot::{TelegramBot, commands::CommandProcessor},
    utils::{database_connection::get_db_connection, decibel_transaction::mint},
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
        let from = msg.from.context("Message missing sender")?;
        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;
        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat.id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let processing_message = bot
            .send_message(msg.chat.id, build_text_for_processing_request())
            .await?;
        let amount = 10000000u64;

        let payload = mint(&cfg.config.contract_address, &user.wallet_address, amount)?;
        let hash = cfg
            .aptos_client
            .sign_submit_txn_with_turnkey_and_fee_payer(
                &user.wallet_address,
                &user.wallet_public_key,
                payload,
            )
            .await?;
        tracing::info!(
            "Minted usdc faucet: {}, sender({})",
            hash,
            user.wallet_address
        );

        bot.edit_message_text(
            msg.chat.id,
            processing_message.id,
            build_text_for_success_mint_faucet(),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    }
}

pub fn build_text_for_wallet_not_created() -> String {
    "Wallet is not created, to create one type \\/start".to_string()
}

fn build_text_for_success_mint_faucet() -> String {
    "Successfully minted faucet, type /wallet to check your wallet and balances".to_string()
}

fn build_text_for_processing_request() -> String {
    "Processing your request, please wait...".to_string()
}
