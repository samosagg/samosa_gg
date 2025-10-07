use std::sync::Arc;

use bigdecimal::BigDecimal;
use teloxide::{
    payloads::EditMessageTextSetters, prelude::Requester, types::ParseMode
};
use uuid::Uuid;

use crate::{
    cache::Cache, db_models::{tokens::Token, wallets::Wallet}, telegram_bot::{build_text_for_contact_support, commands::mint::build_text_for_wallet_not_created, states::StateProcessor, TelegramBot}, utils::{database_connection::get_db_connection, view_requests::view_fa_balance_request, wallet_transaction::transfer_fa}
};


pub struct WithdrawAmount {
    pub user_id: Uuid,
    pub token: String,
    pub address: String
}

#[async_trait::async_trait]
impl StateProcessor for WithdrawAmount {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let amount: BigDecimal = match text.parse::<BigDecimal>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(msg.chat.id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };
        let mut conn = get_db_connection(&cfg.pool).await?;
        let wallet_opt = Wallet::get_primary_wallet_by_user_id(self.user_id, &mut conn).await?;
        let db_wallet = if let Some(wallet) = wallet_opt {
            wallet 
        } else {
            bot.send_message(msg.chat.id, build_text_for_wallet_not_created()).await?;
            return Ok(())
        };

        let token_opt = Token::get_token_by_symbol(self.token.clone(), &mut conn).await?;

        let db_token = if let Some(token) = token_opt {
            token 
        } else {
            bot.send_message(msg.chat.id, build_text_for_contact_support()).await?;
            return Ok(())
        };

        let multiplier = BigDecimal::from(10u64.pow(db_token.decimals as u32));
        let amount_in_base_units = &amount * multiplier;

        let amount_u64 = amount_in_base_units
            .to_string()
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Failed to convert amount to u64"))?;
        
        let request = view_fa_balance_request(
            &db_token.address,
            &db_wallet.address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Expected a balance value but received none."))?
            .clone();
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;

        {
            let mut state = cfg.state.lock().await;
            state.remove(&msg.chat.id);
        }

        if balance < amount_u64 {
            bot.send_message(msg.chat.id, "Insufficient balance to withdraw").await?;
            return Ok(())
        }

        let processing_message = bot
            .send_message(msg.chat.id, "Processing withdraw request, please wait...")
            .await?;

        let payload = transfer_fa(
            &db_token.address, 
            &self.address, 
            amount_u64
        )?;

         let hash = cfg
            .aptos_client
            .sign_submit_txn_with_turnkey_and_fee_payer(
                &db_wallet.address,
                &db_wallet.public_key,
                payload,
            )
            .await?;
        tracing::info!(
            "Withdraw hash: {}, sender({})",
            hash,
            &db_wallet.address 
        );
        
        bot.edit_message_text(
            msg.chat.id,
            processing_message.id,
            format!("Withdraw success\nhash:{}", hash),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
        Ok(())
    }
}
