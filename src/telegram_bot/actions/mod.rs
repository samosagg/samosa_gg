pub mod add_to_group;
pub mod create_trading_account;
pub mod join_existing_clan;
pub mod order_leverage;
pub mod place_order;
pub mod change_degen_mode;
pub mod export_pk;
pub mod accounts;
pub mod slippage;
pub mod stats;
pub mod balances;
pub mod withdraw;
pub mod transfer;
pub mod close;
pub mod update_slippage;

use std::{str::FromStr, sync::Arc};

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    Stats,
    Accounts { 
        user_id: Uuid,
        token: String
    },
    Withdraw { 
        user_id: Uuid,
        token: String
    },
    Balances { 
        user_id: Uuid 
    },
    Transfer { 
        user_id: Uuid
    },
    ExportPk,
    Slippage,
    ChangeDegenMode {
        change_to: bool,
        user_id: Uuid,
        token: String
    },
    Close,
    UpdateSlippage,
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
            },
            UserAction::ChangeDegenMode { change_to, user_id, token } => {
                format!(
                    "degen_mode|{}|{}|{}",
                    change_to,
                    user_id,
                    token
                )
            },
            UserAction::Stats => "stats".to_string(),
            UserAction::ExportPk => "export_pk".to_string(),
            UserAction::Accounts { user_id, token } => format!("accounts|{}|{}", user_id, token),
            UserAction::Slippage => "slippage".to_string(),
            UserAction::Withdraw { user_id, token } => format!("withdraw|{}|{}", user_id, token),
            UserAction::Balances { user_id } => format!("balances|{}", user_id),
            UserAction::Transfer { user_id } => format!("transfer|{}", user_id),
            UserAction::Close => "close".to_string(),
            UserAction::UpdateSlippage => "update_slippage".to_string()
        }
    }
}

impl FromStr for UserAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        match parts[0] {
            "create_trading_account" => Ok(UserAction::CreateTradingAccount),
            "add_group" => Ok(UserAction::AddToGroup),
            "join_clan" => Ok(UserAction::JoinExistingClan),
            "export_pk" => Ok(UserAction::ExportPk),
            "slippage" => Ok(UserAction::Slippage),
            "stats" => Ok(UserAction::Stats),
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
            "degen_mode" if parts.len() == 4 => {
                let change_to = parts[1].parse::<bool>().map_err(|_| ())?;
                let user_id = Uuid::parse_str(parts[2]).map_err(|_| ())?;
                let token = parts[3].to_string();

                Ok(UserAction::ChangeDegenMode { change_to, user_id, token })
            },
            "accounts" if parts.len() == 3  => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                let token = parts[2].to_string();
                Ok(UserAction::Accounts { user_id, token })
            },
            "withdraw" if parts.len() == 3 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                let token = parts[2].to_string();
                Ok(UserAction::Withdraw { user_id, token })
            },
            "balances" if parts.len() == 2 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                Ok(UserAction::Balances { user_id })
            },
            "transfer" if parts.len() == 2 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                Ok(UserAction::Transfer { user_id })
            },
            "update_slippage" => Ok(UserAction::UpdateSlippage),
            "close" => Ok(UserAction::Close),
            _ => Err(()),
        }
    }
}
