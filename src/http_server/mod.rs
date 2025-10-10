pub mod middlewares;
pub mod controllers;
pub mod utils;

use std::{net::SocketAddr, sync::Arc};

use axum::{middleware, routing::get, Router};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config, http_server::{controllers::{auth, health}, middlewares::authentication}, utils::{database_utils::ArcDbPool, shutdown_utils}
};

#[derive(OpenApi)]
#[openapi(
    servers((url = "/api/v1"))
)]
struct ApiDoc;

pub struct HttpServer {
    pool: ArcDbPool,
    config: Arc<Config>,
}

impl HttpServer {
    pub fn new(config: Arc<Config>, pool: ArcDbPool) -> Self {
        Self { pool, config }
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

        let pool = Arc::clone(&self.pool);
        let jwt_secret = self.config.jwt_config.secret.to_string();
        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .route("/health", get(health::check))
            .nest(
                "/api/v1", 
                OpenApiRouter::new()
                    // .nest(
                    //     "/auth", 
                    //     OpenApiRouter::new()
                    //         .route("/tg-verify", get(auth::tg_verify))
                    //         .layer(middleware::from_fn(move |req, next| {
                    //                 authentication::tg_authentication(
                    //                     req, 
                    //                     next,   
                    //                     Arc::clone(&pool), 
                    //                     jwt_secret.clone()
                    //                 )
                    //             })
                    //         ),
                    // )
            )
            .with_state(Arc::clone(self))
            .split_for_parts();
        router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api.clone()))
    }

    async fn shutdown_signal() {
        let cancel_token = shutdown_utils::get_shutdown_token();
        cancel_token.cancelled().await;
    }
}
