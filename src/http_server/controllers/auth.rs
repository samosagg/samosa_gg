use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use axum::{extract::State, response::{Response, IntoResponse}, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use uuid::Uuid;

use crate::{http_server::{controllers::InternalState, middlewares::authentication::Claims, utils::err_handler::{response_400_with_const, response_429_with_unhandled_err}}, models::{api::{requests::connect_wallet::ConnectWallet, responses::auth::AuthResponse}, db::users::User}, schema::users, utils::{database_connection::get_db_connection, db_execution::execute_with_better_error}};
use aptos_crypto::{ed25519::*, Signature, ValidCryptoMaterialStringExt};

pub const ADMIN_TAG: &str = "admin";
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

pub const AUTH_TAG: &str = "auth";
#[utoipa::path(
    get,
    path = "/auth/connect-wallet",
    tag = AUTH_TAG,
    responses(
        (status = 200, description = "Returns auth token", body = [AuthResponse])
    )
)]
pub async fn connect_wallet(
    State(state): InternalState,
    Json(req): Json<ConnectWallet>
) -> Response {
    let mut conn = match get_db_connection(&state.pool).await {
        Ok(conn) => conn,
        Err(_) => {
            return response_429_with_unhandled_err(anyhow::anyhow!("Failed to get database connection"))
        }
    };

    let maybe_existing_user = match User::get_by_connected_address(req.address.clone(), &mut conn).await {
        Ok(maybe_existing_user) => maybe_existing_user,
        Err(_) => {
            return response_429_with_unhandled_err(anyhow::anyhow!("Failed to execute get user query"))
        }
    };

    let db_user = if let Some(existing_user) = maybe_existing_user {
        existing_user
    } else {
        // let public_key_bytes = req.public_key.as_bytes();
        // let signature_bytes = req.signature.as_bytes();
        let public_key = match Ed25519PublicKey::from_encoded_string(&req.public_key) {
            Ok(public_key) => public_key,
            Err(_) => {
                return response_429_with_unhandled_err(anyhow::anyhow!("Failed to get public key from bytes"))
            }
        };

        let signature = match Ed25519Signature::from_encoded_string(&req.signature) {
            Ok(signature) => signature,
            Err(_) => {
                return response_429_with_unhandled_err(anyhow::anyhow!("Failed to get signature from bytes"))
            }
        }; 
        if let Err(_) = signature.verify_arbitrary_msg(req.message.as_bytes(), &public_key) {
            return response_429_with_unhandled_err(anyhow::anyhow!("Signature verification failed"))
        };
        let wallet_name: String = format!("apt-{}", &req.address);
        let (wallet_id, address, public_key) = match state.aptos_client.create_new_wallet_on_turnkey(&wallet_name).await {
            Ok((wallet_id, address, public_key)) => (wallet_id, address, public_key),
            Err(_) => {
                return response_429_with_unhandled_err(anyhow::anyhow!("Failed to create wallet"))
            }
        };
        let new_user = User {
            id: Uuid::new_v4(),
            address: standardize_address(&req.address),
            connected_wallet: Some(req.address.clone()),
            public_key: public_key,
            slippage: 20,
            tg_id: None,
            tg_username: None,
            wallet_id
        };
        let query = diesel::insert_into(users::table)  
            .values(new_user.clone())
            .on_conflict(users::connected_wallet)
            .do_nothing();
        match execute_with_better_error(&mut conn, vec![query]).await {
            Ok(_) => {},
            Err(_) => {
                return response_429_with_unhandled_err(anyhow::anyhow!("Failed to execute query"))
            }
        }
        new_user
    };
    let (value_str, unit) = state
        .config
        .jwt_config
        .expires_in
        .as_ref()
        .map(|e| e.split_at(e.len() - 1))
        .unwrap_or(("30", "d"));
    let value: i64 = value_str.parse().unwrap_or(1);
    let duration = match unit {
        "s" => Duration::seconds(value),
        "m" => Duration::minutes(value),
        "h" => Duration::hours(value),
        "d" => Duration::days(value),
        _ => Duration::days(30),
    };

    let claims = Claims {
        id: db_user.id.to_string(),
        exp: (Utc::now().timestamp() + duration.num_seconds()) as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let token_result = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_config.secret.as_ref()),
    );

    return match token_result {
        Ok(token) => Json(AuthResponse{ token }).into_response(),
        Err(_) => {
            response_400_with_const()
        }
    };
}