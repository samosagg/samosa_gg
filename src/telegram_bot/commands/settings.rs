use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::users::User,
    telegram_bot::{actions::UserAction, commands::{mint::build_text_for_wallet_not_created, CommandProcessor}, TelegramBot},
    utils::database_connection::get_db_connection,
};
use anyhow::Context;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use teloxide::types::ParseMode;
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
        let from = msg.from.context("Message missing sender")?;
        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;
        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat.id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let keybaord = build_keyboard_for_setting(db_user.degen_mode, db_user.id, &db_user.token);
        bot.send_message(msg.chat.id, "Settings")
            .reply_markup(keybaord)
            .await?;
        Ok(())
    }
}

pub fn build_keyboard_for_setting(current_degen_mode: bool, user_id: Uuid, token: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        vec![
            vec![
                InlineKeyboardButton::callback("Stats", UserAction::Stats.to_string()),
                InlineKeyboardButton::callback("Accounts", UserAction::Accounts { user_id, token: token.into() }.to_string()),
                InlineKeyboardButton::callback("Slippage", UserAction::Slippage.to_string())
            ],
            vec![
                InlineKeyboardButton::callback("Export private key", UserAction::ExportPk.to_string()),
            ],
            vec![
                InlineKeyboardButton::callback("Withdraw", UserAction::Withdraw { user_id, token: token.into() }.to_string()),
                InlineKeyboardButton::callback("Balances", UserAction::Balances { user_id }.to_string()),
                // InlineKeyboardButton::callback("Transfer", UserAction::Transfer { user_id }.to_string()),
            ],
            vec![
                InlineKeyboardButton::callback(format!("Degen Mode ({})", if current_degen_mode { "ON" } else { "OFF" }), UserAction::ChangeDegenMode { change_to: !current_degen_mode, user_id, token: token.into() }.to_string()),
            ],
        ]
    )
}