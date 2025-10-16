pub mod indexer_processor;

use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use tokio_util::task::TaskTracker;

use crate::{
    config::Config,
    utils::{database_utils::ArcDbPool, shutdown_utils},
    workers::indexer_processor::IndexerProcessor,
};

pub struct Worker {
    pub indexer_processor: Arc<IndexerProcessor>,
}

impl Worker {
    pub fn new(config: Arc<Config>, pool: ArcDbPool) -> Self {
        Self {
            indexer_processor: Arc::new(IndexerProcessor::new(
                Arc::clone(&pool),
                Arc::clone(&config),
            )),
        }
    }

    pub async fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        tracing::info!("Worker started");

        let tracker = TaskTracker::new();

        let ip_self = Arc::clone(self);
        tracker.spawn(async move { ip_self.indexer_processor.start().await });

        let cancel_token = shutdown_utils::get_shutdown_token();
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracker.close();
                tracing::info!("Waiting for workers task to finish");
                tracker.wait().await;
                sleep(Duration::from_secs(5)).await;
                tracing::info!("All worker tasks finished");
            }
        }
        Ok(())
    }
}
