use cotonou_common::matchmaking::matchmaking_command_dal;

#[derive(Debug)]
pub enum Error {
    Database,
    Tokio,
    Redis,
    Notification,
}

impl From<tokio::sync::watch::error::SendError<()>> for Error {
    fn from(_: tokio::sync::watch::error::SendError<()>) -> Self {
        Error::Tokio
    }
}

impl From<tokio::sync::watch::error::RecvError> for Error {
    fn from(_: tokio::sync::watch::error::RecvError) -> Self {
        Error::Tokio
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(_: tokio::task::JoinError) -> Self {
        Error::Tokio
    }
}

impl From<cotonou_common::redis::redis_connection_manager::Error> for Error {
    fn from(_: cotonou_common::redis::redis_connection_manager::Error) -> Self {
        Error::Redis
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

impl From<cotonou_common::notifications::notification_manager::Error> for Error {
    fn from(_: cotonou_common::notifications::notification_manager::Error) -> Self {
        Error::Notification
    }
}