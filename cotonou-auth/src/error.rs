use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cotonou_common::{account_manager, core_profile_manager};

#[derive(Debug)]
pub enum Error {
    InvalidScheme,
    NoAuthorizeHeader,
    Unauthorized,
    InvalidExpirationTime,
    CannotEncodeJwt,
    Database,
    Hyper(hyper::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unauthorized")
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("Error: {self:?}");
        match self {
            Error::InvalidScheme => StatusCode::UNAUTHORIZED,
            Error::NoAuthorizeHeader => StatusCode::UNAUTHORIZED,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::InvalidExpirationTime => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CannotEncodeJwt => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
