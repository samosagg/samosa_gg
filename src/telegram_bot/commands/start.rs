use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::users::User,
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
        let (text, keyboard) = if let Some(existing_user) = maybe_existing_user {
            (
                build_text_for_existing_user(&existing_user.wallet_address),
                Some(build_keyboard_for_existing_user()), // replace with actual keyboard
            )
        } else {
            (
                build_text_for_new_user(),
                Some(build_keyboard_for_new_user()), // replace with actual keyboard
            )
        };
        if let Some(kb) = keyboard {
            bot.send_message(msg.chat.id, text)
                .reply_markup(kb)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        } else {
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        };
        Ok(())
    }
}

pub fn build_text_for_existing_user(address: &str) -> String {
    format!(
        "Welcome back to PACE TRADE — your gateway to the Aptos universe\\!\n\n\
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
    "Welcome to PACE TRADE\\! Let's set up your trading account".to_string()
}

fn build_keyboard_for_new_user() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Create Trading Account",
        UserAction::CreateTradingAccount.to_string(),
    )]])
}
