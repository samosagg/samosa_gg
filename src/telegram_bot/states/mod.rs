pub mod order_pair;
pub mod order_margin;
pub mod limit_pair;
pub mod limit_price;
pub mod limit_order_margin;
// pub mod ask_slippage;
// pub mod deposit_to_sub_amount;
// pub mod withdraw_address;
// pub mod withdraw_amount;

use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::{cache::Cache, telegram_bot::TelegramBot};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PendingState {
    // WaitingForOrderMargin {
    //     is_long: bool,
    //     market_name: String,
    //     leverage: u8,
    // },
    // WaitingForSlippage,
    // WaitingForWithdrawAddress {
    //     user_id: Uuid,
    //     token: String,
    // },
    // WaitingForWithdrawAmount {
    //     user_id: Uuid,
    //     token: String,
    //     address: String,
    // },
    // WaitingForSubAccountDepositAmount {
    //     wallet_id: Uuid,
    //     subaccount_id: Uuid,
    //     token: String,
    // },
    OrderPair {
        is_long: bool,
    },
    OrderMargin {
        market_name: String,
        is_long: bool,
        leverage: u8
    },
    LimitPair,
    LimitPrice {
        market_name: String
    },
    LimitOrderMargin {
        market_name: String,
        price: BigDecimal,
        leverage: u8
    }
}

#[async_trait::async_trait]
pub trait StateProcessor {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()>;
}
