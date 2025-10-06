use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, ParseMode},
};

use crate::{
    cache::Cache,
    telegram_bot::{
        actions::CallbackQueryProcessor, escape_markdown_v2, states::PendingState, TelegramBot
    },
};

pub struct OrderLeverage {
    pub market: String,
    pub order_type: String,
    pub leverage: u64,
}

#[async_trait::async_trait]
impl CallbackQueryProcessor for OrderLeverage {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: Bot,
        callback_query: CallbackQuery,
    ) -> anyhow::Result<()> {
        let msg = callback_query
            .message
            .ok_or_else(|| anyhow::anyhow!("Message missing in callback query"))?;
        let market = self.market.clone();
        let order_type = self.order_type.clone();
        let leverage = self.leverage;
        bot.send_message(
            msg.chat().id,
            escape_markdown_v2(&build_text_for_choose_position_size(&market, &order_type)),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
        {
            let mut state = cfg.state.lock().await;
            state.insert(
                msg.chat().id,
                PendingState::WaitingForOrderMargin {
                    order_type,
                    market,
                    leverage,
                },
            );
        }

        Ok(())
    }
}

fn build_text_for_choose_position_size(market: &str, order_type: &str) -> String {
    format!(
        "Choose Position Size\n\n\
    Reply with the amount of margin in $ that you would like to {} {} with
    ",
    order_type,
        market
    )
}
