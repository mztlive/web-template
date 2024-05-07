use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthRequest {
    pub account: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(rename = "userid")] // 为了配合antd pro
    pub user_id: String,
    pub name: String,
    pub avatar: String,
}
