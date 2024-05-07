use axum::{extract::State, Json};

use crate::{
    config::AppState, database::repositories::user::UserRepository,
    handles::response::api_ok_with_data,
};

use super::super::errors::{Error, Result};

use super::types::{AuthRequest, AuthResponse};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<AuthRequest>,
) -> Result<AuthResponse> {
    let repository = UserRepository::new();
    let user = repository
        .find_by_account(&request.account, &state.db)
        .await?;

    if let Some(user) = user {
        if user.secret.is_match(&request.password) {
            let token = state.jwt.create_token(user)?;
            return api_ok_with_data(AuthResponse { token });
        }
    }

    Err(Error::BadRequest("用户名或密码错误".to_string()))
}
