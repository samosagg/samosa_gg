pub mod custom_slippage;
pub mod deposit_to_subaccount;
pub mod limit_order_margin;
pub mod limit_pair;
pub mod limit_price;
pub mod order_margin;
pub mod order_pair;
pub mod external_withdraw_amount;
pub mod external_withdraw_address;

use bigdecimal::BigDecimal;

use crate::{cache::Cache, telegram_bot::TelegramBot};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PendingState {
    OrderPair {
        is_long: bool,
    },
    OrderMargin {
        market_name: String,
        is_long: bool,
        leverage: u8,
    },
    LimitPair,
    LimitPrice {
        market_name: String,
    },
    LimitOrderMargin {
        market_name: String,
        price: BigDecimal,
        leverage: u8,
    },
    UpdateSlippage,
    DepositToSubaccount {
        address: String,
        balance: BigDecimal
    },
    ExternalWithdrawAmount{
        balance: BigDecimal
    },
    ExternalWithdrawAddress {
        amount: BigDecimal
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
