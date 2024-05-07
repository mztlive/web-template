use axum::{response::IntoResponse, Json};

use super::errors::Result;

#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
    pub status: u16,
    #[serde(rename = "errorMessage")]
    pub message: String,
    pub data: Option<T>,
    pub success: bool,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let json = Json(self);
        json.into_response()
    }
}

pub fn api_ok() -> Result<()> {
    Ok(ApiResponse {
        status: 200,
        message: "OK".to_string(),
        data: None,
        success: true,
    })
}

pub fn api_ok_with_data<T>(data: T) -> Result<T> {
    Ok(ApiResponse {
        status: 200,
        message: "OK".to_string(),
        data: Some(data),
        success: true,
    })
}

pub fn api_unauthorized() -> Result<()> {
    Ok(ApiResponse {
        status: 401,
        message: "Unauthorized".to_string(),
        data: None,
        success: false,
    })
}

pub fn api_system_error(message: String) -> Result<()> {
    Ok(ApiResponse {
        status: 500,
        message: format!("System error: {}", message),
        data: None,
        success: false,
    })
}

pub fn api_permission_denied() -> Result<()> {
    Ok(ApiResponse {
        status: 403,
        message: "Permission denied".to_string(),
        data: None,
        success: false,
    })
}
