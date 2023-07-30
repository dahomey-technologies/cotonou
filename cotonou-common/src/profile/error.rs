use crate::database;
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TryFromInt Error: {0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("Database Error: {0}")]
    Database(#[from] database::Error),
}
