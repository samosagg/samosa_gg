use std::sync::Arc;

use axum::extract::State;

use crate::http_server::HttpServer;

pub mod health;
pub mod auth;

type InternalState = State<Arc<HttpServer>>;
