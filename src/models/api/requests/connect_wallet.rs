use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ConnectWallet{
    pub message: String,
    pub public_key: String,
    pub signature: String,
    pub address: String, 
}