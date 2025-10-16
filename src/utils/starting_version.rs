use anyhow::Context;

use crate::{
    config::StreamConfig,
    models::db::processor_status::ProcessorStatusQuery,
    utils::{database_connection::get_db_connection, database_utils::ArcDbPool},
};

pub async fn get_starting_version(
    stream_config: &StreamConfig,
    conn_pool: ArcDbPool,
) -> anyhow::Result<i64> {
    let starting_version_from_config = stream_config.starting_version;
    let latest_processed_version_from_db = get_latest_version_from_db(stream_config, conn_pool)
        .await
        .context("Failed to get latest processor version from DB")?
        .unwrap_or(0);
    Ok(starting_version_from_config.max(latest_processed_version_from_db))
}

pub async fn get_latest_version_from_db(
    stream_config: &StreamConfig,
    conn_pool: ArcDbPool,
) -> anyhow::Result<Option<i64>> {
    let mut conn = get_db_connection(&conn_pool).await?;

    match ProcessorStatusQuery::get_by_processor(&stream_config.request_name_header, &mut conn)
        .await?
    {
        Some(status) => Ok(Some(status.last_success_version)),
        None => Ok(None),
    }
}
