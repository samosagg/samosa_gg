use axum::{extract::State, http::{StatusCode, Response}};
pub async fn check() -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .body("OK".into())
        .unwrap()
}