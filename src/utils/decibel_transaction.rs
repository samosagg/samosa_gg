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
    subaccount: &str,
    market: &str,
    price: u64,
    size: u64,
    is_buy: bool,
    time_in_force: u8,
    is_reduce_only: bool,
    client_order_id: Option<String>,
    stop_price: Option<u64>,
    tp_trigger_price: Option<u64>,
    tp_limit_price: Option<u64>,
    sl_trigger_price: Option<u64>,
    sl_limit_price: Option<u64>,
    builder_address: Option<AccountAddress>,
    builder_fees: Option<u64>
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_str(contract_addr)?,
        Identifier::new("dex_accounts")?,
    );
    let args = vec![
        bcs::to_bytes(&AccountAddress::from_str(subaccount)?)?,
        bcs::to_bytes(&AccountAddress::from_str(market)?)?,
        bcs::to_bytes(&price)?,
        bcs::to_bytes(&size)?,
        bcs::to_bytes(&is_buy)?,
        bcs::to_bytes(&time_in_force)?,
        bcs::to_bytes(&is_reduce_only)?,
        bcs::to_bytes(&client_order_id)?,
        bcs::to_bytes(&stop_price)?,
        bcs::to_bytes(&tp_trigger_price)?,
        bcs::to_bytes(&tp_limit_price)?,
        bcs::to_bytes(&sl_trigger_price)?,
        bcs::to_bytes(&sl_limit_price)?,
        bcs::to_bytes(&builder_address)?,
        bcs::to_bytes(&builder_fees)?,
    ];
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("place_order_to_subaccount")?,
        vec![],
        args,
    ));
    Ok(payload)
}

pub fn deposit_to_subaccount_at(
    contract_addr: &str,
    subaccount: &str,
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
            bcs::to_bytes(&AccountAddress::from_str(subaccount)?)?,
            bcs::to_bytes(&AccountAddress::from_str(fa_addr)?)?,
            bcs::to_bytes(&amount)?,
        ],
    ));
    Ok(payload)
}
