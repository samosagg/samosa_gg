#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (http_server, telegram_bot) = pace_api::init().await.expect("Failed to initialize server");
    tokio::spawn(async move {
        if let Err(e) = telegram_bot.start().await {
            tracing::error!("Bot crashed: {:?}", e);
        }
    });

    http_server.start().await?;

    tracing::info!("Http server exited. Shutting down");
    Ok(())
}
