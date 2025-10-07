use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
pub struct TelegramClaims {
    pub id: String,
    pub exp: usize,
    pub iat: usize,
}
