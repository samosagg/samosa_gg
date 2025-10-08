use crate::{
    cache::Cache,
    db_models::{users::NewTelegramUser, wallets::NewWallet},
    schema::{users, wallets},
    telegram_bot::{
        TelegramBot,
        actions::CallbackQueryProcessor,
        commands::start::{build_keyboard_for_existing_user, build_text_for_existing_user},
    },
    utils::{
        database_connection::get_db_connection, db_execution::execute_with_better_error,
        decibel_transaction::delegate_trading_to,
    },
};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct CreateTradingAccount;

#[async_trait::async_trait]
impl CallbackQueryProcessor for CreateTradingAccount {
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
        let telegram_id = from.id.0 as i64;

        let wallet_name = format!("apt-wallet-{}", telegram_id);
        let (wallet_id, wallet_address, wallet_public_key) = cfg
            .aptos_client
            .create_new_wallet_on_turnkey(&wallet_name)
            .await?;
        let new_user = NewTelegramUser::to_db_user_with_custom_uuid(
            uuid::Uuid::new_v4(),
            telegram_id,
            from.username,
        );

        let new_wallet = NewWallet::to_db_wallet(
            new_user.id,
            wallet_id,
            wallet_address.clone(),
            wallet_public_key.clone(),
            true
        );

        let create_user_query = diesel::insert_into(users::table)
            .values(new_user)
            .on_conflict_do_nothing();
        let create_wallet_query = diesel::insert_into(wallets::table)
            .values(new_wallet)
            .on_conflict_do_nothing();

        let mut conn = get_db_connection(&cfg.pool).await?;

        let payload = delegate_trading_to(&cfg.config.contract_address, &wallet_address)?;
        let signed_txn = cfg.aptos_client.sign_txn_with_turnkey_and_fee_payer(
            &wallet_address, 
            &wallet_public_key, 
            payload
        ).await?;

        // let vm_error = cfg.aptos_client.simulate_transaction(&signed_txn).await?;
        // if let Some(err) = vm_error {
        //     bot.send_message(chat_id, err).await?;
        //     return Ok(())
        // } else {
        //     println!("Simulation success");
        // };

        let hash = cfg
            .aptos_client
            .submit_transaction_and_wait(
                signed_txn
            )
            .await?;

        tracing::info!(
            "Delegate trading to hash: {}, sender({})",
            hash,
            wallet_address
        );

         bot.edit_message_text(
            chat_id,
            msg.id(),
            build_text_for_existing_user(&wallet_address),
        )
        .reply_markup(build_keyboard_for_existing_user())
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        execute_with_better_error(&mut conn, vec![create_user_query]).await?;
        execute_with_better_error(&mut conn, vec![create_wallet_query]).await?;
        Ok(())
    }
}
