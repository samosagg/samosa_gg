use anyhow::Ok;
use aptos_sdk::{
    bcs,
    move_types::{identifier::Identifier, language_storage::ModuleId},
    types::{
        account_address::AccountAddress,
        transaction::{EntryFunction, TransactionPayload},
    },
};

pub fn transfer_fa(fa: &str, to: &str, amount: u64) -> anyhow::Result<TransactionPayload> {
    let module = ModuleId::new(
        AccountAddress::ONE,
        Identifier::new("primary_fungible_store")?,
    );
    let to = AccountAddress::from_hex_literal(to)?;
    let fa = AccountAddress::from_hex_literal(fa)?;
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        module,
        Identifier::new("transfer")?,
        vec![],
        vec![
            bcs::to_bytes(&fa)?,
            bcs::to_bytes(&to)?,
            bcs::to_bytes(&amount)?,
        ],
    ));
    Ok(payload)
}
