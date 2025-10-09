use axum::{extract::Request, http::header, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{db_models::users::User, http_server::utils::err_handler::{response_401_unhandled_err, response_401_with_const, response_401_with_message}, utils::{database_connection::get_db_connection, database_utils::ArcDbPool}};
#[derive(Serialize, Deserialize, Clone)]
pub struct TelegramClaims {
    pub id: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn tg_authentication(
    mut req: Request,
    next: Next,
    pool: ArcDbPool,
    jwt_secret: String 
) -> Result<Response, Response> {
    let auth_header = req.headers_mut().get(header::AUTHORIZATION);
    let token = auth_header
        .and_then(|e| e.to_str().ok())
        .filter(|e| e.starts_with("Bearer"))
        .ok_or(response_401_with_const())?
        .split_whitespace()
        .last()
        .ok_or(response_401_with_const())?;

    let token_data = decode::<TelegramClaims>(
        token, 
        &DecodingKey::from_secret(jwt_secret.as_ref()), 
        &Validation::default()
    ).map_err(|e| response_401_unhandled_err(e))?;

    let mut conn = get_db_connection(&pool).await.ok().ok_or(response_401_with_message("Failed to get db pool"))?;
    let telegram_id = token_data.claims.id.parse::<i64>().ok().ok_or(response_401_with_message("Failed to parse telegram id"))?;
    let db_user = User::get_by_telegram_id(telegram_id, &mut conn).await.map_err(|_| response_401_with_message("User not found"))?.ok_or(response_401_with_message("User not found"))?;

    req.extensions_mut().insert(db_user);
    Ok(next.run(req).await)
}