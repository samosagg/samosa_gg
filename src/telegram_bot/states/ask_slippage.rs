use std::sync::Arc;

use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use teloxide::prelude::Requester;

use crate::{
    cache::Cache,
    schema::users,
    telegram_bot::{TelegramBot, states::StateProcessor},
    utils::{database_connection::get_db_connection, db_execution::execute_with_better_error},
};

pub struct AskSlippage;

#[async_trait::async_trait]
impl StateProcessor for AskSlippage {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let new_slippage = match text.parse::<u32>() {
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

        let telegram_id = match msg.from {
            Some(user) => user.id.0 as i64,
            None => {
                return Ok(()); // Message without sender, ignore
            }
        };
        let mut conn = get_db_connection(&cfg.pool).await?;
        let query = diesel::update(users::table.filter(users::telegram_id.eq(Some(telegram_id))))
            .set(users::slippage.eq(new_slippage as i32));

        execute_with_better_error(&mut conn, vec![query]).await?;

        bot.send_message(
            msg.chat.id,
            format!("Successfully updated slippage to {}%", new_slippage),
        )
        .await?;
        {
            let mut state = cfg.state.lock().await;
            state.remove(&msg.chat.id);
        }
        Ok(())
    }
}
