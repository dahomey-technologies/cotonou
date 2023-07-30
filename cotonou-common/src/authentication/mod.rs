mod jwt_auth_middleware;
mod jwt_claims;
mod user;

pub use jwt_auth_middleware::*;
pub use jwt_claims::*;
pub use user::*;

use axum::http::{self, HeaderMap};

pub fn get_authorization(headers: &HeaderMap) -> Option<(&str, &str)> {
    let authorization = headers.get(http::header::AUTHORIZATION)?.to_str().ok();
    let (scheme, credentials) = authorization?.split_once(' ')?;
    Some((scheme, credentials))
}