use std::sync::Arc;

use bigdecimal::BigDecimal;
use teloxide::{
    dispatching::dialogue::GetChatId, payloads::EditMessageTextSetters, prelude::Requester,
    types::ParseMode,
};
use uuid::Uuid;

use crate::{
    cache::Cache,
    db_models::{subaccounts::SubAccount, tokens::Token, wallets::Wallet},
    telegram_bot::{
        TelegramBot, build_text_for_contact_support,
        commands::mint::build_text_for_wallet_not_created, states::StateProcessor,
    },
    utils::{
        database_connection::get_db_connection, decibel_transaction::deposit_to_subaccount,
        view_requests::view_fa_balance_request, wallet_transaction::transfer_fa,
    },
};

pub struct DepositToSubaccountAmount {
    pub wallet_id: Uuid,
    pub subaccount_id: Uuid,
    pub token: String,
}

#[async_trait::async_trait]
impl StateProcessor for DepositToSubaccountAmount {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let amount: BigDecimal = match text.parse::<BigDecimal>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(msg.chat.id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };
        let mut conn = get_db_connection(&cfg.pool).await?;
        let maybe_token = Token::get_token_by_symbol(self.token.clone(), &mut conn).await?;
        let db_token = if let Some(token) = maybe_token {
            token
        } else {
            bot.send_message(chat_id, "Failed to get token by sybmol")
                .await?;
            return Ok(());
        };
        let maybe_subaccount = SubAccount::get_by_id(self.subaccount_id, &mut conn).await?;
        let db_subaccount = if let Some(subaccount) = maybe_subaccount {
            subaccount
        } else {
            bot.send_message(chat_id, "Failed to get subaccount by id")
                .await?;
            return Ok(());
        };
        let maybe_wallet = Wallet::get_by_id(db_subaccount.wallet_id, &mut conn).await?;
        let db_wallet = if let Some(wallet) = maybe_wallet {
            wallet
        } else {
            bot.send_message(chat_id, "Failed to get wallet by id")
                .await?;
            return Ok(());
        };
        let multiplier = BigDecimal::from(10u64.pow(db_token.decimals as u32));
        let amount_in_base_units = &amount * multiplier;

        let amount_u64 = amount_in_base_units
            .to_string()
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Failed to convert amount to u64"))?;
        let payload = deposit_to_subaccount(
            &cfg.config.contract_address,
            &db_subaccount.address,
            &db_token.address,
            amount_u64,
        )?;
        let signed_txn = cfg
            .aptos_client
            .sign_txn_with_turnkey_and_fee_payer(&db_wallet.address, &db_wallet.public_key, payload)
            .await?;
        let hash = cfg
            .aptos_client
            .submit_transaction_and_wait(signed_txn)
            .await?;
        tracing::info!(
            "Deposit to subaccount: {}, sender:{}",
            db_subaccount.address,
            db_wallet.address
        );
        bot.send_message(chat_id, format!("Hash:{}", hash)).await?;
        Ok(())
    }
}
