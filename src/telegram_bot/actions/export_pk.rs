
use std::sync::Arc;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};

use crate::{
    cache::Cache,
    telegram_bot::{
        actions::{CallbackQueryProcessor, UserAction}, TelegramBot
    },
};

pub struct ExportPk;

#[async_trait::async_trait]
impl CallbackQueryProcessor for ExportPk {
    async fn process(
        &self,
        _cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
        let text = "⚠️ Export Private Key\n\nYour private key gives full control over your funds.\n\nNever share it with anyone — not even admins or bots.\n\nDo you still want to export your key?";
        let markup = InlineKeyboardMarkup::new(
            vec![
                vec![
                    InlineKeyboardButton::callback("🔑 Yes, show my key", UserAction::ShowPk.to_string()),
                    InlineKeyboardButton::callback("❌ Cancel", UserAction::Cancel.to_string())
                ]
            ]
        );
        bot.send_message(msg.chat().id, text).reply_markup(markup).await?;
        Ok(())
    }
}
