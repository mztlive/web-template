use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse, Json};

use crate::{database, domain, jwt};

use super::response::ApiResponse;

// define a error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("要操作的数据不存在")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,

    #[error(transparent)]
    RepositoryError(#[from] database::errors::Error),

    #[error(transparent)]
    LogicError(#[from] domain::errors::Error),

    #[error("other error: {0}")]
    OtherError(String),

    #[error(transparent)]
    ValidationErrors(#[from] validator::ValidationErrors),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt::Error),

    #[error("文件上传错误: {0}")]
    FileUploadError(#[from] MultipartError),

    #[error("文件系统错误: {0}")]
    FileError(#[from] std::io::Error),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] mongodb::error::Error),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::OtherError(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::OtherError(s.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::InternalServerError => 500,
            Error::BadRequest(_) => 400,
            Error::NotFound => 404,
            Error::Unauthorized => 401,
            Error::Forbidden => 403,
            Error::RepositoryError(_) => 500,
            Error::LogicError(_) => 500,
            Error::OtherError(_) => 400,
            Error::ValidationErrors(_) => 400,
            Error::JwtError(_) => 500,
            Error::FileUploadError(_) => 400,
            Error::FileError(_) => 500,
            Error::DatabaseError(_) => 500,
        };

        let body = ApiResponse::<()> {
            status,
            message: self.to_string(),
            data: None,
            success: false,
        };

        (StatusCode::OK, Json(body)).into_response()
    }
}

pub fn internal_server_error() -> Error {
    Error::InternalServerError
}

pub fn data_not_found() -> Error {
    Error::NotFound
}

pub type Result<T> = std::result::Result<ApiResponse<T>, Error>;
