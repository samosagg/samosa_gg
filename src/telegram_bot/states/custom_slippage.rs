use std::sync::Arc;

use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use teloxide::prelude::Requester;

use crate::{
    cache::Cache,
    schema::users,
    telegram_bot::{TelegramBot, states::StateProcessor},
    utils::{database_connection::get_db_connection, db_execution::execute_with_better_error},
};

pub struct CustomSlippage;

#[async_trait::async_trait]
impl StateProcessor for CustomSlippage {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let new_slippage = match text.parse::<i64>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(msg.chat.id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };
        if new_slippage <= 0 || new_slippage >= 100 {
            bot.send_message(msg.chat.id, "Slippage must be between 0 to 100")
                .await?;
            return Ok(());
        };
        let chat_id = msg.chat.id;

        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }

        let from = msg.from.ok_or_else(|| anyhow::anyhow!("From is missing"))?;
        let tg_id = from.id.0 as i64;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let query = diesel::update(users::table.filter(users::tg_id.eq(Some(tg_id))))
            .set(users::slippage.eq(new_slippage));

        execute_with_better_error(&mut conn, vec![query]).await?;

        bot.send_message(
            chat_id,
            format!("Successfully updated slippage to {}%", new_slippage),
        ).await?;
       
        Ok(())
    }
}
