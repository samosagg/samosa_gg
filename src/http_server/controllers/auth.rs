// use axum::{extract::State, http::{Response, StatusCode}, Extension};

// use crate::{db::users::User, http_server::{controllers::InternalState, middlewares::authentication::TelegramClaims, utils::err_handler::response_401_with_message}, models::api::responses::auth::AuthResponse};

// pub const ADMIN_TAG: &str = "admin";
// #[utoipa::path(
//     get,
//     path = "/auth/tg-verify",
//     tag = ADMIN_TAG,
//     responses(
//         (status = 200, description = "Returns auth token", body = [AuthResponse])
//     ),
//     security(
//         ("BearerAuth" = [])
//     )
// )]
// pub async fn tg_verify(
//     State(state): InternalState,
//     Extension(db_user): Extension<User>
// ) -> Response<String> {
    
//     Response::builder()
//         .status(StatusCode::OK)
//         .body("OK".into())
//         .unwrap()
// }