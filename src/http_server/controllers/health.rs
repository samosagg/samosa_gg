use axum::{
    extract::State,
    http::{Response, StatusCode},
};
pub async fn check() -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .body("OK".into())
        .unwrap()
}
