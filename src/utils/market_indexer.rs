use reqwest::Client;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    cache::{AssetContext, ICache, Market},
    utils::shutdown_utils,
};

pub struct MarketIndexer<TCache: ICache> {
    decibel_url: String,
    cache: Arc<TCache>,
}

impl<TCache: ICache> MarketIndexer<TCache>
where
    TCache: ICache + Send + Sync + 'static,
{
    pub fn new(decibel_url: String, cache: Arc<TCache>) -> Self {
        Self { decibel_url, cache }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let client = Client::new();

        let cancel_token = shutdown_utils::get_shutdown_token();

        tokio::select! {
            _ = async {
                loop {
                    if cancel_token.is_cancelled() {
                        break;
                    }

                    if let Err(e) = self.fetch_and_store_markets(&client).await {
                        tracing::error!("Failed to fetch and store markets: {e:#}");
                    }
                    sleep(Duration::from_secs(5 * 60)).await;
                }
            } => {},
            _ = cancel_token.cancelled() => {
                tracing::info!("Market indexer worker finished");
            }
        }
        Ok(())
    }

    pub async fn fetch_and_store_markets(&self, client: &Client) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/markets", self.decibel_url);
        let markets = client.get(url).send().await?.json::<Vec<Market>>().await?;

        self.cache.set_markets(markets).await;
        Ok(())
    }

    pub async fn fetch_and_store_asset_contexts(&self, client: &Client) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/asset_contexts", self.decibel_url);
        let asset_contexts = client
            .get(url)
            .send()
            .await?
            .json::<Vec<AssetContext>>()
            .await?;

        self.cache.set_asset_contexts(asset_contexts).await;
        Ok(())
    }
}
