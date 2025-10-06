// use anyhow::Context;
use std::sync::Arc;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode}};
use uuid::Uuid;

use crate::{
    cache::Cache, db_models::wallets::Wallet, telegram_bot::{actions::CallbackQueryProcessor, TelegramBot}, utils::database_connection::get_db_connection
};

pub struct Accounts {
    pub user_id: Uuid
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for Accounts {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let wallets = Wallet::get_wallets_by_user_id(self.user_id, &mut conn).await?;

        let mut text = String::new();
        
        if !wallets.is_empty() {
            for (idx, wallet) in wallets.iter().enumerate() {
                text.push_str(&build_text_for_wallet(idx, &wallet.address, wallet.is_primary));
                text.push_str("\n\n");
            }
        }

        bot.send_message(msg.chat().id, text)
            .reply_markup(build_keyboard_for_accounts())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}

fn build_text_for_wallet(idx: usize, address: &str, is_primary: bool) -> String {
    format!(
        "{}. #{} {}\n\n
        Address: {}
        Secured by Turnkey",
        idx,
        &address[..5],
        if is_primary { "- Primary" } else { "" },
        address,
    )
}

fn build_keyboard_for_accounts() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        vec![
            vec![
                InlineKeyboardButton::callback("Set primary account", "todo")
            ],
            vec![
                InlineKeyboardButton::callback("New Decibel Account", "todo")
            ],
            vec![
                InlineKeyboardButton::callback("Manage Account", "todo")
            ],
        ]
    )
}