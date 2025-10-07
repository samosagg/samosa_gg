use std::sync::Arc;

use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods};
use teloxide::{
    prelude::Requester,
};
use uuid::Uuid;

use crate::{
    cache::Cache, telegram_bot::{states::{PendingState, StateProcessor}, TelegramBot},
};

use aptos_sdk::types::account_address::AccountAddress;

pub struct WithdrawAddress {
    pub user_id: Uuid,
    pub token: String,
}

#[async_trait::async_trait]
impl StateProcessor for WithdrawAddress {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        msg: teloxide::types::Message,
        text: String,
    ) -> anyhow::Result<()> {
        let address = AccountAddress::from_hex_literal(&text)?;
        {
            let mut state = cfg.state.lock().await;
            state.insert(msg.chat.id, PendingState::WaitingForWithdrawAmount { user_id: self.user_id, token: self.token.clone(), address: address.to_string() });
        }
        bot.send_message(msg.chat.id, format!(
            "Entered address: {}\n\nEnter the amount in {} you want to withdraw\n\nNote: This process is irreversible, please check the address carefully",
            address.to_string(),
            self.token
        )).await?;
        Ok(())
    }
}
