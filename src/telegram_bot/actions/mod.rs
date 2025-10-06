pub mod add_to_group;
pub mod create_trading_account;
pub mod join_existing_clan;
pub mod order_leverage;
pub mod place_order;

use std::{str::FromStr, sync::Arc};

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::{cache::Cache, telegram_bot::TelegramBot};

#[async_trait::async_trait]
pub trait CallbackQueryProcessor {
    async fn process(
        &self,
        cfg: Arc<TelegramBot<Cache>>,
        bot: teloxide::Bot,
        callback_query: teloxide::types::CallbackQuery,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    CreateTradingAccount,
    AddToGroup,
    JoinExistingClan,
    Order {
        market: String,
        order_type: String,
        leverage: u64,
    },
    ConfirmOrder {
        market: String,
        order_type: String,
        leverage: u64,
        amount: BigDecimal,
    },
}

impl ToString for UserAction {
    fn to_string(&self) -> String {
        match self {
            UserAction::CreateTradingAccount => "create_trading_account".to_string(),
            UserAction::AddToGroup => "add_to_group".to_string(),
            UserAction::JoinExistingClan => "join_existing_clan".to_string(),
            UserAction::Order {
                market,
                order_type,
                leverage,
            } => {
                format!("order|{}|{}|{}", market, order_type, leverage)
            }
            UserAction::ConfirmOrder {
                market,
                order_type,
                leverage,
                amount,
            } => {
                format!(
                    "confirm_order|{}|{}|{}|{}",
                    market, order_type, leverage, amount
                )
            }
        }
    }
}

impl FromStr for UserAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        match parts[0] {
            "create" => Ok(UserAction::CreateTradingAccount),
            "add_group" => Ok(UserAction::AddToGroup),
            "join_clan" => Ok(UserAction::JoinExistingClan),
            "order" if parts.len() == 4 => {
                let market = parts[1].to_string();
                let order_type = parts[2].to_string();
                let leverage = parts[3].parse::<u64>().map_err(|_| ())?;
                Ok(UserAction::Order {
                    market,
                    order_type,
                    leverage,
                })
            }
            "confirm_order" if parts.len() == 5 => {
                let market = parts[1].to_string();
                let order_type = parts[2].to_string();
                let leverage = parts[3].parse::<u64>().map_err(|_| ())?;
                let amount = parts[4].parse::<BigDecimal>().map_err(|_| ())?;
                Ok(UserAction::ConfirmOrder {
                    market,
                    order_type,
                    leverage,
                    amount,
                })
            }
            _ => Err(()),
        }
    }
}
