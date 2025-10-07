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
    pub telegram_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub secondary_wallet_address: Option<String>,
    pub degen_mode: bool,
    pub slippage: i32,
    pub token: String,
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
    pub telegram_id: i64,
    pub telegram_username: Option<String>,
    pub degen_mode: bool,
}

impl NewTelegramUser {
    pub fn to_db_user(telegram_id: i64, telegram_username: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            telegram_id,
            telegram_username,
            degen_mode: false,
        }
    }
    pub fn to_db_user_with_custom_uuid(
        id: Uuid,
        telegram_id: i64,
        telegram_username: Option<String>,
    ) -> Self {
        Self {
            id,
            telegram_id,
            telegram_username,
            degen_mode: false,
        }
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct NewSecondaryWalletUser {
    pub id: Uuid,
    pub secondary_wallet_address: String,
    pub degen_mode: bool,
}

impl NewSecondaryWalletUser {
    pub fn to_db_user(secondary_wallet_address: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            secondary_wallet_address: standardize_address(&secondary_wallet_address),
            degen_mode: false,
        }
    }
}
