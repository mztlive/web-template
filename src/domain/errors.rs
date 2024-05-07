#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    LogicError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
