// use anyhow::Context;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::ParseMode,
};
use uuid::Uuid;

use crate::{
    cache::Cache,
    db_models::{tokens::Token, wallets::Wallet},
    telegram_bot::{
        TelegramBot, actions::CallbackQueryProcessor, build_text_for_contact_support,
        escape_markdown_v2,
    },
    utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
};

pub struct Accounts {
    pub user_id: Uuid,
    pub token: String,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for Accounts {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;

        let mut conn = get_db_connection(&cfg.pool).await?;
        let wallets = Wallet::get_wallets_by_user_id(self.user_id, &mut conn).await?;
        let token_opt = Token::get_token_by_symbol(self.token.clone(), &mut conn).await?;

        let db_token = if let Some(token) = token_opt {
            token
        } else {
            bot.send_message(msg.chat().id, build_text_for_contact_support())
                .await?;
            return Ok(());
        };

        let pending_message = bot
            .send_message(msg.chat().id, "Fetching wallets...")
            .await?;
        let mut text = String::new();

        if !wallets.is_empty() {
            for (idx, wallet) in wallets.iter().enumerate() {
                let request = view_fa_balance_request(&db_token.address, &wallet.address)?;
                let response = cfg.aptos_client.view(&request).await?;
                let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));

                let balance = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;

                text.push_str(&build_text_for_wallet(
                    idx + 1,
                    &wallet.address,
                    wallet.is_primary,
                    balance / 10u64.pow(db_token.decimals as u32),
                    &db_token.symbol,
                ));
                text.push_str("\n\n");
            }
        }

        bot.edit_message_text(msg.chat().id, pending_message.id, escape_markdown_v2(&text))
            // .reply_markup(build_keyboard_for_accounts())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}

fn build_text_for_wallet(
    idx: usize,
    address: &str,
    is_primary: bool,
    balance: u64,
    symbol: &str,
) -> String {
    format!(
        "{}. {} {}\n\n\
        Address: `{}`\n\
        Balance: {} {}\n\
        Secured by Turnkey",
        idx,
        &address[..5],
        if is_primary { "- Primary" } else { "" },
        address,
        balance,
        symbol
    )
}

// fn build_keyboard_for_accounts() -> InlineKeyboardMarkup {
//     InlineKeyboardMarkup::new(
//         vec![
//             vec![
//                 InlineKeyboardButton::callback("Set primary account", "todo")
//             ],
//             vec![
//                 InlineKeyboardButton::callback("New Decibel Account", "todo")
//             ],
//             vec![
//                 InlineKeyboardButton::callback("Manage Account", "todo")
//             ],
//         ]
//     )
// }
