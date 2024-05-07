use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use mongodb::Database;
use serde::Deserialize;

use crate::{config::AppState, handles::response::api_system_error};

use super::{
    errors,
    response::{api_permission_denied, api_unauthorized},
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserID(pub String);

#[derive(Debug, Clone)]
pub struct Account(pub String);

pub trait User {
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn account(&self) -> String;
}

pub trait UserRepository {
    async fn find_by_id(
        &self,
        id: &str,
        db: &Database,
    ) -> std::result::Result<Option<Box<dyn User>>, errors::Error>;
}

/// Authorization middleware
pub async fn authorization(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let unauthorized = api_unauthorized().into_response();

    let token = match request.headers().get("Authorization") {
        Some(token) => {
            if let Err(_) = token.to_str() {
                return unauthorized;
            }

            let token = token.to_str().unwrap(); // the unwrap is safe
            token.trim_start_matches("Bearer ")
        }
        None => return unauthorized,
    };

    match state.jwt.verify_token(token) {
        Ok(payload) => {
            // TODO: check if the user is in the database

            request.extensions_mut().insert(UserID(payload.id));
            request.extensions_mut().insert(Account(payload.account));
            next.run(request).await
        }
        Err(_) => return unauthorized,
    }
}

/// Rbac Middleware
pub async fn rbac(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let uname = match request.extensions().get::<Account>() {
        Some(uid) => uid.to_owned(),
        None => return api_unauthorized().into_response(),
    };

    let is_permission = state
        .rbac
        .check_permission(uname.0, request.uri().path().to_string())
        .await;

    match is_permission {
        Ok(is_ok) => {
            if is_ok {
                return next.run(request).await;
            }
        }
        Err(err) => return api_system_error(err.to_string()).into_response(),
    }

    api_permission_denied().into_response()
}
