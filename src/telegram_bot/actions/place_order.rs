use std::sync::Arc;

use anyhow::Context;
use bigdecimal::BigDecimal;
use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters, SendMessageSetters},
    prelude::Requester,
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::{Cache, ICache},
    db_models::{subaccounts::SubAccount, users::User},
    telegram_bot::{
        TelegramBot, actions::CallbackQueryProcessor,
        commands::mint::build_text_for_wallet_not_created, states::long_pair::escape_markdown_v2,
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

        let market_opt = cfg.cache.get_market(&self.market).await;
        let market = if let Some(market) = market_opt {
            market
        } else {
            bot.send_message(
                msg.chat().id,
                "Market missing, please try placing order again",
            )
            .await?;
            return Ok(());
        };
        bot.edit_message_reply_markup(msg.chat().id, msg.id())
            .reply_markup(InlineKeyboardMarkup::new(vec![vec![]]))
            .await?;
        let processing_message = bot
            .send_message(msg.chat().id, "Placing your order, please wait...")
            .await?;

        let mut conn = get_db_connection(&cfg.pool)
            .await
            .context("Failed to get database connection")?;

        let maybe_existing_user = User::get_by_telegram_id(from.id.0 as i64, &mut conn).await?;
        let user = if let Some(existing_user) = maybe_existing_user {
            existing_user
        } else {
            bot.send_message(msg.chat().id, build_text_for_wallet_not_created())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let maybe_subaccount =
            SubAccount::get_primary_subaccount_by_user_id(user.id, &mut conn).await?;
        let subaccount = if let Some(sub_account) = maybe_subaccount {
            sub_account
        } else {
            bot.send_message(msg.chat().id, "No primary sub account found")
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        };
        let is_long = if self.order_type == "long" {
            true
        } else {
            false
        };
        let payload = place_order_to_subaccount(
            &cfg.config.contract_address,
            &subaccount.address,
            &market.market_addr,
            self.amount.to_string().parse::<u64>()?,
            1000,
            is_long,
            self.leverage,
        )?;
        let hash = cfg
            .aptos_client
            .sign_submit_txn_with_turnkey_and_fee_payer(
                &user.wallet_address,
                &user.wallet_public_key,
                payload,
            )
            .await?;
        tracing::info!(
            "Place order to subaccount hash: {}, sender({})",
            hash,
            &user.wallet_address
        );

        bot.edit_message_text(
            msg.chat().id,
            processing_message.id,
            escape_markdown_v2(&build_text_for_order_placed(&hash)),
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
