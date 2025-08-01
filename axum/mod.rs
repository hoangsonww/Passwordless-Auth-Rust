pub mod handlers;
pub mod middleware;
pub mod extractors;
pub mod errors;

use crate::{config::Config, db::Database, email::Emailer, webauthn::WebauthnState};
use axum::{routing::{get, post}, Router};
use std::sync::Arc;

/// Shared application state for handlers.
#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub db: Arc<Database>,
    pub emailer: Arc<Emailer>,
    pub webauthn: Arc<WebauthnState>,
}

pub type SharedState = Arc<AppState>;

/// Constructs the main router with all routes and middleware applied.
pub fn create_router(state: AppState) -> Router {
    let shared = Arc::new(state);
    Router::new()
        .route("/request/magic", post(handlers::request_magic))
        .route("/verify/magic", get(handlers::verify_magic))
        .route("/totp/enroll", post(handlers::totp_enroll))
        .route("/totp/verify", post(handlers::totp_verify))
        .route("/token/refresh", post(handlers::refresh_token))
        .route(
            "/webauthn/register/options",
            post(handlers::webauthn_register_options),
        )
        .route(
            "/webauthn/register/complete",
            post(handlers::webauthn_register_complete),
        )
        .route(
            "/webauthn/login/options",
            post(handlers::webauthn_login_options),
        )
        .route(
            "/webauthn/login/complete",
            post(handlers::webauthn_login_complete),
        )
        .with_state(shared)
        .layer(middleware::common_layers())
}
