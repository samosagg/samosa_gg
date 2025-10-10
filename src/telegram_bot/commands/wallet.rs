// use std::sync::Arc;

// use crate::{
//     cache::Cache,
//     db::{subaccounts::SubAccount, tokens::Token, users::User, wallets::Wallet as DbWallet},
//     telegram_bot::{
//         TelegramBot,
//         actions::UserAction,
//         build_text_for_contact_support,
//         commands::{CommandProcessor, mint::build_text_for_wallet_not_created},
//     },
//     utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
// };
// use anyhow::Context;
// use serde_json::to_string;
// use teloxide::types::ParseMode;
// use teloxide::{
//     prelude::*,
//     types::{InlineKeyboardButton, InlineKeyboardMarkup},
// };

// pub struct Wallet;

// #[async_trait::async_trait]
// impl CommandProcessor for Wallet {
//     async fn process(
//         &self,
//         cfg: Arc<TelegramBot<Cache>>,
//         bot: Bot,
//         msg: Message,
//     ) -> anyhow::Result<()> {
//         let from = msg.from.context("Message missing sender")?;
//         let chat_id = msg.chat.id;

//         let mut conn = get_db_connection(&cfg.pool)
//             .await
//             .context("Failed to get database connection")?;

//         let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
//         let db_user = if let Some(existing_user) = maybe_existing_user {
//             existing_user
//         } else {
//             bot.send_message(msg.chat.id, build_text_for_wallet_not_created())
//                 .parse_mode(ParseMode::MarkdownV2)
//                 .await?;
//             return Ok(());
//         };

//         let maybe_existing_wallet =
//             DbWallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
//         let db_wallet = if let Some(existing_wallet) = maybe_existing_wallet {
//             existing_wallet
//         } else {
//             bot.send_message(msg.chat.id, build_text_for_contact_support())
//                 .parse_mode(ParseMode::MarkdownV2)
//                 .await?;
//             return Ok(());
//         };

//         let subaccounts = SubAccount::get_subaccounts_by_wallet_id(db_wallet.id, &mut conn).await?;

//         let maybe_token = Token::get_token_by_symbol(db_user.token, &mut conn).await?;
//         let db_token = if let Some(token) = maybe_token {
//             token
//         } else {
//             bot.send_message(msg.chat.id, build_text_for_contact_support())
//                 .parse_mode(ParseMode::MarkdownV2)
//                 .await?;
//             return Ok(());
//         };

//         let request = view_fa_balance_request(&db_token.address, &db_wallet.address)?;
//         let response = cfg.aptos_client.view(&request).await?;
//         let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
//         let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;

//         let mut text = build_text_for_wallet_with_balance(
//             &db_wallet.address,
//             balance / 10u64.pow(db_token.decimals as u32),
//         );
//         if !subaccounts.is_empty() {
//             text.push_str("\n\n**SubAccounts**");
//             for (idx, subaccount) in subaccounts.iter().enumerate() {
//                 text.push_str(&format!("\n{} `{}`", idx + 1, subaccount.address));
//             }
//         }
//         bot.send_message(chat_id, text)
//             .reply_markup(InlineKeyboardMarkup::new(vec![vec![
//                 InlineKeyboardButton::callback(
//                     "Deposit to Sub Account",
//                     UserAction::DepositToSubAccount {
//                         subaccount_id: None,
//                     }
//                     .to_string(),
//                 ),
//             ]]))
//             .parse_mode(ParseMode::MarkdownV2)
//             .await?;

//         Ok(())
//     }
// }

// fn build_text_for_wallet_with_balance(address: &str, balance: u64) -> String {
//     format!("Main Account: \n`{}`\nBalance\\: {} USDC", address, balance)
// }
