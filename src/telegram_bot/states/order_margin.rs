use crate::{
    cache::{Cache, ICache},
    models::db::users::User,
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
    utils::{
        database_connection::get_db_connection,
        decibel_transaction::place_order_to_subaccount,
        perps_math::{notional_price, position_size, position_value},
        view_requests::view_primary_subaccount,
    },
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

pub struct OrderMargin {
    pub market_name: String,
    pub is_long: bool,
    pub leverage: u8,
    pub balance: f64,
}

#[async_trait::async_trait]
impl StateProcessor for OrderMargin {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let from = msg.from.context("From is missing in message")?;
        let tg_id = from.id.0 as i64;

        let amount: BigDecimal = match text.parse::<BigDecimal>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(chat_id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };
        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }

        let balance = BigDecimal::from_str(&self.balance.to_string())?;

        if amount > balance {
            return Err(anyhow::anyhow!(
                "❌ Insufficient balance. You entered: {} USDC\nAvailable: {:.2} USDC",
                amount,
                balance
            ));
        }

        let market = cfg
            .cache
            .get_market(&self.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market. Please try again"))?;
        let asset_context = cfg
            .cache
            .get_asset_context(&market.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market data. Please try again"))?;
        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Wallet not created yet. Type /start to create wallet")
            })?;

        if !db_user.degen_mode {
            let order_type = if self.is_long { "long" } else { "short" };
            let text = format!(
                "<b>✅ Order Summary</b>\n\n\
                You are opening a <b>{}</b> position on <b>{}</b>\n\
                • Amount: <b>{} USDC</b>\n\
                • Entry Price: <b>${:.4}</b>\n\
                • Leverage: <b>{}x</b>\n\n\
                Confirm to proceed or cancel to go back.",
                order_type, market.market_name, amount, asset_context.mark_price, self.leverage
            );
            let kb = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback(
                    "✅ Yes",
                    UserAction::PlaceOrder {
                        market_name: self.market_name.clone(),
                        is_long: self.is_long,
                        leverage: self.leverage,
                        amount,
                    }
                    .to_string(),
                ),
                InlineKeyboardButton::callback("❌ Cancel", UserAction::Cancel.to_string()),
            ]]);
            bot.send_message(chat_id, text)
                .reply_markup(kb)
                .parse_mode(ParseMode::Html)
                .await?;
        } else {
            let request = view_primary_subaccount(&cfg.config.contract_address, &db_user.address)?;
            let response = cfg.aptos_client.view(&request).await?;
            let value = response
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Primary subaccount not found"))?;
            let subaccount = value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Expected primary subaccount as string"))?;
            let entry_price = asset_context.mark_price.clone();
            let notional_price = notional_price(&amount, self.leverage);
            let position_size = position_size(&notional_price, &entry_price);
            let order_size = position_value(&position_size, &entry_price);
            let slippage: BigDecimal = BigDecimal::from_str("0.2")?; // 20% slippage
            let adjusted_price = if self.is_long {
                &entry_price * (BigDecimal::from_str("1.0")? + &slippage)
            } else {
                &entry_price * (BigDecimal::from_str("1.0")? - &slippage)
            };

            let rounded_price = adjusted_price.with_scale(2);
            let scaled_price = &rounded_price * BigDecimal::from_str("100000000")?;
            let price = scaled_price.with_scale(0).to_string().parse::<u64>()?;
            // size
            let rounded_size = order_size.with_scale(2);
            let scaled_size = &rounded_size * BigDecimal::from_str("100000")?;
            let size = scaled_size.with_scale(0).to_string().parse::<u64>()?;
            let payload = place_order_to_subaccount(
                &cfg.config.contract_address,
                subaccount,
                &market.market_addr,
                price,
                size,
                self.is_long,
                0,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )?;
            let txn = cfg
                .aptos_client
                .sign_txn_with_turnkey_and_fee_payer(&db_user.address, &db_user.public_key, payload)
                .await?;

            let txn_hash = cfg.aptos_client.submit_transaction_and_wait(txn).await?;

            tracing::info!(
                "{} placed order to subaccount {}: https://explorer.aptoslabs.com/txn/{}?network=decibel",
                db_user.address,
                subaccount,
                txn_hash.clone()
            );

            let order_type = if self.is_long { "long" } else { "short" };
            bot.send_message(
            chat_id,
            format!("✅ Trade opened! <b>{} {} {}x</b> for <b>{} USDC</b> at <b>${}</b> <a href='https://explorer.aptoslabs.com/txn/{}?network=decibel'>View Txn</a>", self.market_name, order_type.to_uppercase(), self.leverage, amount, asset_context.mark_price.clone(), txn_hash),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        }

        Ok(())
    }
}
