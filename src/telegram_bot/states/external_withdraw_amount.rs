use std::sync::Arc;

use bigdecimal::BigDecimal;
use teloxide::{
    payloads::SendMessageSetters, prelude::Requester,
    types::ParseMode,
};

use crate::{
    cache::Cache,
    telegram_bot::{
        states::{PendingState, StateProcessor}, TelegramBot
    },
};

pub struct ExternalWithdrawAmount{
    pub balance: BigDecimal
}

#[async_trait::async_trait]
impl StateProcessor for ExternalWithdrawAmount {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let amount: BigDecimal = match text.parse::<BigDecimal>() {
            Ok(num) => num,
            Err(_) => {
                bot.send_message(msg.chat.id, "Please enter a valid number")
                    .await?;
                return Ok(());
            }
        };

        {
            let mut state = cfg.state.lock().await;
            state.remove(&chat_id);
        }

        if amount > self.balance {
            bot.send_message(chat_id, "You don't have enough USDC balance").await?;
            return Ok(())
        };

        {
            let mut state = cfg.state.lock().await;
            state.insert(chat_id, PendingState::ExternalWithdrawAddress { amount: amount.clone() });
        }

        let text = format!(
            "🌐 <b>Withdraw to External Wallet</b>\n\n\
            Amount to send: <b>{} USDC</b>\n\n\
            Please enter the destination wallet address where you want to send the funds:\n\
            ⚠️ Double-check the address — once submitted, the transaction will be sent automatically and cannot be reversed.",
            amount
        );

        bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
        bot.send_message(chat_id, "Reply with the destination address:").await?;
        Ok(())
    }
}
