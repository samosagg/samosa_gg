// use anyhow::Context;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    cache::Cache,
    db_models::users::User,
    telegram_bot::{
        TelegramBot,
        actions::{CallbackQueryProcessor, UserAction},
        commands::mint::build_text_for_wallet_not_created,
    },
    utils::database_connection::get_db_connection,
};

pub struct Slippage;

#[async_trait::async_trait]
impl CallbackQueryProcessor for Slippage {
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

        let mut conn = get_db_connection(&cfg.pool).await?;

        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;

        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat().id, build_text_for_wallet_not_created())
                .await?;
            return Ok(());
        };

        let text = build_text_for_slippage(db_user.slippage);
        let kb = build_keyboard_for_slippage_update();
        bot.send_message(msg.chat().id, text)
            .reply_markup(kb)
            .await?;
        Ok(())
    }
}

fn build_text_for_slippage(current_slippage: i32) -> String {
    format!("Slippage settings\n\nMax Slippage: {}%", current_slippage)
}

fn build_keyboard_for_slippage_update() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Cancel", UserAction::Close.to_string()),
        InlineKeyboardButton::callback("Update Slippage", UserAction::UpdateSlippage.to_string()),
    ]])
}
