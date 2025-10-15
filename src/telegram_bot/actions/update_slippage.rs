use std::sync::Arc;
use teloxide::{prelude::*, types::ForceReply};

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor, states::PendingState},
};

pub struct UpdateSlippage;
#[async_trait::async_trait]
impl CallbackQueryProcessor for UpdateSlippage {
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
        bot.send_message(chat_id, "Reply with the slippage in % you want to set:")
            .reply_markup(ForceReply::new().selective())
            .await?;
        {
            let mut state = cfg.state.lock().await;
            state.insert(chat_id, PendingState::UpdateSlippage);
        }
        Ok(())
    }
}
