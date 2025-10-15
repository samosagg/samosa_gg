// use anyhow::Context;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::Cache,
    models::db::users::User,
    telegram_bot::{
        TelegramBot,
        actions::{CallbackQueryProcessor, UserAction},
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
        let chat_id = msg.chat().id;
        let from = callback_query.from;
        let tg_id = from.id.0 as i64;
        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Wallet not created. Type /start to create wallet"))?;

        let text = format!(
            "🌊 Slippage Settings\n\nYour current Slippage: <b>{}%</b>\n\n💡 Lower slippage = safer trades but might fail in volatile markets.\n⚡ Higher slippage = faster execution but higher risk.",
            db_user.slippage
        );
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("✏️ Update", UserAction::UpdateSlippage.to_string()),
            InlineKeyboardButton::callback("🔻 Close", UserAction::Cancel.to_string()),
        ]]);
        bot.send_message(chat_id, text)
            .reply_markup(keyboard)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}
