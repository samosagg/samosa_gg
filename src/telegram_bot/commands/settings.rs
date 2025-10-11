use std::sync::Arc;

use crate::{
    cache::Cache, telegram_bot::{
        actions::UserAction, commands::CommandProcessor, TelegramBot
    }, utils::database_connection::get_db_connection
};
use anyhow::Context;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use uuid::Uuid;

pub struct Settings;

#[async_trait::async_trait]
impl CommandProcessor for Settings {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let _ = msg.from.context("From is missing in message")?;
        let chat_id = msg.chat.id;

        let keybaord = InlineKeyboardMarkup::new(
            vec![
                vec![
                    InlineKeyboardButton::callback("Export Private Key", UserAction::ExportPk.to_string())
                ],
                  vec![
                    InlineKeyboardButton::callback("Change Notification Preferences", UserAction::ChangeNotificationPreferences.to_string())
                ],
                vec![
                    InlineKeyboardButton::callback("Slippage", UserAction::Slippage.to_string())
                ]
            ]
        );
        bot.send_message(chat_id, "Settings").reply_markup(keybaord).await?;
        Ok(())
    }
}
