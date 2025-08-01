use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("email error: {0}")]
    Email(String),

    #[error("jwt error: {0}")]
    Jwt(String),

    #[error("magic link invalid or expired")]
    InvalidMagicLink,

    #[error("magic link already used")]
    MagicLinkUsed,

    #[error("totp invalid")]
    TotpInvalid,

    #[error("webauthn error: {0}")]
    WebAuthn(String),

    #[error("refresh token invalid")]
    InvalidRefreshToken,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match &self {
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Email(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InvalidMagicLink => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::MagicLinkUsed => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::TotpInvalid => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::WebAuthn(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidRefreshToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
        };
        let body = Json(ErrorBody { error: msg });
        (code, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::BadRequest(format!("{}", e))
    }
}
