use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cotonou_common::{database, profile, steam};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("InvalidScheme Error")]
    InvalidScheme,
    #[error("NoAuthorizeHeader Error")]
    NoAuthorizeHeader,
    #[error("Unauthorized Error")]
    Unauthorized,
    #[error("Forbidden Error")]
    Forbidden,
    #[error("InvalidExpirationTime Error")]
    InvalidExpirationTime,
    #[error("CannotEncodeJwt Error")]
    CannotEncodeJwt(#[from] jsonwebtoken::errors::Error),
    #[error("Database Error: {0}")]
    Database(#[from] database::Error),
    #[error("Profile Error: {0}")]
    Profile(#[from] profile::Error),
    #[error("Hyper Error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("Steam Error: {0}")]
    Steam(#[from] steam::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("Error: {self:?}");
        match self {
            Error::InvalidScheme => StatusCode::UNAUTHORIZED,
            Error::NoAuthorizeHeader => StatusCode::UNAUTHORIZED,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::InvalidExpirationTime => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CannotEncodeJwt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Profile(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Steam(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
