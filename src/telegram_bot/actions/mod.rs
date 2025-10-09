pub mod accounts;
pub mod add_to_group;
pub mod ask_order_amount;
pub mod balances;
pub mod change_degen_mode;
pub mod close;
pub mod create_trading_account;
pub mod deposit_to_subaccount;
pub mod export_pk;
pub mod join_existing_clan;
pub mod order_leverage;
pub mod place_order;
pub mod slippage;
pub mod stats;
pub mod transfer;
pub mod update_slippage;
pub mod withdraw;
pub mod chart;
pub mod open_position;

use std::{str::FromStr, sync::Arc};

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
    // Chart action on button click
    Chart { 
        market_name: String,
        interval: String
    },
    // On click of long/short button
    OpenPosition {
        is_long: bool,
        market_name: String
    },
    CreateTradingAccount,
    AddToGroup,
    JoinExistingClan,
    // Order {
    //     market: String,
    //     order_type: String,
    //     leverage: u64,
    // },
    // ConfirmOrder {
    //     market: String,
    //     order_type: String,
    //     leverage: u64,
    //     amount: BigDecimal,
    // },
    Stats,
    Accounts {
        user_id: Uuid,
        token: String,
    },
    Withdraw {
        user_id: Uuid,
        token: String,
    },
    Balances {
        user_id: Uuid,
    },
    Transfer {
        user_id: Uuid,
    },
    ExportPk,
    Slippage,
    ChangeDegenMode {
        change_to: bool,
        user_id: Uuid,
        token: String,
    },
    Close,
    UpdateSlippage,
    DepositToSubAccount {
        subaccount_id: Option<Uuid>,
    },
    MarketOrder {
        is_long: bool,
        market_name: String,
        leverage: u8,
    },
}

impl ToString for UserAction {
    fn to_string(&self) -> String {
        match self {
            UserAction::Chart { market_name, interval } => format!("chart|{}|{}", market_name, interval),
            UserAction::OpenPosition { is_long, market_name } => format!("op|{}|{}", is_long, market_name),
            UserAction::CreateTradingAccount => "create_trading_account".to_string(),
            UserAction::AddToGroup => "add_to_group".to_string(),
            UserAction::JoinExistingClan => "join_existing_clan".to_string(),
            UserAction::ChangeDegenMode {
                change_to,
                user_id,
                token,
            } => {
                format!("degen_mode|{}|{}|{}", change_to, user_id, token)
            }
            UserAction::Stats => "stats".to_string(),
            UserAction::ExportPk => "export_pk".to_string(),
            UserAction::Accounts { user_id, token } => format!("accounts|{}|{}", user_id, token),
            UserAction::Slippage => "slippage".to_string(),
            UserAction::Withdraw { user_id, token } => format!("withdraw|{}|{}", user_id, token),
            UserAction::Balances { user_id } => format!("balances|{}", user_id),
            UserAction::Transfer { user_id } => format!("transfer|{}", user_id),
            UserAction::Close => "close".to_string(),
            UserAction::UpdateSlippage => "update_slippage".to_string(),
            UserAction::DepositToSubAccount { subaccount_id } => format!(
                "deposit_to_subaccount|{}",
                subaccount_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "".to_string())
            ),
            UserAction::MarketOrder {
                is_long,
                market_name,
                leverage,
            } => format!("mo|{}|{}|{}", is_long, market_name, leverage),
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
            "degen_mode" if parts.len() == 4 => {
                let change_to = parts[1].parse::<bool>().map_err(|_| ())?;
                let user_id = Uuid::parse_str(parts[2]).map_err(|_| ())?;
                let token = parts[3].to_string();

                Ok(UserAction::ChangeDegenMode {
                    change_to,
                    user_id,
                    token,
                })
            }
            "accounts" if parts.len() == 3 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                let token = parts[2].to_string();
                Ok(UserAction::Accounts { user_id, token })
            }
            "withdraw" if parts.len() == 3 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                let token = parts[2].to_string();
                Ok(UserAction::Withdraw { user_id, token })
            }
            "balances" if parts.len() == 2 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                Ok(UserAction::Balances { user_id })
            }
            "transfer" if parts.len() == 2 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                Ok(UserAction::Transfer { user_id })
            }
            "update_slippage" => Ok(UserAction::UpdateSlippage),
            "close" => Ok(UserAction::Close),
            "deposit_to_subaccount" if parts.len() == 2 => {
                let subaccount_id = if parts[1].is_empty() {
                    None
                } else {
                    Some(Uuid::parse_str(parts[1]).map_err(|_| ())?)
                };
                Ok(UserAction::DepositToSubAccount { subaccount_id })
            }
            "mo" if parts.len() == 4 => {
                let is_long = parts[1].parse::<bool>().map_err(|_| ())?;
                let market_name = parts[2].to_string();
                let leverage = parts[3].parse::<u8>().map_err(|_| ())?;
                Ok(UserAction::MarketOrder {
                    is_long,
                    market_name,
                    leverage,
                })
            },
            "chart" if parts.len() == 3 => {
                let market_name = parts[1].to_string();
                let interval = parts[2].to_string();
                Ok(UserAction::Chart { market_name, interval })
            },
            "op" if parts.len() == 3 => {
                let is_long = parts[1].parse::<bool>().map_err(|_| ())?;
                let market_name = parts[2].to_string();
                Ok(UserAction::OpenPosition { is_long, market_name })
            },
            _ => Err(()),
        }
    }
}
