use bigdecimal::BigDecimal;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub market_addr: String,
    pub market_name: String,
    pub sz_decimals: u8,
    pub px_decimals: u8,
    pub max_leverage: u64,
    pub max_open_interest: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetContext {
    pub market: String,
    pub volume_24h: BigDecimal,
    pub funding_index: BigDecimal,
    pub open_interest: BigDecimal,
    pub mark_price: BigDecimal,
    pub mid_price: BigDecimal,
    pub oracle_price: BigDecimal,
    pub previous_day_price: BigDecimal,
    pub price_change_pct_24h: BigDecimal,
    pub price_history: Vec<BigDecimal>,
}

#[async_trait::async_trait]
pub trait ICache: Send + Sync + 'static {
    fn is_healthy(&self) -> bool;

    async fn get_market(&self, market_name: &str) -> Option<Market>;
    async fn get_markets_ilike(&self, market_name: &str) -> Vec<Market>;
    async fn set_markets(&self, markets: Vec<Market>);

    async fn set_asset_contexts(&self, asset_contexts: Vec<AssetContext>);
    async fn get_asset_context(&self, market: &str) -> Option<AssetContext>;
}
pub struct Cache {
    markets: MokaCache<String, Vec<Market>>,
    asset_contexts: MokaCache<String, Vec<AssetContext>>,
}

impl Cache {
    pub fn create_moka_cache<K, V>(max_capacity: u64) -> MokaCache<K, V>
    where
        K: 'static + Send + Sync + Eq + Hash,
        V: 'static + Clone + Send + Sync,
    {
        MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_idle(Duration::from_secs(3600 * 2))
            .time_to_live(Duration::from_secs(3600 * 12))
            .build()
    }

    pub fn default() -> Self {
        let markets = Self::create_moka_cache(500);
        let asset_contexts = Self::create_moka_cache(500);
        Self {
            markets,
            asset_contexts,
        }
    }
}

#[async_trait::async_trait]
impl ICache for Cache {
    fn is_healthy(&self) -> bool {
        true
    }

    async fn get_market(&self, market_name: &str) -> Option<Market> {
        if let Some(markets) = self.markets.get("markets").await {
            markets.into_iter().find(|m| m.market_name == market_name)
        } else {
            None
        }
    }

    async fn get_markets_ilike(&self, market_name: &str) -> Vec<Market> {
        if let Some(markets) = self.markets.get("markets").await {
            markets
                .into_iter()
                .filter(|m| {
                    m.market_name.eq_ignore_ascii_case(market_name)
                        || (market_name.len() < m.market_name.len())
                            && m.market_name
                                .to_lowercase()
                                .starts_with(&format!("{}/", market_name.to_lowercase()))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    async fn set_markets(&self, markets: Vec<Market>) {
        self.markets.insert("markets".to_string(), markets).await;
    }

    async fn set_asset_contexts(&self, asset_contexts: Vec<AssetContext>) {
        self.asset_contexts
            .insert("asset_contexts".to_string(), asset_contexts)
            .await;
    }

    async fn get_asset_context(&self, market: &str) -> Option<AssetContext> {
        if let Some(contexts) = self.asset_contexts.get("asset_contexts").await {
            contexts.into_iter().find(|m| m.market == market)
        } else {
            None
        }
    }
}
