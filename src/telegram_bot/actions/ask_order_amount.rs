// use std::sync::Arc;

// use teloxide::{Bot, prelude::Requester, types::CallbackQuery};

// use crate::{
//     cache::Cache,
//     telegram_bot::{TelegramBot, actions::CallbackQueryProcessor, states::PendingState},
// };

// pub struct AskOrderAmount {
//     pub market_name: String,
//     pub is_long: bool,
//     pub leverage: u8,
// }

// #[async_trait::async_trait]
// impl CallbackQueryProcessor for AskOrderAmount {
//     async fn process(
//         &self,
//         cfg: Arc<TelegramBot<Cache>>,
//         bot: Bot,
//         callback_query: CallbackQuery,
//     ) -> anyhow::Result<()> {
//         let msg = callback_query
//             .message
//             .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
//         let chat_id = msg.chat().id;
//         let market_name = self.market_name.clone();
//         let text = build_text_for_choose_position_size(&self.market_name, self.is_long);
//         bot.send_message(chat_id, text).await?;

//         {
//             let mut state = cfg.state.lock().await;
//             state.insert(
//                 msg.chat().id,
//                 PendingState::WaitingForOrderMargin {
//                     is_long: self.is_long,
//                     market_name,
//                     leverage: self.leverage,
//                 },
//             );
//         }

//         Ok(())
//     }
// }

// fn build_text_for_choose_position_size(market_name: &str, is_long: bool) -> String {
//     format!(
//         "Choose Position Size\n\nReply with the amount of margin in $ that you would like to {} {} with",
//         if is_long { "long" } else { "short" },
//         market_name
//     )
// }
