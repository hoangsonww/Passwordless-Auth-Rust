use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{
    audit::AuditLogger,
    db::Database,
    error::{ApiError, ErrorResponse},
    session::Session,
};
use tracing::error;

#[derive(Clone)]
pub struct AdminState {
    pub db: Arc<Database>,
    pub audit: Arc<AuditLogger>,
}

/// User information response
#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub totp_enabled: bool,
    pub webauthn_credentials_count: i32,
    pub created_at: String,
}

/// Session information response
#[derive(Serialize)]
pub struct SessionInfo {
    pub token: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub revoked: bool,
}

/// Pagination query parameters
#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_offset")]
    pub offset: i32,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_offset() -> i32 {
    0
}

fn default_limit() -> i32 {
    50
}

/// List all users with pagination
pub async fn list_users(
    State(state): State<AdminState>,
    Query(params): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut stmt = state.db.conn
        .prepare(
            "SELECT id, email, totp_secret, created_at FROM users ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )
        .map_err(|e| {
            error!("Database error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    let users = stmt
        .query_map(rusqlite::params![params.limit, params.offset], |row| {
            let id: String = row.get(0)?;
            let email: String = row.get(1)?;
            let totp_secret: Option<String> = row.get(2)?;
            let created_at: String = row.get(3)?;

            // Count WebAuthn credentials
            let cred_count: i32 = row.get(0).unwrap_or(0);

            Ok(UserInfo {
                id,
                email,
                totp_enabled: totp_secret.is_some(),
                webauthn_credentials_count: cred_count,
                created_at,
            })
        })
        .map_err(|e| {
            error!("Query error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Row mapping error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    Ok(Json(users))
}

/// Get user by ID
pub async fn get_user(
    State(state): State<AdminState>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut stmt = state.db.conn
        .prepare("SELECT id, email, totp_secret, created_at FROM users WHERE id = ?1")
        .map_err(|e| {
            error!("Database error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    let user = stmt
        .query_row(rusqlite::params![user_id], |row| {
            let id: String = row.get(0)?;
            let email: String = row.get(1)?;
            let totp_secret: Option<String> = row.get(2)?;
            let created_at: String = row.get(3)?;

            Ok(UserInfo {
                id,
                email,
                totp_enabled: totp_secret.is_some(),
                webauthn_credentials_count: 0,
                created_at,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ErrorResponse::not_found(ApiError::user_not_found())
            }
            _ => {
                error!("Query error: {}", e);
                ErrorResponse::internal_error(ApiError::internal_error())
            }
        })?;

    Ok(Json(user))
}

/// List sessions for a user
pub async fn list_user_sessions(
    State(state): State<AdminState>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut stmt = state.db.conn
        .prepare(
            "SELECT token, user_id, created_at, expires_at, revoked FROM refresh_tokens WHERE user_id = ?1 ORDER BY created_at DESC"
        )
        .map_err(|e| {
            error!("Database error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    let sessions = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(SessionInfo {
                token: row.get(0)?,
                user_id: row.get(1)?,
                created_at: row.get(2)?,
                expires_at: row.get(3)?,
                revoked: row.get(4)?,
            })
        })
        .map_err(|e| {
            error!("Query error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Row mapping error: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    Ok(Json(sessions))
}

/// Revoke a specific session
pub async fn revoke_session(
    State(state): State<AdminState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    Session::revoke_refresh_token(&state.db, &token).map_err(|e| {
        error!("Failed to revoke session: {}", e);
        ErrorResponse::internal_error(ApiError::internal_error())
    })?;

    state.audit.log(
        &state.db.conn,
        crate::audit::AuditEventType::SessionRevoked,
        None,
        None,
        None,
        None,
        Some(&token),
        true,
    );

    Ok((StatusCode::OK, "Session revoked"))
}

/// Revoke all sessions for a user
pub async fn revoke_all_user_sessions(
    State(state): State<AdminState>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    state.db.conn
        .execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE user_id = ?1",
            rusqlite::params![user_id],
        )
        .map_err(|e| {
            error!("Failed to revoke sessions: {}", e);
            ErrorResponse::internal_error(ApiError::internal_error())
        })?;

    state.audit.log(
        &state.db.conn,
        crate::audit::AuditEventType::SessionRevoked,
        Some(&user_id),
        None,
        None,
        None,
        Some("all_sessions"),
        true,
    );

    Ok((StatusCode::OK, "All sessions revoked"))
}

/// Get system statistics
#[derive(Serialize)]
pub struct SystemStats {
    pub total_users: i32,
    pub total_sessions: i32,
    pub active_sessions: i32,
    pub total_audit_logs: i32,
}

pub async fn get_stats(
    State(state): State<AdminState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let total_users: i32 = state.db.conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .unwrap_or(0);

    let total_sessions: i32 = state.db.conn
        .query_row("SELECT COUNT(*) FROM refresh_tokens", [], |row| row.get(0))
        .unwrap_or(0);

    let active_sessions: i32 = state.db.conn
        .query_row(
            "SELECT COUNT(*) FROM refresh_tokens WHERE revoked = 0 AND datetime(expires_at) > datetime('now')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_audit_logs: i32 = state.db.conn
        .query_row("SELECT COUNT(*) FROM audit_logs", [], |row| row.get(0))
        .unwrap_or(0);

    let stats = SystemStats {
        total_users,
        total_sessions,
        active_sessions,
        total_audit_logs,
    };

    Ok(Json(stats))
}

/// Create admin router
pub fn admin_router(state: AdminState) -> Router {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id/sessions", get(list_user_sessions))
        .route("/sessions/:token", delete(revoke_session))
        .route("/users/:user_id/sessions", delete(revoke_all_user_sessions))
        .route("/stats", get(get_stats))
        .with_state(state)
}
