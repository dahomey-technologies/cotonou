use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redis Error")]
    Redis(#[from] rustis::Error),

    #[error("Json Error")]
    Json(#[from] serde_json::Error),
}
