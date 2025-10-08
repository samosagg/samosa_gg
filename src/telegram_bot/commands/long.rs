use std::sync::Arc;

use crate::cache::Cache;
use crate::db_models::users::User;
use crate::db_models::wallets::Wallet;
use crate::telegram_bot::build_text_for_contact_support;
use crate::telegram_bot::commands::mint::build_text_for_wallet_not_created;
use crate::telegram_bot::states::PendingState;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use crate::utils::database_connection::get_db_connection;
use anyhow::Context;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct Long;

#[async_trait::async_trait]
impl CommandProcessor for Long {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let from = msg.from.context("Message is missing sender")?;
        let chat_id = msg.chat.id;
        let mut conn = get_db_connection(&cfg.pool).await?;
        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let db_user = if let Some(user) = maybe_existing_user {
            user
        } else {
            bot.send_message(chat_id, build_text_for_wallet_not_created())
                .await?;
            return Ok(());
        };
        let maybe_wallet = Wallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
        let db_wallet = if let Some(wallet) = maybe_wallet {
            wallet
        } else {
            bot.send_message(chat_id, build_text_for_contact_support())
                .await?;
            return Ok(());
        };
        {
            let mut state = cfg.state.lock().await;
            state.insert(chat_id, PendingState::WaitingForOrderPair { is_long: true });
        }
        let text = build_text_for_asking_pair(true);
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}

fn build_text_for_asking_pair(is_long: bool) -> String {
    format!(
        "Enter the name of the token you want to place {} order to, e\\.g\\. apt or apt/usd",
        if is_long { "long" } else { "short" }
    )
}
