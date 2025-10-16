pub mod custom_slippage;
pub mod deposit_to_subaccount;
pub mod external_withdraw_address;
pub mod external_withdraw_amount;
pub mod limit_order_margin;
pub mod order_margin;
pub mod order_pair;

use bigdecimal::BigDecimal;

use crate::{cache::Cache, telegram_bot::TelegramBot};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PendingState {
    OrderPair {
        is_long: bool,
        balance: f64,
    },
    OrderMargin {
        market_name: String,
        is_long: bool,
        leverage: u8,
        balance: f64,
    },
    UpdateSlippage,
    DepositToSubaccount {
        address: String,
        balance: BigDecimal,
    },
    ExternalWithdrawAmount {
        balance: BigDecimal,
    },
    ExternalWithdrawAddress {
        amount: BigDecimal,
    },
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
