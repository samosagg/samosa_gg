use std::sync::Arc;

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::bb8::{Pool, PooledConnection},
};

pub type DbPool = Pool<AsyncPgConnection>;
pub type ArcDbPool = Arc<DbPool>;
pub type DbPoolConnection<'a> = PooledConnection<'a, AsyncPgConnection>;
