// use anyhow::Context;
use std::sync::Arc;
use anyhow::Context;
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    cache::Cache, db_models::{users::User, wallets::Wallet}, telegram_bot::{actions::CallbackQueryProcessor, build_text_for_contact_support, commands::mint::build_text_for_wallet_not_created, escape_markdown_v2, TelegramBot}, utils::database_connection::get_db_connection
};

pub struct ExportPk;

#[async_trait::async_trait]
impl CallbackQueryProcessor for ExportPk {
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
        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;
        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat().id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let primary_wallet_opt = Wallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
        let db_wallet = if let Some(existing_wallet) = primary_wallet_opt {
            existing_wallet
        } else {
            bot.send_message(msg.chat().id, build_text_for_contact_support())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let private_key_str = cfg.aptos_client.export_private_key(&db_wallet.address).await?;
        let text = build_text_for_export_pk(&private_key_str);
        bot.send_message(msg.chat().id, escape_markdown_v2(&text)).parse_mode(ParseMode::MarkdownV2).await?;
        Ok(())
    }
}

fn build_text_for_export_pk(private_key: &str) -> String {
    format!(
        "WARNING: Never share your private key!\n\n\
        If anyone, including samosa.gg team or mods, asks for your private key, IT IS A SCAM! Sending it to them will give them full control over your wallet.\n\n\
        samosa.gg team and mods will NEVER ask for your private key.\n\n\
        ||`{}`||",
        private_key
    )
}
