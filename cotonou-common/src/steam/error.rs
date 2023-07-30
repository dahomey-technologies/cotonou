use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP Error: {0}")]
    HttpError(#[from] crate::http::Error),

    #[error("Steam Error: {0}")]
    SteamError(#[from] crate::steam::SteamError),
}