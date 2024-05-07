use std::time::Duration;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
};

use crate::config::AppState;

use super::{login, middlewares};

/// Creates the main application router with all the routes configured.
///
/// This function sets up all the routes for the application, including authentication,
/// organization management, project management, and more. It also configures middleware
/// for timeout and CORS settings.
///
/// # Arguments
///
/// * `app_state` - The application state containing shared resources across routes.
///
/// # Returns
///
/// Returns a `Router` with all the routes and middleware configured.
pub fn create(app_state: AppState) -> Router {
    // build our application with a single route
    let app = Router::new()
        .route("/login", post(login::login))
        // .nest("/", secret_routes(app_state.clone()))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        );
    app
}

fn rbac_routes(state: AppState) -> Router<AppState> {
    Router::new().route_layer(middleware::from_fn_with_state(
        state.clone(),
        middlewares::rbac,
    ))
}

/// Defines secret routes that require authorization.
///
/// These routes are intended for authenticated users to access specific functionalities
/// such as fetching user information, managing tasks, and handling qualifications.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources and configurations.
///
/// # Returns
///
/// Returns a `Router` configured with secret routes.
fn secret_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", rbac_routes(state.clone()))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::authorization,
        ))
}
