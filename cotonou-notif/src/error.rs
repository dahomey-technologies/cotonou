use axum::response::{IntoResponse, Response};
use cotonou_common::{notifications::notification_manager, redis::redis_connection_manager};
use hyper::StatusCode;

#[derive(Debug)]
pub enum Error {
    NotifManager(notification_manager::Error),
    Redis(redis_connection_manager::Error),
    Hyper(hyper::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotifManager(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

impl From<notification_manager::Error> for Error {
    fn from(e: notification_manager::Error) -> Self {
        Error::NotifManager(e)
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}

impl From<redis_connection_manager::Error> for Error {
    fn from(e: redis_connection_manager::Error) -> Self {
        Error::Redis(e)
    }
}
