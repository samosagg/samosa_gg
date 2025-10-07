use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::{users::User, wallets::Wallet},
    telegram_bot::{TelegramBot, actions::UserAction, commands::CommandProcessor},
    utils::database_connection::get_db_connection,
};
use anyhow::Context;
use teloxide::types::ParseMode;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
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
        let from = msg.from.context("Message missing sender")?;

        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;

        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat.id, build_text_for_new_user())
                .reply_markup(build_keyboard_for_new_user())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let maybe_existing_wallet =
            Wallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
        let existing_wallet = if let Some(wallet) = maybe_existing_wallet {
            wallet
        } else {
            bot.send_message(msg.chat.id, build_text_for_new_user())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let kb = build_keyboard_for_existing_user();
        let text = build_text_for_existing_user(&existing_wallet.address);

        bot.send_message(msg.chat.id, text)
            .reply_markup(kb)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}

pub fn build_text_for_existing_user(address: &str) -> String {
    format!(
        "Welcome back to SAMOSAGG — your gateway to the Aptos universe\\!\n\n\
            Your wallet address: `{}`\n\n",
        address
    )
}

pub fn build_keyboard_for_existing_user() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Add to Group", UserAction::AddToGroup.to_string()),
        InlineKeyboardButton::callback(
            "Join existing clan",
            UserAction::JoinExistingClan.to_string(),
        ),
    ]])
}

fn build_text_for_new_user() -> String {
    "Welcome to SAMOSAGG\\! Let's set up your trading account".to_string()
}

fn build_keyboard_for_new_user() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Create Trading Account",
        UserAction::CreateTradingAccount.to_string(),
    )]])
}
