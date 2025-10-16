use std::str::FromStr;
use std::sync::Arc;

use crate::cache::{Cache, ICache};
use crate::models::db::users::User;
use crate::telegram_bot::{TelegramBot, commands::CommandProcessor};
use crate::utils::database_connection::get_db_connection;
use crate::utils::decibel_transaction::place_order_to_subaccount;
use crate::utils::perps_math::{notional_price, position_size, position_value};
use crate::utils::view_requests::{view_fa_balance_request, view_primary_subaccount};
use anyhow::Context;
use bigdecimal::BigDecimal;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct Limit;

#[async_trait::async_trait]
impl CommandProcessor for Limit {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        msg: Message,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let from = msg.from.as_ref().context("Missing from in message")?;
        let tg_id = from.id.0 as i64;

        let args = msg.text().context(limit_text())?;
        let parsed_args = args.split_whitespace().skip(1).collect::<Vec<&str>>();

        if parsed_args.len() < 5 {
            return Err(anyhow::anyhow!(
                "Invalid format: \nUsage:\n{}",
                limit_text()
            ));
        }
        let direction = parsed_args[0].to_string();
        if direction != "long" && direction != "short" {
            return Err(anyhow::anyhow!("Direction must be long or short"));
        }
        let asset = parsed_args[1].to_string();
        let similar_markets = cfg.cache.get_markets_ilike(&asset).await;
        if similar_markets.len() == 0 {
            return Err(anyhow::anyhow!("Ticker not found, try again"));
        }
        let market = similar_markets
            .first()
            .context("Ticker not found on first index")?;

        let leverage: u8 = match parsed_args[2].to_lowercase().trim_end_matches("x").parse() {
            Ok(num) if num >= 1 && num <= market.max_leverage => num,
            _ => {
                return Err(anyhow::anyhow!(
                    "Leverage must be between 1x and {}x for {}",
                    market.max_leverage,
                    market.market_name
                ));
            }
        };

        let limit_price: BigDecimal = {
            let price_str = parsed_args[3].trim_start_matches('$');
            match price_str.parse() {
                Ok(num) => num,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "⚠️ Invalid limit order price. Use format like $22.5"
                    ));
                }
            }
        };

        let amount_input = parsed_args[4].to_string();
        let (amount_usdc, amount_pct): (Option<BigDecimal>, Option<BigDecimal>) =
            if amount_input.ends_with('%') {
                let pct_str = amount_input.trim_end_matches('%');
                match BigDecimal::from_str(pct_str) {
                    Ok(num) if num > BigDecimal::from(0) && num <= BigDecimal::from(100) => {
                        (None, Some(num))
                    }
                    _ => return Err(anyhow::anyhow!("⚠️ Invalid percentage. Example: 50%")),
                }
            } else {
                let amt_str = amount_input.trim_start_matches('$');
                match BigDecimal::from_str(amt_str) {
                    Ok(num) if num > BigDecimal::from(0) => (Some(num), None),
                    _ => return Err(anyhow::anyhow!("⚠️ Invalid amount. Example: $10")),
                }
            };
        let asset_context = cfg
            .cache
            .get_asset_context(&market.market_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get market data. Please try again"))?;
        let mut conn = get_db_connection(&cfg.pool).await?;
        let db_user = User::get_by_telegram_id(tg_id, &mut conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Wallet not created, type /start to create wallet"))?;

        let request = view_fa_balance_request(
            "0x6555ba01030b366f91c999ac943325096495b339d81e216a2af45e1023609f02",
            &db_user.address,
        )?;
        let response = cfg.aptos_client.view(&request).await?;
        let balance_json = response.get(0).cloned().unwrap_or(serde_json::json!("0"));
        let balance: u64 = serde_json::from_value::<String>(balance_json)?.parse::<u64>()?;
        let usdc = (balance as f64) / 10f64.powi(6);

        let balance_bd = BigDecimal::from_str(&usdc.to_string())?;
        let amount_to_trade: BigDecimal = if let Some(usdc_val) = amount_usdc.clone() {
            if usdc_val > balance_bd {
                return Err(anyhow::anyhow!(
                    "❌ Insufficient balance.\nYour balance: {:.2} USDC\nYou entered: {} USDC",
                    usdc,
                    usdc_val
                ));
            }
            usdc_val
        } else if let Some(pct_val) = amount_pct.clone() {
            if pct_val > BigDecimal::from(100) || pct_val <= BigDecimal::from(0) {
                return Err(anyhow::anyhow!("⚠️ Percentage must be between 0% and 100%"));
            }
            &balance_bd * &pct_val / BigDecimal::from(100u32)
        } else {
            return Err(anyhow::anyhow!("⚠️ Amount not specified"));
        };

        let request = view_primary_subaccount(&cfg.config.contract_address, &db_user.address)?;
        let response = cfg.aptos_client.view(&request).await?;
        let value = response
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Primary subaccount not found"))?;
        let subaccount = value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected primary subaccount as string"))?;
        let entry_price = asset_context.mark_price.clone();
        let notional_price = notional_price(&amount_to_trade, leverage);
        let position_size = position_size(&notional_price, &entry_price);
        let order_size = position_value(&position_size, &entry_price);

        let scaled_price = &limit_price * BigDecimal::from_str("100000000")?;
        let price = scaled_price.with_scale(0).to_string().parse::<u64>()?;
        // size
        let rounded_size = order_size.with_scale(2);
        let scaled_size = &rounded_size * BigDecimal::from_str("100000")?;
        let size = scaled_size.with_scale(0).to_string().parse::<u64>()?;

        let is_buy = if direction == "long" { true } else { false };
        let payload = place_order_to_subaccount(
            &cfg.config.contract_address,
            subaccount,
            &market.market_addr,
            price,
            size,
            is_buy,
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

        bot.send_message(
            chat_id,
            format!("✅ Trade opened! <b>{} {} {}x</b> for <b>{} USDC</b> at <b>${}</b> <a href='https://explorer.aptoslabs.com/txn/{}?network=decibel'>View Txn</a>", market.market_name, direction.to_uppercase(), leverage, amount_to_trade, limit_price, txn_hash),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        Ok(())
    }
}

fn limit_text() -> String {
    return "/limit <long/short> <asset> <leverage> <limit-order-price> <amount/pct>".to_string();
}
