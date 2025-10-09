use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)] 
pub struct AuthResponse {
    pub token: String
}