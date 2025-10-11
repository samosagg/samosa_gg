use diesel::{
    prelude::Insertable, AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{ schema::users, utils::database_utils::DbPoolConnection };

#[derive(AsChangeset, Debug, Queryable, Clone, Insertable)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct User {
    pub id: Uuid,
    pub tg_id: Option<i64>,
    pub tg_username: Option<String>,
    pub connected_wallet: Option<String>,
    pub address: String,
    pub public_key: String,
    pub wallet_id: String,
    pub slippage: i64
}

impl User {
    pub async fn get_by_telegram_id(
        tg_id: i64,
        conn: &mut DbPoolConnection<'_>
    ) -> diesel::QueryResult<Option<Self>> {
        users::table
            .filter(users::tg_id.eq(Some(tg_id)))
            .select(users::all_columns)
            .first::<Self>(conn).await
            .optional()
    }

    pub async fn get_by_telegram_username(
        tg_username: String,
        conn: &mut DbPoolConnection<'_>
    ) -> diesel::QueryResult<Option<Self>> {
        users::table
            .filter(users::tg_username.eq(Some(tg_username)))
            .select(users::all_columns)
            .first::<Self>(conn).await
            .optional()
    }

    pub async fn get_by_connected_address(
        address: String,
        conn: &mut DbPoolConnection<'_>
    ) -> diesel::QueryResult<Option<Self>> {
        users::table
            .filter(users::connected_wallet.eq(Some(address)))
            .select(users::all_columns)
            .first::<Self>(conn).await
            .optional()
    }

    pub fn to_db_tg_user(
        tg_id: i64,
        tg_username: Option<String>,
        address: String,
        public_key: String,
        wallet_id: String 
    ) -> Self {
        Self { 
            id: Uuid::new_v4(), 
            tg_id: Some(tg_id), 
            tg_username, 
            connected_wallet: None, 
            address, 
            public_key, 
            wallet_id ,
            slippage: 20
        }
    }
}

