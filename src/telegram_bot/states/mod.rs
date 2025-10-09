pub mod ask_slippage;
pub mod deposit_to_sub_amount;
pub mod order_margin;
pub mod order_pair;
pub mod withdraw_address;
pub mod withdraw_amount;

use uuid::Uuid;

use crate::{cache::Cache, telegram_bot::TelegramBot};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PendingState {
    WaitingForOrderMargin {
        is_long: bool,
        market_name: String,
        leverage: u8,
    },
    WaitingForSlippage,
    WaitingForWithdrawAddress {
        user_id: Uuid,
        token: String,
    },
    WaitingForWithdrawAmount {
        user_id: Uuid,
        token: String,
        address: String,
    },
    WaitingForSubAccountDepositAmount {
        wallet_id: Uuid,
        subaccount_id: Uuid,
        token: String,
    },
    WaitingForOrderPair {
        is_long: bool,
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
