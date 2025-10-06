use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use diesel::{
    AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, prelude::Insertable,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{schema::sub_accounts, utils::database_utils::DbPoolConnection};

#[derive(AsChangeset, Debug, Queryable)]
#[diesel(table_name = sub_accounts)]
#[diesel(primary_key(id))]
pub struct SubAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub is_primary: Option<bool>,
}

impl SubAccount {
    pub async fn get_subaccounts_by_user_id(
        user_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Vec<Self>> {
        sub_accounts::table
            .filter(sub_accounts::user_id.eq(user_id))
            .load::<Self>(conn)
            .await
    }
    pub async fn get_primary_subaccount_by_user_id(
        user_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Option<SubAccount>> {
        sub_accounts::table
            .filter(sub_accounts::user_id.eq(user_id))
            .filter(sub_accounts::is_primary.eq(Some(true)))
            .select(sub_accounts::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = sub_accounts)]
#[diesel(primary_key(id))]
pub struct NewSubAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub is_primary: bool,
}

impl NewSubAccount {
    pub fn to_db_subaccount(user_id: Uuid, address: String, is_primary: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            address: standardize_address(&address),
            is_primary,
        }
    }
}
