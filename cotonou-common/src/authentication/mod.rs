#[cfg(feature = "authentication")]
use axum::http::{self, HeaderMap};
#[cfg(feature = "authentication")]
pub mod jwt_auth_middleware;
#[cfg(feature = "authentication")]
pub mod jwt_claims;
#[cfg(feature = "authentication")]
pub mod user;

#[cfg(feature = "authentication")]
pub fn get_authorization(headers: &HeaderMap) -> Option<(&str, &str)> {
    let authorization = headers.get(http::header::AUTHORIZATION)?.to_str().ok();
    let (scheme, credentials) = authorization?.split_once(' ')?;
    Some((scheme, credentials))
}