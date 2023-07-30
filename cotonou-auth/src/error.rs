use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cotonou_common::{account_manager, core_profile_manager, steam};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("InvalidScheme error")]
    InvalidScheme,
    #[error("InvalidScheme NoAuthorizeHeader")]
    NoAuthorizeHeader,
    #[error("InvalidScheme Unauthorized")]
    Unauthorized,
    #[error("InvalidScheme Forbidden")]
    Forbidden,
    #[error("InvalidScheme InvalidExpirationTime")]
    InvalidExpirationTime,
    #[error("InvalidScheme CannotEncodeJwt")]
    CannotEncodeJwt,
    #[error("InvalidScheme Database")]
    Database,
    #[error("Hyper Error: {0}")]
    Hyper(hyper::Error),
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
            Error::CannotEncodeJwt => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Steam(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

impl From<account_manager::Error> for Error {
    fn from(error: account_manager::Error) -> Self {
        match error {
            account_manager::Error::Database => Error::Database,
        }
    }
}

impl From<core_profile_manager::Error> for Error {
    fn from(error: core_profile_manager::Error) -> Self {
        match error {
            core_profile_manager::Error::Database => Error::Database,
        }
    }
}

impl From<cotonou_common::generic_dal::Error> for Error {
    fn from(error: cotonou_common::generic_dal::Error) -> Self {
        match error {
            cotonou_common::generic_dal::Error::Database => Error::Database,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_error: jsonwebtoken::errors::Error) -> Self {
        Error::CannotEncodeJwt
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}
