pub mod ask_slippage;
pub mod long_pair;
pub mod place_order_quote;
pub mod short_pair;
pub mod withdraw_address;
pub mod withdraw_amount;

use uuid::Uuid;

use crate::{cache::Cache, telegram_bot::TelegramBot};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PendingState {
    WaitingForLongPair,
    WaitingForShortPair,
    WaitingForOrderMargin {
        order_type: String,
        market: String,
        leverage: u64,
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
