use std::sync::Arc;

use crate::{
    cache::Cache, models::db::users::User, schema::users, telegram_bot::{commands::{CommandProcessor, PrivateCommand}, TelegramBot}, utils::{database_connection::get_db_connection, db_execution::execute_with_better_error, decibel_transaction::{delegate_trading_to, mint}, view_requests::view_fa_balance_request}
};
use anyhow::Context;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands
};
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
        let message = bot.send_message(chat_id, "👋 Welcome to TradeBot! Getting your account ready...").await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            // user profile setup
            let wallet_name = format!("aptaptwallet-{}", tg_id);
            let (wallet_id, wallet_address, wallet_public_key) =
            cfg.aptos_client.create_new_wallet_on_turnkey(&wallet_name).await?;
            let new_user = User::to_db_tg_user(
                tg_id,
                from.username,
                wallet_address.clone(),
                wallet_public_key.clone(),
                wallet_id
            );
            let create_user_query = diesel
                ::insert_into(users::table)
                .values(new_user.clone())
                .on_conflict_do_nothing();
            execute_with_better_error(&mut conn, vec![create_user_query]).await?;
            // delegate trading to
            let payload = delegate_trading_to(&cfg.config.contract_address, &wallet_address)?;
            let txn = cfg
                .aptos_client
                .sign_txn_with_turnkey_and_fee_payer(
                    &wallet_address, 
                    &wallet_public_key, 
                    payload
                )
                .await?;
            let txn_hash = cfg
                .aptos_client
                .submit_transaction_and_wait(txn)
                .await?;
            tracing::info!("{} delegated trading: https://explorer.aptoslabs.com/txn/{}?network=decibel", &wallet_address, txn_hash);
            new_user
        };
        let request = view_fa_balance_request("0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02", &db_user.address)?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;
        let usdc = balance / 10u64.pow(6);
        let text = format!(
            "👋 Welcome to TradeBot\\!\n\n`{}`\n\n*\\{} USDC*{}\n\n{}",
            db_user.address,
            usdc,
            if usdc < 15 {"\n\n*Note*\\: Balance too low\\. Type /mint to mint some USDC"} else {""},
            PrivateCommand::descriptions()
        );
        bot.edit_message_text(chat_id, message.id, text).parse_mode(ParseMode::MarkdownV2).await?;
        Ok(())
    }
}