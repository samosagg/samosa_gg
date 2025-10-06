use std::sync::LazyLock;
use tokio::signal;
use tokio_util::sync::CancellationToken;

static SHUTDOWN_TOKEN: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);

pub fn get_shutdown_token() -> CancellationToken {
    SHUTDOWN_TOKEN.clone()
}

pub async fn poll_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            SHUTDOWN_TOKEN.cancel();
        },
        _ = terminate => {
            SHUTDOWN_TOKEN.cancel();
        },
    }

    tracing::info!("Signal received, starting graceful shutdown");
}
