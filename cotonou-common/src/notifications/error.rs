use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redis Error: {0}")]
    Redis(#[from] rustis::Error),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
}
