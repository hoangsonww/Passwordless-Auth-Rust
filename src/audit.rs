use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Audit event types for tracking authentication activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// User requested a magic link
    MagicLinkRequested,
    /// User verified a magic link successfully
    MagicLinkVerified,
    /// Magic link verification failed
    MagicLinkFailed,
    /// User enrolled TOTP
    TotpEnrolled,
    /// User verified TOTP successfully
    TotpVerified,
    /// TOTP verification failed
    TotpFailed,
    /// WebAuthn registration started
    WebauthnRegisterStarted,
    /// WebAuthn registration completed
    WebauthnRegisterCompleted,
    /// WebAuthn registration failed
    WebauthnRegisterFailed,
    /// WebAuthn login started
    WebauthnLoginStarted,
    /// WebAuthn login completed
    WebauthnLoginCompleted,
    /// WebAuthn login failed
    WebauthnLoginFailed,
    /// Token refreshed
    TokenRefreshed,
    /// Token refresh failed
    TokenRefreshFailed,
    /// Session revoked
    SessionRevoked,
    /// User logged out
    UserLoggedOut,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Invalid request
    InvalidRequest,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MagicLinkRequested => "magic_link_requested",
            Self::MagicLinkVerified => "magic_link_verified",
            Self::MagicLinkFailed => "magic_link_failed",
            Self::TotpEnrolled => "totp_enrolled",
            Self::TotpVerified => "totp_verified",
            Self::TotpFailed => "totp_failed",
            Self::WebauthnRegisterStarted => "webauthn_register_started",
            Self::WebauthnRegisterCompleted => "webauthn_register_completed",
            Self::WebauthnRegisterFailed => "webauthn_register_failed",
            Self::WebauthnLoginStarted => "webauthn_login_started",
            Self::WebauthnLoginCompleted => "webauthn_login_completed",
            Self::WebauthnLoginFailed => "webauthn_login_failed",
            Self::TokenRefreshed => "token_refreshed",
            Self::TokenRefreshFailed => "token_refresh_failed",
            Self::SessionRevoked => "session_revoked",
            Self::UserLoggedOut => "user_logged_out",
            Self::RateLimitExceeded => "rate_limit_exceeded",
            Self::InvalidRequest => "invalid_request",
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    pub id: i64,
    pub event_type: String,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<String>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

/// Audit logger for tracking authentication events
pub struct AuditLogger {
    // In a production system, this might write to a separate database or log service
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {}
    }

    /// Log an audit event to the database
    pub fn log(
        &self,
        conn: &Connection,
        event_type: AuditEventType,
        user_id: Option<&str>,
        email: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        metadata: Option<&str>,
        success: bool,
    ) {
        let event_str = event_type.as_str();

        // Log to structured logs
        info!(
            event = event_str,
            user_id = user_id,
            email = email,
            ip_address = ip_address,
            success = success,
            "Audit event"
        );

        // Also persist to database
        let result = conn.execute(
            "INSERT INTO audit_logs (event_type, user_id, email, ip_address, user_agent, metadata, success, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                event_str,
                user_id,
                email,
                ip_address,
                user_agent,
                metadata,
                success,
                Utc::now().to_rfc3339()
            ],
        );

        if let Err(e) = result {
            error!("Failed to write audit log to database: {}", e);
        }
    }

    /// Get recent audit logs for a user
    pub fn get_user_logs(
        &self,
        conn: &Connection,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<AuditLog>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, event_type, user_id, email, ip_address, user_agent, metadata, success, created_at
             FROM audit_logs
             WHERE user_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let logs = stmt.query_map(rusqlite::params![user_id, limit], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                event_type: row.get(1)?,
                user_id: row.get(2)?,
                email: row.get(3)?,
                ip_address: row.get(4)?,
                user_agent: row.get(5)?,
                metadata: row.get(6)?,
                success: row.get(7)?,
                created_at: {
                    let dt_str: String = row.get(8)?;
                    DateTime::parse_from_rfc3339(&dt_str)
                        .unwrap()
                        .with_timezone(&Utc)
                },
            })
        })?;

        logs.collect()
    }

    /// Get all audit logs with pagination
    pub fn get_all_logs(
        &self,
        conn: &Connection,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<AuditLog>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, event_type, user_id, email, ip_address, user_agent, metadata, success, created_at
             FROM audit_logs
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let logs = stmt.query_map(rusqlite::params![limit, offset], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                event_type: row.get(1)?,
                user_id: row.get(2)?,
                email: row.get(3)?,
                ip_address: row.get(4)?,
                user_agent: row.get(5)?,
                metadata: row.get(6)?,
                success: row.get(7)?,
                created_at: {
                    let dt_str: String = row.get(8)?;
                    DateTime::parse_from_rfc3339(&dt_str)
                        .unwrap()
                        .with_timezone(&Utc)
                },
            })
        })?;

        logs.collect()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
