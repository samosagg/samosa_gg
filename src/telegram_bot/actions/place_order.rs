use std::sync::Arc;

use anyhow::Context;
use bigdecimal::BigDecimal;
use teloxide::{
    Bot,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Requester,
    types::{CallbackQuery, ParseMode},
};

use crate::{
    cache::{Cache, ICache},
    db_models::{subaccounts::SubAccount, users::User, wallets::Wallet},
    telegram_bot::{
        actions::CallbackQueryProcessor, build_text_for_contact_support, commands::mint::build_text_for_wallet_not_created, escape_markdown_v2, TelegramBot
    },
    utils::{
        database_connection::get_db_connection, decibel_transaction::place_order_to_subaccount,
    },
};

pub struct PlaceOrder {
    pub market: String,
    pub order_type: String,
    pub leverage: u64,
    pub amount: BigDecimal,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for PlaceOrder {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
        let from = callback_query.from;
        let chat_id = msg.chat().id;

        let market_opt = cfg.cache.get_market(&self.market).await;
        let market = if let Some(market) = market_opt {
            market
        } else {
            bot.send_message(
                chat_id,
                "Market missing, please try placing order again",
            )
            .await?;
            return Ok(());
        };

        let processing_message = bot
            .send_message(chat_id, "Placing your order, please wait...")
            .await?;

        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;

        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let db_user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(chat_id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let maybe_wallet = Wallet::get_primary_wallet_by_user_id(db_user.id, &mut conn).await?;
        let db_wallet = if let Some(wallet) = maybe_wallet {
            wallet 
        } else {
            bot.send_message(chat_id, build_text_for_contact_support()).await?;
            return Ok(())
        };
        let maybe_subaccount = SubAccount::get_primary_subaccount_by_wallet_id(db_wallet.id, &mut conn).await?;
        let db_subaccount = if let Some(subaccount) = maybe_subaccount {
            subaccount 
        } else {
            bot.send_message(chat_id, "Sub account not found, please contact support").await?;
            return Ok(())
        };
        let is_long = if self.order_type == "long" {
            true
        } else {
            false
        };
        let payload = place_order_to_subaccount(
            &cfg.config.contract_address,
            &db_subaccount.address,
            &market.market_addr,
            self.amount.to_string().parse::<u64>()?,
            1000u64,
            is_long,
            self.leverage,
        )?;
         let signed_txn = cfg.aptos_client.sign_txn_with_turnkey_and_fee_payer(
            &db_wallet.address, 
            &db_wallet.public_key, 
            payload
        ).await?;

        // let vm_error = cfg.aptos_client.simulate_transaction(&signed_txn).await?;
        // if let Some(err) = vm_error {
        //     bot.send_message(chat_id, err).await?;
        //     return Ok(())
        // } else {
        //     println!("Simulation success");
        // };

        let hash = cfg
            .aptos_client
            .submit_transaction_and_wait(
                signed_txn
            )
            .await?;
        tracing::info!(
            "Place order to subaccount hash: {}, sender({})",
            hash,
            &db_wallet.address
        );

        bot.edit_message_text(
            msg.chat().id,
            processing_message.id,
            escape_markdown_v2(&build_text_for_order_placed(&"empty")),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    }
}

fn build_text_for_order_placed(hash: &str) -> String {
    format!(
        "Order placed successfully\n\
    [View Txn](https://explorer.aptoslabs.com/txn/{}?newtork=decibel)
    ",
        hash
    )
}
