use std::sync::Arc;

use bigdecimal::BigDecimal;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    cache::Cache,
    telegram_bot::{TelegramBot, actions::UserAction, states::StateProcessor},
};

pub struct DepositToSubaccount {
    pub address: String,
    pub balance: BigDecimal
}

#[async_trait::async_trait]
impl StateProcessor for DepositToSubaccount {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let amount = match text.parse::<BigDecimal>() {
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
        if amount > self.balance {
            bot.send_message(chat_id, "You don't have enough USDC balance").await?;
            return Ok(())
        };

      let text = format!(
        "⚡ You’re sending <b>{} USDC</b> to your subaccount!\n\n\
        🧾 <b>Subaccount:</b> <code>{}</code>\n\n\
        Double-check the amount before confirming — once it’s in, it’s ready for trading 🚀",
        amount, 
        self.address
        );


        let markup = InlineKeyboardMarkup::new(
            vec![
                vec![
                    InlineKeyboardButton::callback("✅ Confirm Deposit", UserAction::ConfirmSubaccountDeposit { amount }.to_string()),
                    InlineKeyboardButton::callback("❌ Cancel", UserAction::Cancel.to_string()),
                ]
            ]
        );

        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(markup)
            .await?;

        Ok(())
    }
}
