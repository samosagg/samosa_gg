pub mod accounts;
pub mod add_to_group;
pub mod ask_order_amount;
pub mod balances;
pub mod cancel;
pub mod change_degen_mode;
pub mod change_notification;
pub mod chart;
pub mod confirm_subaccount_deposit;
pub mod create_trading_account;
pub mod deposit_to_subaccount;
pub mod export_pk;
pub mod external_withdraw;
pub mod join_existing_clan;
pub mod limit_order_leverage;
pub mod open_position;
pub mod order_leverage;
pub mod place_limit_order;
pub mod place_order;
pub mod show_pk;
pub mod slippage;
pub mod stats;
pub mod transfer;
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
    OrderLeverage {
        market_name: String,
        is_long: bool,
        leverage: u8,
        balance: f64,
    },
    PlaceOrder {
        market_name: String,
        is_long: bool,
        leverage: u8,
        amount: BigDecimal,
    },
    Cancel,
    LimitOrderLeverage {
        market_name: String,
        price: BigDecimal,
        leverage: u8,
    },
    PlaceLimitOrder {
        market_name: String,
        price: BigDecimal,
        leverage: u8,
        amount: BigDecimal,
        is_long: bool,
    },
    ExportPk,
    ShowPk,
    ChangeNotificationPreferences,
    Slippage,
    UpdateSlippage,
    ChangeDegenMode {
        user_id: Uuid,
        to: bool,
    },
    DepositToSubaccount,
    ConfirmSubaccountDeposit {
        amount: BigDecimal,
    },
    ExternalWithdraw,
}

impl ToString for UserAction {
    fn to_string(&self) -> String {
        match self {
            UserAction::OrderLeverage {
                market_name,
                is_long,
                leverage,
                balance,
            } => format!(
                "order_lev|{}|{}|{}|{}",
                market_name, is_long, leverage, balance
            ),
            UserAction::PlaceOrder {
                market_name,
                is_long,
                leverage,
                amount,
            } => format!("place|{}|{}|{}|{}", market_name, is_long, leverage, amount),
            UserAction::Cancel => "cancel".to_string(),
            UserAction::LimitOrderLeverage {
                market_name,
                price,
                leverage,
            } => format!("limit_leverage|{}|{}|{}", market_name, price, leverage),
            UserAction::PlaceLimitOrder {
                market_name,
                price,
                leverage,
                amount,
                is_long,
            } => format!(
                "limit|{}|{}|{}|{}|{}",
                market_name, price, leverage, amount, is_long
            ),
            UserAction::ExportPk => "export_pk".to_string(),
            UserAction::ShowPk => "show_pk".to_string(),
            UserAction::ChangeNotificationPreferences => "change_notification".to_string(),
            UserAction::Slippage => "slippage".to_string(),
            UserAction::ChangeDegenMode { user_id, to } => {
                format!("change_degen|{}|{}", user_id, to)
            }
            UserAction::UpdateSlippage => "update_slippage".to_string(),
            UserAction::DepositToSubaccount => "dep_to_sub".to_string(),
            UserAction::ConfirmSubaccountDeposit { amount } => {
                format!("confirm_dep_to_sub|{}", amount)
            }
            UserAction::ExternalWithdraw => "external_withdraw".to_string(),
        }
    }
}

impl FromStr for UserAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        match parts[0] {
            "order_lev" if parts.len() == 5 => {
                let market_name = parts[1].to_string();
                let is_long = parts[2].parse::<bool>().map_err(|_| ())?;
                let leverage = parts[3].parse::<u8>().map_err(|_| ())?;
                let balance = parts[4].parse::<f64>().map_err(|_| ())?;
                Ok(UserAction::OrderLeverage {
                    market_name,
                    is_long,
                    leverage,
                    balance,
                })
            }
            "place" if parts.len() == 5 => {
                let market_name = parts[1].to_string();
                let is_long = parts[2].parse::<bool>().map_err(|_| ())?;
                let leverage = parts[3].parse::<u8>().map_err(|_| ())?;
                let amount = BigDecimal::from_str(&parts[4].to_string()).map_err(|_| ())?;
                Ok(UserAction::PlaceOrder {
                    market_name,
                    is_long,
                    leverage,
                    amount,
                })
            }
            "cancel" => Ok(UserAction::Cancel),
            "limit_leverage" if parts.len() == 4 => {
                let market_name = parts[1].to_string();
                let price = BigDecimal::from_str(&parts[2].to_string()).map_err(|_| ())?;
                let leverage = parts[3].parse::<u8>().map_err(|_| ())?;
                Ok(UserAction::LimitOrderLeverage {
                    market_name,
                    price,
                    leverage,
                })
            }
            "limit" if parts.len() == 6 => {
                let market_name = parts[1].to_string();
                let price = BigDecimal::from_str(&parts[2].to_string()).map_err(|_| ())?;
                let leverage = parts[3].parse::<u8>().map_err(|_| ())?;
                let amount = BigDecimal::from_str(&parts[4].to_string()).map_err(|_| ())?;
                let is_long = parts[5].parse::<bool>().map_err(|_| ())?;
                Ok(UserAction::PlaceLimitOrder {
                    market_name,
                    price,
                    leverage,
                    amount,
                    is_long,
                })
            }
            "export_pk" => Ok(UserAction::ExportPk),
            "show_pk" => Ok(UserAction::ShowPk),
            "change_notification" => Ok(UserAction::ChangeNotificationPreferences),
            "slippage" => Ok(UserAction::Slippage),
            "change_degen" if parts.len() == 3 => {
                let user_id = Uuid::parse_str(parts[1]).map_err(|_| ())?;
                let to = parts[2].parse::<bool>().map_err(|_| ())?;
                Ok(UserAction::ChangeDegenMode { user_id, to })
            }
            "update_slippage" => Ok(UserAction::UpdateSlippage),
            "dep_to_sub" => Ok(UserAction::DepositToSubaccount),
            "confirm_dep_to_sub" if parts.len() == 2 => {
                let amount = BigDecimal::from_str(&parts[1].to_string()).map_err(|_| ())?;
                Ok(UserAction::ConfirmSubaccountDeposit { amount })
            }
            "external_withdraw" => Ok(UserAction::ExternalWithdraw),
            _ => Err(()),
        }
    }
}
