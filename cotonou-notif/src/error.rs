use axum::response::{IntoResponse, Response};
use cotonou_common::{notifications, redis};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Notification Error: {0}")]
    Notification(#[from] notifications::Error),
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::Error),
    #[error("Hyper Error: {0}")]
    Hyper(#[from] hyper::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Notification(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
