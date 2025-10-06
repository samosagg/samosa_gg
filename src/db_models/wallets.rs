use diesel::{
    prelude::Insertable, AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{schema::wallets, utils::database_utils::DbPoolConnection};

#[derive(AsChangeset, Debug, Queryable)]
#[diesel(table_name = wallets)]
#[diesel(primary_key(id))]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub wallet_id: String,
    pub address: String,
    pub public_key: String,
    pub is_primary: bool
}

impl Wallet {
    pub async fn get_wallets_by_user_id(
        user_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Vec<Self>> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .load::<Self>(conn)
            .await
    }
    pub async fn get_primary_wallet_by_user_id(
        user_id: Uuid,
        conn: &mut DbPoolConnection<'_>
    ) -> diesel::QueryResult<Option<Self>> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::is_primary.eq(true))
            .select(wallets::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = wallets)]
#[diesel(primary_key(id))]
pub struct NewWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub wallet_id: String,
    pub address: String,
    pub public_key: String
}

impl NewWallet {
    pub fn to_db_wallet(user_id: Uuid, wallet_id: String, address: String, public_key: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            wallet_id,  
            address, 
            public_key
        }
    }
}
