use aptos_sdk::rest_client::aptos_api_types::{
    EntryFunctionId, MoveModuleId, MoveStructTag, MoveType, ViewRequest,
};
use aptos_sdk::types::account_address::AccountAddress;
use serde_json::json;

pub fn view_fa_balance_request(
    fa_metadata: &str,
    wallet_address: &str,
) -> anyhow::Result<ViewRequest> {
    let entry_function_id = EntryFunctionId {
        module: MoveModuleId {
            address: AccountAddress::ONE.to_string().parse()?,
            name: "primary_fungible_store".parse()?,
        },
        name: "balance".parse()?,
    };
    let request = ViewRequest {
        function: entry_function_id,
        type_arguments: vec![
            MoveType::Struct(MoveStructTag {
                address: AccountAddress::ONE.to_string().parse()?,
                module: "fungible_asset".parse()?,
                name: "Metadata".parse()?,
                generic_type_params: vec![],
            })
        ],
        arguments: vec![json!(wallet_address), json!(fa_metadata)],
    };
    Ok(request)
}
