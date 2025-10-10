// // use anyhow::Context;
// use std::sync::Arc;
// use teloxide::{prelude::*, types::ForceReply};
// use uuid::Uuid;

// use crate::{
//     cache::Cache,
//     telegram_bot::{TelegramBot, actions::CallbackQueryProcessor, states::PendingState},
// };

// pub struct Withdraw {
//     pub user_id: Uuid,
//     pub token: String,
// }

// #[async_trait::async_trait]
// impl CallbackQueryProcessor for Withdraw {
//     async fn process(
//         &self,
//         cfg: Arc<TelegramBot<Cache>>,
//         bot: Bot,
//         callback_query: CallbackQuery,
//     ) -> anyhow::Result<()> {
//         let msg = callback_query
//             .message
//             .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

//         {
//             let mut state = cfg.state.lock().await;
//             state.insert(
//                 msg.chat().id,
//                 PendingState::WaitingForWithdrawAddress {
//                     user_id: self.user_id,
//                     token: self.token.clone(),
//                 },
//             );
//         }

//         bot.send_message(
//             msg.chat().id,
//             format!(
//                 "Reply with the address you want to withdraw {} to",
//                 self.token
//             ),
//         )
//         .reply_markup(ForceReply::new().selective())
//         .await?;

//         Ok(())
//     }
// }
