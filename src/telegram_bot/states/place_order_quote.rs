use crate::{
    cache::{Cache, ICache},
    telegram_bot::{TelegramBot, actions::UserAction, escape_markdown_v2, states::StateProcessor},
};
use bigdecimal::BigDecimal;
use std::sync::Arc;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

pub struct PlaceOrderQuote {
    pub market: String,
    pub order_type: String,
    pub leverage: u64,
}

#[async_trait::async_trait]
impl StateProcessor for PlaceOrderQuote {
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
        let market_opt = cfg.cache.get_market(&self.market).await;
        let market = if let Some(market) = market_opt {
            market
        } else {
            bot.send_message(
                msg.chat.id,
                "Market missing, please try placing order again",
            )
            .await?;
            return Ok(());
        };
        let context = cfg.cache.get_asset_context(&market.market_name).await;
        let asset = if let Some(asset_context) = context {
            asset_context
        } else {
            bot.send_message(msg.chat.id, "Failed to get pair details")
                .await?;
            return Ok(());
        };
        let is_long = if self.order_type == "long" {
            true
        } else {
            false
        };
        let custom_mm = BigDecimal::parse_bytes(b"1.5", 10);
        let mm = if let Some(mm) = custom_mm {
            mm
        } else {
            bot.send_message(msg.chat.id, "Failed to get maintainence margin")
                .await?;
            return Ok(());
        };
        let liq_price = calculate_liquidation_price(
            &asset.mark_price,
            &BigDecimal::from(self.leverage),
            &mm,
            is_long,
        );
        let text =
            build_text_for_order_confirmation(&market.market_name, asset.mark_price, liq_price);
        let keyboard = build_order_confirm_keyboard(
            &market.market_name,
            &self.order_type,
            self.leverage,
            amount,
        );
        bot.send_message(msg.chat.id, escape_markdown_v2(&text))
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(keyboard)
            .await?;
        {
            let mut state = cfg.state.lock().await;
            state.remove(&msg.chat.id);
        }
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
) -> InlineKeyboardMarkup {
    let callback_data: String = UserAction::ConfirmOrder {
        market: market.into(),
        order_type: order_type.into(),
        leverage,
        amount,
    }
    .to_string();
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Confirm Order",
        callback_data,
    )]])
}
