use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standardized API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    // Predefined error constructors
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message)
    }

    pub fn rate_limited() -> Self {
        Self::new(
            "RATE_LIMITED",
            "Too many requests. Please try again later.",
        )
    }

    pub fn internal_error() -> Self {
        Self::new("INTERNAL_ERROR", "An internal error occurred")
    }

    pub fn invalid_credentials() -> Self {
        Self::new("INVALID_CREDENTIALS", "Invalid credentials provided")
    }

    pub fn expired_token() -> Self {
        Self::new("EXPIRED_TOKEN", "Token has expired")
    }

    pub fn invalid_token() -> Self {
        Self::new("INVALID_TOKEN", "Invalid token provided")
    }

    pub fn magic_link_used() -> Self {
        Self::new("MAGIC_LINK_USED", "This magic link has already been used")
    }

    pub fn magic_link_expired() -> Self {
        Self::new("MAGIC_LINK_EXPIRED", "This magic link has expired")
    }

    pub fn totp_not_enrolled() -> Self {
        Self::new("TOTP_NOT_ENROLLED", "TOTP is not enrolled for this user")
    }

    pub fn invalid_totp() -> Self {
        Self::new("INVALID_TOTP", "Invalid TOTP code provided")
    }

    pub fn user_not_found() -> Self {
        Self::new("USER_NOT_FOUND", "User not found")
    }

    pub fn session_not_found() -> Self {
        Self::new("SESSION_NOT_FOUND", "Session not found")
    }

    pub fn webauthn_error(details: impl Into<String>) -> Self {
        Self::new("WEBAUTHN_ERROR", "WebAuthn operation failed").with_details(details)
    }

    pub fn validation_error(details: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", "Validation failed").with_details(details)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Standardized error response with status code
pub struct ErrorResponse {
    pub status: StatusCode,
    pub error: ApiError,
}

impl ErrorResponse {
    pub fn new(status: StatusCode, error: ApiError) -> Self {
        Self { status, error }
    }

    pub fn bad_request(error: ApiError) -> Self {
        Self::new(StatusCode::BAD_REQUEST, error)
    }

    pub fn unauthorized(error: ApiError) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, error)
    }

    pub fn forbidden(error: ApiError) -> Self {
        Self::new(StatusCode::FORBIDDEN, error)
    }

    pub fn not_found(error: ApiError) -> Self {
        Self::new(StatusCode::NOT_FOUND, error)
    }

    pub fn conflict(error: ApiError) -> Self {
        Self::new(StatusCode::CONFLICT, error)
    }

    pub fn rate_limited(error: ApiError) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, error)
    }

    pub fn internal_error(error: ApiError) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error)
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.error)).into_response()
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ErrorResponse>;
