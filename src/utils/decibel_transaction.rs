use std::str::FromStr;

use anyhow::Ok;
use aptos_sdk::{
    bcs,
    move_types::{identifier::Identifier, language_storage::ModuleId},
    types::{
        account_address::AccountAddress,
        transaction::{EntryFunction, TransactionPayload},
    },
};

pub fn delegate_trading_to(
    contract_address: &str,
    wallet_address: &str,
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_hex_literal(contract_address)?,
        Identifier::new("dex_accounts")?,
    );
    let to = AccountAddress::from_hex_literal(wallet_address)?;

    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("delegate_trading_to")?,
        vec![],
        vec![bcs::to_bytes(&to)?],
    ));
    Ok(payload)
}

pub fn mint(
    contract_address: &str,
    wallet_address: &str,
    amount: u64,
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_hex_literal(contract_address)?,
        Identifier::new("usdc")?,
    );
    let receiver = AccountAddress::from_hex_literal(wallet_address)?;
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("mint")?,
        vec![],
        vec![bcs::to_bytes(&receiver)?, bcs::to_bytes(&amount)?],
    ));
    Ok(payload)
}

pub fn place_order_to_subaccount(
    contract_addr: &str,
    subaccount_addr: &str,
    market_addr: &str,
    order_value: u64,
    order_size: u64,
    is_long: bool,
    leverage: u64,
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_str(contract_addr)?,
        Identifier::new("dex_accounts")?,
    );
    let args = vec![
        bcs::to_bytes(&AccountAddress::from_str(subaccount_addr)?)?,
        bcs::to_bytes(&AccountAddress::from_str(market_addr)?)?,
        bcs::to_bytes(&1000u64)?,
        bcs::to_bytes(&1000u64)?,
        bcs::to_bytes(&false)?,
        bcs::to_bytes(&2u8)?,
        bcs::to_bytes(&false)?,
        bcs::to_bytes(&None::<String>)?,
        bcs::to_bytes(&None::<u64>)?,
        bcs::to_bytes(&None::<u64>)?,
        bcs::to_bytes(&None::<u64>)?,
        bcs::to_bytes(&None::<u64>)?,
        bcs::to_bytes(&None::<u64>)?,
        bcs::to_bytes(&None::<AccountAddress>)?,
        bcs::to_bytes(&None::<u64>)?,
    ];
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("place_order_to_subaccount")?,
        vec![],
        args,
    ));
    Ok(payload)
}

pub fn deposit_to_subaccount(
    contract_addr: &str,
    subaccount_addr: &str,
    fa_addr: &str,
    amount: u64,
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_str(contract_addr)?,
        Identifier::new("dex_accounts")?,
    );
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("deposit_to_subaccount_at")?,
        vec![],
        vec![
            bcs::to_bytes(&AccountAddress::from_str(subaccount_addr)?)?,
            bcs::to_bytes(&AccountAddress::from_str(fa_addr)?)?,
            bcs::to_bytes(&amount)?,
        ],
    ));
    Ok(payload)
}
