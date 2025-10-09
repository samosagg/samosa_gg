use crate::{
    cache::{Cache, ICache},
    db_models::{tokens::Token, users::User},
    telegram_bot::{
        actions::UserAction, commands::mint::build_text_for_wallet_not_created, escape_markdown_v2, states::StateProcessor, TelegramBot
    },
    utils::{database_connection::get_db_connection, perps_math::{liquidation_price, notional_price, position_size, position_value}},
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
        let from = msg.from.context("Message missing sender")?;
        let telegram_id = from.id.0 as i64;
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
        let market_opt = cfg.cache.get_market(&self.market_name).await;
        let market = if let Some(mkt) = market_opt {
            mkt
        } else {
            bot.send_message(
                chat_id,
                "Missing market data, please try placing order again",
            )
            .await?;
            return Ok(());
        };
        let context = cfg.cache.get_asset_context(&market.market_name).await;
        let asset_context = if let Some(ctx) = context {
            ctx
        } else {
            bot.send_message(
                msg.chat.id,
                "Missing market context, please try placing order again",
            )
            .await?;
            return Ok(());
        };
        let mut conn = get_db_connection(&cfg.pool).await?;
        let maybe_existing_user = User::get_by_telegram_id(telegram_id, &mut conn).await?;
        let db_user = if let Some(user) = maybe_existing_user {
            user
        } else {
            bot.send_message(chat_id, build_text_for_wallet_not_created())
                .await?;
            return Ok(());
        };
        if db_user.degen_mode {
            let maybe_token = Token::get_token_by_symbol(db_user.token, &mut conn).await?;
            let db_token = if let Some(token) = maybe_token {
                token 
            } else {
                bot.send_message(chat_id, "Token not found by symbol").await?;
                return Ok(())
            };
            
            
        } else {
            let entry_price = asset_context.mark_price.clone();
            let notional_price = notional_price(&amount, self.leverage);
            let position_size = position_size(&notional_price, &entry_price);
            let position_value = position_value(&position_size, &entry_price);
            let liq_price = liquidation_price(self.is_long, &entry_price, self.leverage, &BigDecimal::from_str("0.001").unwrap());
            let text = format!(
                "Placing {} order\n\n\
                Entry price ${}\n\
                Margin ${}\n\
                Order Size {}/{} {}\n\
                Liquidation Price ${}\n\
                Leverage {}x\n\n\
                Click on the Confirm Order button below to confirm your order",
                if self.is_long { "long" } else { "short" },
                entry_price.with_scale(4),
                amount.with_scale(4),
                position_size.with_scale(4),
                position_value.with_scale(4),
                &market.market_name,
                liq_price.with_scale(4),
                self.leverage
            );
            // let keyboard = build_order_confirm_keyboard(
            //     &market.market_name,
            //     &self.order_type,
            //     self.leverage,
            //     amount,
            // );
            bot.send_message(msg.chat.id, escape_markdown_v2(&text))
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(InlineKeyboardMarkup::new(
                    vec![
                        vec![
                            InlineKeyboardButton::callback("Confirm Order", "callback_data")
                        ]
                    ]
                ))
                .await?;
        };

        Ok(())
    }
}

fn build_text_for_order_confirmation(
    market_name: &str,
    entry_price: BigDecimal,
    liq_price: BigDecimal,
) -> String {
    format!(
        "Confirm your order for {}\n\n\
        Entry price: {}\n\
        Liquidation price: {}
        ",
        market_name, entry_price, liq_price
    )
}

fn calculate_liquidation_price(
    mark_price: &BigDecimal,
    leverage: &BigDecimal,
    maintenance_margin: &BigDecimal,
    is_long: bool,
) -> BigDecimal {
    let one = BigDecimal::from(1);
    if is_long {
        mark_price * (&one - &one / leverage + maintenance_margin)
    } else {
        mark_price * (&one + &one / leverage - maintenance_margin)
    }
}

fn build_order_confirm_keyboard(
    market: &str,
    order_type: &str,
    leverage: u64,
    amount: BigDecimal,
) {
    // let callback_data: String = UserAction::ConfirmOrder {
    //     market: market.into(),
    //     order_type: order_type.into(),
    //     leverage,
    //     amount,
    // }
    // .to_string();
    // InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
    //     "Confirm Order",
    //     callback_data,
    // )]])
}
