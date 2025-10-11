use axum::{http::StatusCode, Json, response::{IntoResponse, Response}};
use jsonwebtoken::errors::Error;
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::api::responses::{BAD_REQUEST_ERR, UNAUTHORIZED_ERR};

pub fn response_400_with_const() -> Response {
    (StatusCode::BAD_REQUEST, Json(BAD_REQUEST_ERR)).into_response()
}
pub fn response_401_with_const() -> Response {
    (StatusCode::UNAUTHORIZED, Json(UNAUTHORIZED_ERR)).into_response()
}
pub fn response_401_unhandled_err(e: Error) -> Response {
    let error = HttpResponseErr::new("ERR_401", &e.to_string());

    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

pub fn response_401_with_message(msg: &str) -> Response {
    let error = HttpResponseErr::new("ERR_401", msg);

    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

pub fn response_429_with_unhandled_err(e: anyhow::Error) -> Response {
    let error = HttpResponseErr::new("ERR_429", &e.to_string());

    (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response()
}
#[derive(Serialize, ToSchema)]
pub struct HttpResponseErr {
    pub code: String,
    pub msg: String,
}

impl HttpResponseErr {
    pub fn new(code: &str, msg: &str) -> Self {
        Self {
            code: code.to_string(),
            msg: msg.to_string(),
        }
    }
}
