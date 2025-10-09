pub mod auth;
use serde::Serialize;

use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HttpResponseConstErr {
    pub code: &'static str,
    pub msg: &'static str
}

pub const UNAUTHORIZED_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_401",
    msg: "Unauthorized",
};

pub const BAD_REQUEST_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_400",
    msg: "Bad Request",
};