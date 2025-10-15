
use std::{sync::Arc, time::Duration};
use teloxide::{prelude::*, types::ParseMode};
use tokio::time::sleep;

use crate::{
    cache::Cache, models::db::users::User, telegram_bot::{
        actions::{CallbackQueryProcessor}, TelegramBot
    }, utils::database_connection::get_db_connection
};

pub struct ShowPk;

#[async_trait::async_trait]
impl CallbackQueryProcessor for ShowPk {
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
        let tg_id = from.id.0 as i64;
        let chat_id = msg.chat().id;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn).await?.ok_or_else(|| anyhow::anyhow!("Wallet not created. Type /start to create wallet"))?;
        let private_key = cfg.aptos_client.export_private_key(&db_user.address).await?;
        let text = format!("🔑 Your Private Key \\(keep it safe\\)\\:\n\n`{}`\n\n⚠️ This message will be deleted automatically in 30 seconds\\.", private_key);
        let sent_message = bot.send_message(chat_id, text).parse_mode(ParseMode::MarkdownV2).await?;
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(30)).await;
            if let Err(e) = bot_clone.delete_message(chat_id, sent_message.id).await {
                tracing::error!("Failed to delete private key temporary message: {:?}", e);
            }
        });
        Ok(())
    }
}
