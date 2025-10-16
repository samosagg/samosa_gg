pub mod controllers;
pub mod middlewares;
pub mod utils;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{
    config::Config,
    http_server::controllers::{auth, health},
    utils::{aptos_client::AptosClient, database_utils::ArcDbPool, shutdown_utils},
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::{CompressionLayer, CompressionLevel},
    cors::{self, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::{RequestBodyTimeoutLayer, TimeoutLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    servers((url = "/api/v1"))
)]
struct ApiDoc;

pub struct HttpServer {
    pool: ArcDbPool,
    config: Arc<Config>,
    aptos_client: Arc<AptosClient>,
}

impl HttpServer {
    pub fn new(config: Arc<Config>, pool: ArcDbPool, aptos_client: Arc<AptosClient>) -> Self {
        Self {
            pool,
            config,
            aptos_client,
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        tracing::info!("Starting HTTP server...");

        let state = Arc::new(self);

        let listener_address = format!("0.0.0.0:{}", state.config.server_config.port);
        let listener = TcpListener::bind(listener_address).await?;

        axum::serve(
            listener,
            state
                .router()
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(Self::shutdown_signal())
        .await
        .expect("HTTP server crashed");

        tracing::info!("HTTP server completed");
        Ok(())
    }

    fn router(self: &Arc<Self>) -> Router {
        let api_middleware = ServiceBuilder::new()
            .layer(CompressionLayer::new().quality(CompressionLevel::Fastest))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        DefaultMakeSpan::new()
                            .level(Level::INFO)
                            .include_headers(true),
                    )
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .include_headers(true),
                    ),
            )
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(4)))
            .layer(TimeoutLayer::new(Duration::from_secs(5)));

        let governor_config = GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap();

        let governor_limiter = governor_config.limiter().clone();
        let interval = Duration::from_secs(60);

        let cors = CorsLayer::new()
            .allow_headers(cors::Any)
            .allow_methods(cors::Any)
            .allow_origin(cors::Any)
            .expose_headers(cors::Any)
            .max_age(Duration::from_secs(24 * 3600));

        let governor = GovernorLayer::new(governor_config);

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                governor_limiter.retain_recent();
            }
        });

        // let pool = Arc::clone(&self.pool);
        // let jwt_secret = self.config.jwt_config.secret.to_string();

        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .route("/health", get(health::check))
            .nest(
                "/api/v1",
                OpenApiRouter::new()
                    .nest(
                        "/auth",
                        OpenApiRouter::new().route("/connect-wallet", post(auth::connect_wallet)), // Uncomment below when you need Telegram authentication
                                                                                                   // .route("/tg-verify", get(auth::tg_verify))
                                                                                                   // .layer(middleware::from_fn(move |req, next| {
                                                                                                   //     authentication::tg_authentication(
                                                                                                   //         req,
                                                                                                   //         next,
                                                                                                   //         Arc::clone(&pool),
                                                                                                   //         jwt_secret.clone()
                                                                                                   //     )
                                                                                                   // }))
                    )
                    .layer(api_middleware), // Apply here, at the end of /api/v1 nest
            )
            .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
            .layer(RequestBodyLimitLayer::new(8 * 1024 * 1024))
            .layer(cors)
            .layer(governor)
            .with_state(Arc::clone(self))
            .split_for_parts();

        router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api.clone()))
    }

    async fn shutdown_signal() {
        let cancel_token = shutdown_utils::get_shutdown_token();
        cancel_token.cancelled().await;
    }
}
