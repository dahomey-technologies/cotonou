use axum::response::{IntoResponse, Response};
use cotonou_common::{database, matchmaking, notifications, profile};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database Error: {0}")]
    Database(#[from] database::Error),
    #[error("Matchmaking Error: {0}")]
    Matchmaking(#[from] matchmaking::Error),
    #[error("Notification Error: {0}")]
    Notification(#[from] notifications::Error),
    #[error("Profile Error: {0}")]
    Profile(#[from] profile::Error),
    #[error("Unauthorized Error")]
    Unauthorized,
    #[error("MissingParameter Error: {0}")]
    MissingParameter(String),
    #[error("InvalidParameter Error: {0}")]
    InvalidParameter(String),
    #[error("Hyper Error: {0}")]
    Hyper(#[from] hyper::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("Error: {self:?}");
        match self {
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Matchmaking(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Notification(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Profile(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::MissingParameter(parameter) => (
                StatusCode::BAD_REQUEST,
                format!("Missing parameter: {}", parameter),
            )
                .into_response(),
            Error::InvalidParameter(parameter) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid parameter: {}", parameter),
            )
                .into_response(),
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
