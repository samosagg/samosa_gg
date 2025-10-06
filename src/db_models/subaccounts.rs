use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use diesel::{
    AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, prelude::Insertable,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{schema::subaccounts, utils::database_utils::DbPoolConnection};

#[derive(AsChangeset, Debug, Queryable)]
#[diesel(table_name = subaccounts)]
#[diesel(primary_key(id))]
pub struct SubAccount {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub address: String,
    pub is_primary: Option<bool>,
}

impl SubAccount {
    pub async fn get_subaccounts_by_wallet_id(
        wallet_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Vec<Self>> {
        subaccounts::table
            .filter(subaccounts::wallet_id.eq(wallet_id))
            .load::<Self>(conn)
            .await
    }
    pub async fn get_primary_subaccount_by_wallet_id(
        wallet_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Option<SubAccount>> {
        subaccounts::table
            .filter(subaccounts::wallet_id.eq(wallet_id))
            .filter(subaccounts::is_primary.eq(Some(true)))
            .select(subaccounts::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = subaccounts)]
#[diesel(primary_key(id))]
pub struct NewSubAccount {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub address: String,
    pub is_primary: bool,
}

impl NewSubAccount {
    pub fn to_db_subaccount(wallet_id: Uuid, address: String, is_primary: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            address: standardize_address(&address),
            is_primary,
        }
    }
}
