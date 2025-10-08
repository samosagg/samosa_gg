use crate::{
    cache::{Cache, ICache},
    db_models::users::User,
    telegram_bot::{
        TelegramBot, actions::UserAction, commands::mint::build_text_for_wallet_not_created,
        escape_markdown_v2, states::StateProcessor,
    },
    utils::database_connection::get_db_connection,
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use std::sync::Arc;
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
            // let payload =
        } else {
            // let liq_price = calculate_liquidation_price(
            //     &asset.mark_price,
            //     &BigDecimal::from(self.leverage),
            //     &mm,
            //     is_long,
            // );
            let text = "Order quote will be displayed here";
            // let keyboard = build_order_confirm_keyboard(
            //     &market.market_name,
            //     &self.order_type,
            //     self.leverage,
            //     amount,
            // );
            bot.send_message(msg.chat.id, escape_markdown_v2(&text))
                .parse_mode(ParseMode::MarkdownV2)
                // .reply_markup(keyboard)
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
