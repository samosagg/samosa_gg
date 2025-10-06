use diesel::{
    AsChangeset, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, prelude::Insertable,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{schema::settings, utils::database_utils::DbPoolConnection};

#[derive(AsChangeset, Debug, Queryable)]
#[diesel(table_name = settings)]
#[diesel(primary_key(id))]
pub struct Setting {
    pub id: Uuid,
    pub user_id: Uuid,
    pub degen_mode: bool,
    pub notifications: bool,
}

impl Setting {
    pub async fn get_setting_by_user_id(
        user_id: Uuid,
        conn: &mut DbPoolConnection<'_>,
    ) -> diesel::QueryResult<Option<Setting>> {
        settings::table
            .filter(settings::user_id.eq(user_id))
            .select(settings::all_columns)
            .first::<Self>(conn)
            .await
            .optional()
    }
}

#[derive(AsChangeset, Debug, Insertable)]
#[diesel(table_name = settings)]
#[diesel(primary_key(id))]
pub struct NewSetting {
    pub id: Uuid,
    pub user_id: Uuid,
    pub degen_mode: bool,
    pub notifications: bool,
}

impl NewSetting {
    pub fn to_db_setting(user_id: Uuid, degen_mode: bool, notifications: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            degen_mode,
            notifications
        }
    }
}
