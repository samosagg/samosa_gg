// // use anyhow::Context;
// use std::sync::Arc;
// use teloxide::{
//     prelude::*,
//     types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
// };
// use uuid::Uuid;

// use crate::{
//     cache::Cache,
//     db::{subaccounts::SubAccount, users::User, wallets::Wallet},
//     telegram_bot::{
//         TelegramBot,
//         actions::{CallbackQueryProcessor, UserAction},
//         build_text_for_contact_support,
//         commands::mint::build_text_for_wallet_not_created,
//         states::PendingState,
//     },
//     utils::database_connection::get_db_connection,
// };

// pub struct DepositToSubAccount {
//     pub subaccount_id: Option<Uuid>,
// }

// #[async_trait::async_trait]
// impl CallbackQueryProcessor for DepositToSubAccount {
//     async fn process(
//         &self,
//         cfg: Arc<TelegramBot<Cache>>,
//         bot: Bot,
//         callback_query: CallbackQuery,
//     ) -> anyhow::Result<()> {
//         let msg = callback_query
//             .message
//             .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
//         let from = callback_query.from;
//         let chat_id = msg.chat().id;
//         let telegram_id = from.id.0 as i64;
//         let message_id = msg.id();
//         let mut conn = get_db_connection(&cfg.pool).await?;

//         let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
//         let db_user = if let Some(existing_user) = maybe_existing_user {
//             existing_user
//         } else {
//             bot.send_message(chat_id, build_text_for_wallet_not_created())
//                 .parse_mode(ParseMode::MarkdownV2)
//                 .await?;
//             return Ok(());
//         };

//         let maybe_existing_wallet =
//             Wallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
//         let db_wallet = if let Some(existing_wallet) = maybe_existing_wallet {
//             existing_wallet
//         } else {
//             bot.send_message(chat_id, build_text_for_contact_support())
//                 .parse_mode(ParseMode::MarkdownV2)
//                 .await?;
//             return Ok(());
//         };

//         let subaccount_id = if let Some(sub_id) = self.subaccount_id.clone() {
//             sub_id
//         } else {
//             let text = "Choose a subaccount to deposit to:";
//             let subaccounts =
//                 SubAccount::get_subaccounts_by_wallet_id(db_wallet.id, &mut conn).await?;
//             let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
//             let mut row: Vec<InlineKeyboardButton> = vec![];
//             if !subaccounts.is_empty() {
//                 for (_, subaccount) in subaccounts.iter().enumerate() {
//                     let callback_data = UserAction::DepositToSubAccount {
//                         subaccount_id: Some(subaccount.id.clone()),
//                     }
//                     .to_string();
//                     row.push(InlineKeyboardButton::callback(
//                         format!("{}", subaccount.address),
//                         callback_data,
//                     ));
//                     if row.len() == 2 {
//                         keyboard.push(row);
//                         row = vec![];
//                     }
//                 }
//             };
//             if !row.is_empty() {
//                 keyboard.push(row);
//             };
//             let kb = InlineKeyboardMarkup::new(keyboard);
//             bot.edit_message_text(chat_id, message_id, text)
//                 .reply_markup(kb)
//                 .await?;
//             return Ok(());
//         };
//         // todo deposit
//         {
//             let mut state = cfg.state.lock().await;
//             state.insert(
//                 chat_id,
//                 PendingState::WaitingForSubAccountDepositAmount {
//                     wallet_id: db_wallet.id,
//                     subaccount_id,
//                     token: db_user.token.clone(),
//                 },
//             );
//         }

//         bot.send_message(
//             chat_id,
//             format!(
//                 "Enter the amount in {} to transfer to subaccount:",
//                 db_user.token
//             ),
//         )
//         .await?;
//         Ok(())
//     }
// }
