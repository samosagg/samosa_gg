// use chrono::{Duration, Utc};
// use jsonwebtoken::{EncodingKey, Header};
// use std::sync::Arc;
// use url::Url;

// use crate::{
//     cache::Cache,
//     db::users::User,
//     http_server::middlewares::authentication::TelegramClaims,
//     telegram_bot::{
//         TelegramBot,
//         commands::{CommandProcessor, mint::build_text_for_wallet_not_created},
//     },
//     utils::database_connection::get_db_connection,
// };
// use anyhow::Context;
// use teloxide::{
//     prelude::*,
//     types::{InlineKeyboardButton, InlineKeyboardMarkup},
// };

// pub struct Terminal;

// #[async_trait::async_trait]
// impl CommandProcessor for Terminal {
//     async fn process(
//         &self,
//         cfg: Arc<TelegramBot<Cache>>,
//         bot: Bot,
//         msg: Message,
//     ) -> anyhow::Result<()> {
//         let from = msg.from.context("Message missing sender")?;
//         let mut conn = get_db_connection(&cfg.pool).await?;
//         let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
//         let _ = if let Some(existing_user) = maybe_existing_user {
//             existing_user
//         } else {
//             bot.send_message(msg.chat.id, build_text_for_wallet_not_created())
//                 .await?;
//             return Ok(());
//         };

//         let (value_str, unit) = cfg
//             .config
//             .jwt_config
//             .expires_in
//             .as_ref()
//             .map(|e| e.split_at(e.len() - 1))
//             .unwrap_or(("30", "d"));
//         let value: i64 = value_str.parse().unwrap_or(1);
//         let duration = match unit {
//             "s" => Duration::seconds(value),
//             "m" => Duration::minutes(value),
//             "h" => Duration::hours(value),
//             "d" => Duration::days(value),
//             _ => Duration::days(30),
//         };

//         let claims = TelegramClaims {
//             id: from.id.0.to_string(),
//             exp: (Utc::now().timestamp() + duration.num_seconds()) as usize,
//             iat: Utc::now().timestamp() as usize,
//         };

//         let token_result = jsonwebtoken::encode(
//             &Header::default(),
//             &claims,
//             &EncodingKey::from_secret(cfg.config.jwt_config.secret.as_ref()),
//         );

//         let token = match token_result {
//             Ok(token) => token,
//             Err(_) => {
//                 bot.send_message(msg.chat.id, "Failed to generate token")
//                     .await?;
//                 return Ok(());
//             }
//         };
//         let login_url = Url::parse(&format!(
//             "{}/tg-login?token={}",
//             cfg.config.terminal_url, token
//         ))?;
//         let kb = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
//             "Login on terminal",
//             login_url,
//         )]]);
//         let text = "Click on the button below to go to terminal webpage";
//         bot.send_message(msg.chat.id, text).reply_markup(kb).await?;
//         Ok(())
//     }
// }
