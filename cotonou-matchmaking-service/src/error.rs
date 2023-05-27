use axum::response::{IntoResponse, Response};
use cotonou_common::{matchmaking::matchmaking_command_dal, notifications::notification_manager};
use hyper::StatusCode;

use crate::profile_for_matchmaking_manager;

#[derive(Debug)]
pub enum Error {
    Database,
    Notification,
    Unauthorized,
    MissingParameter(String),
    InvalidParameter(String),
    Hyper(hyper::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("Error: {self:?}");
        match self {
            Error::Database => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Notification => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::MissingParameter(parameter) => (StatusCode::BAD_REQUEST, format!("Missing parameter: {}", parameter)).into_response(),
            Error::InvalidParameter(parameter) => (StatusCode::BAD_REQUEST, format!("Invalid parameter: {}", parameter)).into_response(),
            Error::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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

impl From<profile_for_matchmaking_manager::Error> for Error {
    fn from(_: profile_for_matchmaking_manager::Error) -> Self {
        Error::Database
    }
}

impl From<matchmaking_command_dal::Error> for Error {
    fn from(_: matchmaking_command_dal::Error) -> Self {
        Error::Database
    }
}

impl From<cotonou_common::game_server_dal::Error> for Error {
    fn from(_: cotonou_common::game_server_dal::Error) -> Self {
        Error::Database
    }
}

impl From<cotonou_common::matchmaking_ticket_dal::Error> for Error {
    fn from(_: cotonou_common::matchmaking_ticket_dal::Error) -> Self {
        Error::Database
    }
}

impl From<cotonou_common::matchmaking_session_dal::Error> for Error {
    fn from(_: cotonou_common::matchmaking_session_dal::Error) -> Self {
        Error::Database
    }
}

impl From<cotonou_common::matchmaking_average_waiting_time_dal::Error> for Error {
    fn from(_: cotonou_common::matchmaking_average_waiting_time_dal::Error) -> Self {
        Error::Database
    }
}

impl From<notification_manager::Error> for Error {
    fn from(_: notification_manager::Error) -> Self {
        Error::Notification
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}