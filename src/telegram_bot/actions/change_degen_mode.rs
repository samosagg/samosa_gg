use std::sync::Arc;

use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use teloxide::{
    Bot, payloads::EditMessageReplyMarkupSetters, prelude::Requester, types::CallbackQuery,
};
use uuid::Uuid;

use crate::{
    cache::Cache,
    schema::users,
    telegram_bot::{
        TelegramBot, actions::CallbackQueryProcessor,
        commands::settings::build_keyboard_for_setting,
    },
    utils::{database_connection::get_db_connection, db_execution::execute_with_better_error},
};

pub struct DegenMode {
    pub change_to: bool,
    pub user_id: Uuid,
    pub token: String,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for DegenMode {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        let update_settings_query = diesel::update(users::table.filter(users::id.eq(self.user_id)))
            .set(users::degen_mode.eq(self.change_to));
        let mut conn = get_db_connection(&cfg.pool).await?;

        execute_with_better_error(&mut conn, vec![update_settings_query]).await?;

        let (text1, text2) = if self.change_to {
            ("enabled", " no longer ")
        } else {
            ("disabled", " ")
        };
        let text = format!(
            "Degen mode is now {}. You will{}be asked to confirm trades from now on.",
            text1, text2
        );
        bot.send_message(msg.chat().id, text).await?;

        bot.edit_message_reply_markup(msg.chat().id, msg.id())
            .reply_markup(build_keyboard_for_setting(
                self.change_to,
                self.user_id,
                &self.token,
            ))
            .await?;
        Ok(())
    }
}
