use std::sync::Arc;

use crate::{
    cache::Cache,
    models::db::users::User,
    schema::users,
    telegram_bot::{
        TelegramBot,
        commands::{CommandProcessor, PrivateCommand},
    },
    utils::{
        database_connection::get_db_connection, db_execution::execute_with_better_error,
        decibel_transaction::delegate_trading_to,
    },
};
use anyhow::Context;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
pub struct Start;

#[async_trait::async_trait]
impl CommandProcessor for Start {
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
        let message = bot
            .send_message(
                chat_id,
                "👋 Welcome to TradeBot! Getting your account ready...",
            )
            .await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            let wallet_name = format!("aptos-{}", tg_id);
            let (wallet_id, wallet_address, wallet_public_key) = cfg
                .aptos_client
                .create_new_wallet_on_turnkey(&wallet_name)
                .await?;
            let new_user = User::to_db_tg_user(
                tg_id,
                from.username,
                wallet_address.clone(),
                wallet_public_key.clone(),
                wallet_id,
            );
            let create_user_query = diesel::insert_into(users::table)
                .values(new_user.clone())
                .on_conflict_do_nothing();
            execute_with_better_error(&mut conn, vec![create_user_query]).await?;
            // delegate trading to
            let payload = delegate_trading_to(&cfg.config.contract_address, &wallet_address)?;
            let txn = cfg
                .aptos_client
                .sign_txn_with_turnkey_and_fee_payer(&wallet_address, &wallet_public_key, payload)
                .await?;
            let txn_hash = cfg.aptos_client.submit_transaction_and_wait(txn).await?;
            tracing::info!(
                "{} delegated trading: https://explorer.aptoslabs.com/txn/{}?network=decibel",
                &wallet_address,
                txn_hash
            );
            new_user
        };
        let text = format!(
            "👋 Welcome to TradeBot\\!\n\n`{}`\n\n{}",
            db_user.address,
            PrivateCommand::descriptions()
        );
        bot.edit_message_text(chat_id, message.id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}
