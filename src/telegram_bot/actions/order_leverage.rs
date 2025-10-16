use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, ParseMode},
};

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::CallbackQueryProcessor, states::PendingState},
};

pub struct OrderLeverage {
    pub market_name: String,
    pub is_long: bool,
    pub leverage: u8,
    pub balance: f64,
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
            .ok_or_else(|| anyhow::anyhow!("Message is missing in callback query"))?;
        let chat_id = msg.chat().id;

        {
            let mut state = cfg.state.lock().await;
            state.insert(
                msg.chat().id,
                PendingState::OrderMargin {
                    market_name: self.market_name.clone(),
                    is_long: self.is_long,
                    leverage: self.leverage,
                    balance: self.balance,
                },
            );
        }

        bot.send_message(
            chat_id,
            format!(
                "<b>💵 Enter the USDC amount you want to trade</b>\n\n\
            Example: <code>{}</code>\n\
            (This is in USDC — make sure you have enough balance in your wallet.)",
                self.balance
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;

        Ok(())
    }
}
