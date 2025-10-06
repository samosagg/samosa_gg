use std::sync::Arc;

use crate::{
    cache::Cache,
    db_models::{subaccounts::SubAccount, users::User},
    telegram_bot::{
        TelegramBot,
        commands::{CommandProcessor, mint::build_text_for_wallet_not_created},
    },
    utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
};
use anyhow::Context;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct Wallet;

#[async_trait::async_trait]
impl CommandProcessor for Wallet {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let from = msg.from.context("Message missing sender")?;
        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;
        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat.id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let sub_accounts = SubAccount::get_subaccounts_by_user_id(user.id, &mut conn).await?;

        let request = view_fa_balance_request(
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            &user.wallet_address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Expected a balance value but received none."))?
            .clone();
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;

        let mut text =
            build_text_for_wallet_with_balance(&user.wallet_address, balance / 10u64.pow(6));

        if !sub_accounts.is_empty() {
            text.push_str("\n\nSub Accounts:\n");
            for (_, sub) in sub_accounts.iter().enumerate() {
                text.push_str(&build_text_for_subaccount(&sub.address));
                if let Some(true) = sub.is_primary {
                    text.push_str(" \\(Primary\\)");
                }
                text.push_str("\n");
            }
        }
        bot.send_message(msg.chat.id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }
}

fn build_text_for_wallet_with_balance(address: &str, balance: u64) -> String {
    format!("Main Account: \n`{}`\nBalance\\: {} USDC", address, balance)
}

fn build_text_for_subaccount(address: &str) -> String {
    format!("`{}`", address)
}
