use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Hyper Error: {0}")]
    HyperError(#[from] hyper::Error),

    #[error("HTTP Error: {0}")]
    HttpError(hyper::StatusCode),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
}