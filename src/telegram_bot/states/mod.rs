pub mod long_pair;
pub mod place_order_quote;
pub mod short_pair;

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
