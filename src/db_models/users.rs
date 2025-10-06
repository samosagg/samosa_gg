use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use diesel::{
    AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, prelude::Insertable,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{schema::users, utils::database_utils::DbPoolConnection};

#[derive(AsChangeset, Debug, Queryable)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct User {
    pub id: Uuid,
    pub wallet_id: String,
    pub wallet_address: String,
    pub wallet_public_key: String,
    pub telegram_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub secondary_wallet_address: Option<String>,
}

impl User {
    pub async fn get_by_telegram_id(
        telegram_id: i64,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Option<Self>> {
        users::table
            .filter(users::telegram_id.eq(Some(telegram_id)))
            .select(users::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }

    pub async fn get_by_secondary_wallet_address(
        secondary_wallet_address: String,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Option<Self>> {
        users::table
            .filter(users::secondary_wallet_address.eq(Some(secondary_wallet_address)))
            .select(users::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct NewTelegramUser {
    pub id: Uuid,
    pub wallet_id: String,
    pub wallet_address: String,
    pub wallet_public_key: String,
    pub telegram_id: i64,
    pub telegram_username: Option<String>,
}

impl NewTelegramUser {
    pub fn to_db_user(
        wallet_id: String,
        wallet_address: String,
        wallet_public_key: String,
        telegram_id: i64,
        telegram_username: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            wallet_address,
            wallet_public_key,
            telegram_id,
            telegram_username,
        }
    }
    pub fn to_db_user_with_custom_uuid(
        id: Uuid,
        wallet_id: String,
        wallet_address: String,
        wallet_public_key: String,
        telegram_id: i64,
        telegram_username: Option<String>,
    ) -> Self {
        Self {
            id,
            wallet_id,
            wallet_address,
            wallet_public_key,
            telegram_id,
            telegram_username,
        }
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct NewSecondaryWalletUser {
    pub id: Uuid,
    pub wallet_id: String,
    pub wallet_address: String,
    pub wallet_public_key: String,
    pub secondary_wallet_address: String,
}

impl NewSecondaryWalletUser {
    pub fn to_db_user(
        wallet_id: String,
        wallet_address: String,
        wallet_public_key: String,
        secondary_wallet_address: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            wallet_address,
            wallet_public_key,
            secondary_wallet_address: standardize_address(&secondary_wallet_address),
        }
    }
}
