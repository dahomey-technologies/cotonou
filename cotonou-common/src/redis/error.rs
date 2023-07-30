use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redis Error: {0}")]
    Redis(#[from] rustis::Error),
    #[error("BadUriFormat Error: {0}")]
    BadUriFormat(#[from] url::ParseError),
}
