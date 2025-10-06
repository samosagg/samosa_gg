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
    contract_address: &str,
    subaccount: &str,
    market_address: &str,
    order_value: u64,
    order_size: u64,
    is_long: bool,
    leverage: u64,
) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::from_hex_literal(contract_address)?,
        Identifier::new("dex_accounts")?,
    );
    let args = vec![
        bcs::to_bytes(
            &AccountAddress::from_hex_literal(subaccount)?
        )?,
        bcs::to_bytes(
            &AccountAddress::from_hex_literal(market_address)?
        )?,
        bcs::to_bytes(&order_value)?,
        bcs::to_bytes(&order_size)?,
        bcs::to_bytes(&is_long)?,
        bcs::to_bytes(&leverage)?,
        bcs::to_bytes(&false)?,
        bcs::to_bytes(&None::<u64>)?,
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

