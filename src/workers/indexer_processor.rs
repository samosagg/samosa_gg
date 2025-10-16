use std::sync::Arc;

use crate::{
    config::Config,
    utils::{
        database_connection::get_db_connection, database_utils::ArcDbPool,
        starting_version::get_starting_version,
    },
};

pub struct IndexerProcessor {
    db_pool: ArcDbPool,
    config: Arc<Config>,
}

impl IndexerProcessor {
    pub fn new(db_pool: ArcDbPool, config: Arc<Config>) -> Self {
        Self { db_pool, config }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let starting_version =
            get_starting_version(&self.config.stream_config, self.db_pool.clone()).await?;
        tracing::info!(
            "Starting events processor at processor version {:?}",
            starting_version
        );

        Ok(())
    }
}
