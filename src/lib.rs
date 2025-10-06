pub mod cache;
pub mod config;
pub mod db_models;
pub mod http_server;
#[path = "db_migrations/schema.rs"]
pub mod schema;
pub mod telegram_bot;
pub mod utils;

use std::sync::Arc;

use anyhow::Context;
use reqwest::Client;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cache::Cache,
    config::Config,
    http_server::HttpServer,
    telegram_bot::TelegramBot,
    utils::{
        aptos_client::AptosClient, database_connection::new_db_pool, market_indexer::MarketIndexer,
        shutdown_utils,
    },
};

pub async fn init() -> anyhow::Result<(HttpServer, TelegramBot<Cache>)> {
    let config = Arc::new(init_config().context("Failed to initialize configuration")?);
    let pool = new_db_pool(&config.db_config.url, config.db_config.pool_size).await;
    let aptos_client = Arc::new(
        AptosClient::new(Arc::clone(&config))
            .await
            .context("Failed to initialize aptos client")?,
    );
    tokio::spawn(shutdown_utils::poll_for_shutdown_signal());

    let cache = Arc::new(Cache::default());

    init_market(&config.decibel_url, Arc::clone(&cache))
        .await
        .context("Failed to initialize market")?;

    Ok((
        HttpServer::new(Arc::clone(&config), Arc::clone(&pool)),
        TelegramBot::new(
            Arc::clone(&config),
            Arc::clone(&pool),
            Arc::clone(&aptos_client),
            Arc::clone(&cache),
        ),
    ))
}

fn init_config() -> anyhow::Result<Config> {
    let config = Config::load().context("Failed to load configuration")?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("config: {:#?}", config);

    Ok(config)
}

async fn init_market(decibel_url: &str, cache: Arc<Cache>) -> anyhow::Result<()> {
    let client = Client::new();
    let market_indexer = MarketIndexer::new(decibel_url.to_string(), cache);

    market_indexer.fetch_and_store_markets(&client).await?;
    market_indexer
        .fetch_and_store_asset_contexts(&client)
        .await?;
    Ok(())
}
