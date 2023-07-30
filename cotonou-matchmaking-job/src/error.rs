use cotonou_common::{database, matchmaking, notifications, redis};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database Error: {0}")]
    Database(#[from] database::Error),
    #[error("Matchmaking Error: {0}")]
    Matchmaking(#[from] matchmaking::Error),
    #[error("Notification Error: {0}")]
    Notification(#[from] notifications::Error),
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::Error),
    #[error("SendError Error: {0}")]
    SendError(#[from] tokio::sync::watch::error::SendError<()>),
    #[error("RecvError Error: {0}")]
    RecvError(#[from] tokio::sync::watch::error::RecvError),
    #[error("JoinError Error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
