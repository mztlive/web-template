use mongodb::bson::{self, document};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),

    #[error("bson error: {0}")]
    BsonError(#[from] bson::ser::Error),

    #[error("can not read value from document: {0}")]
    AccessValueError(#[from] document::ValueAccessError),

    #[error("optimistic locking error")]
    OptimisticLockingError,
}

pub type Result<T> = std::result::Result<T, Error>;
