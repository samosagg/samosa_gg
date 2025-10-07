// use anyhow::Context;
use std::sync::Arc;
use teloxide::prelude::*;
use uuid::Uuid;

use crate::{
    cache::Cache,
    db_models::{tokens::Token, wallets::Wallet},
    telegram_bot::{
        TelegramBot, actions::CallbackQueryProcessor,
        commands::mint::build_text_for_wallet_not_created,
    },
    utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request},
};

pub struct Balances {
    pub user_id: Uuid,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for Balances {
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
        let primary_wallet_opt =
            Wallet::get_primary_wallet_by_user_id(self.user_id, &mut conn).await?;
        let wallet = if let Some(primary_wallet) = primary_wallet_opt {
            primary_wallet
        } else {
            bot.send_message(msg.chat().id, build_text_for_wallet_not_created())
                .await?;
            return Ok(());
        };

        let tokens = Token::get_tokens(&mut conn).await?;
        let mut text = String::from("Balances: \n");

        if !tokens.is_empty() {
            for (idx, db_token) in tokens.iter().enumerate() {
                let request = view_fa_balance_request(&db_token.address, &wallet.address)?;
                let response = cfg.aptos_client.view(&request).await?;
                let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));

                let balance = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;
                let size = balance / 10u64.pow(db_token.decimals as u32);
                text.push_str(&format!(
                    "{} {}\n\
                        Value: ${}\n\
                        Size: {}",
                    idx + 1,
                    db_token.symbol,
                    size,
                    size
                ));
                text.push_str("\n\n");
            }
        }

        bot.send_message(msg.chat().id, text).await?;
        Ok(())
    }
}
