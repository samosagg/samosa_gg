use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::users::User,
    telegram_bot::{TelegramBot, commands::CommandProcessor},
    utils::{database_connection::get_db_connection, decibel_transaction::mint},
};
use anyhow::Context;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use teloxide::types::ParseMode;

pub struct Settings;

#[async_trait::async_trait]
impl CommandProcessor for Settings {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        

        Ok(())
    }
}

fn build_keyboard_for_setting() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        vec![
            vec![
                InlineKeyboardButton::callback("Stats", "stats"),
                InlineKeyboardButton::callback("Sub Accounts", "subaccounts"),
                // InlineKeyboardButton::callback("Notifications", "notifications"),
            ],
            vec![
                InlineKeyboardButton::callback("Export private key", "export"),
            ],
            vec![
                InlineKeyboardButton::callback("Withdraw", "with"),
                InlineKeyboardButton::callback("Balances", "balance"),
                InlineKeyboardButton::callback("Transfer", "transfer"),
            ],
            vec![
                InlineKeyboardButton::callback("Degen Mode", "degen_mode"),
            ],
        ]
    )
}