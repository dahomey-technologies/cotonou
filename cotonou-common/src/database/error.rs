use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database Error")]
    Database,

    #[error("MongoDb Error: {0}")]
    MongoDb(#[from] mongodb::error::Error),
}
