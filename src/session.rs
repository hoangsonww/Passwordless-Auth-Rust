use crate::db::Database;
use rusqlite::params;
use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("token revoked or expired")]
    Invalid,
}

pub struct Session;

impl Session {
    pub fn create_refresh_token(
        db: &Database,
        user_id: &str,
        expiry_seconds: i64,
    ) -> Result<String, SessionError> {
        let token = Uuid::new_v4().to_string();
        let now = Database::now_ts();
        let expires_at = now + expiry_seconds;
        db.conn.execute(
            "INSERT INTO refresh_tokens (token, user_id, expires_at, revoked, created_at) VALUES (?1, ?2, ?3, 0, ?4)",
            params![token, user_id, expires_at, now],
        )?;
        Ok(token)
    }

    pub fn validate_refresh_token(
        db: &Database,
        token: &str,
    ) -> Result<String, SessionError> {
        let mut stmt = db.conn.prepare(
            "SELECT user_id, expires_at, revoked FROM refresh_tokens WHERE token = ?1",
        )?;
        let mut rows = stmt.query(params![token])?;
        if let Some(r) = rows.next()? {
            let user_id: String = r.get(0)?;
            let expires_at: i64 = r.get(1)?;
            let revoked: i64 = r.get(2)?;
            let now = Database::now_ts();
            if revoked != 0 || now > expires_at {
                return Err(SessionError::Invalid);
            }
            Ok(user_id)
        } else {
            Err(SessionError::Invalid)
        }
    }

    pub fn revoke_refresh_token(db: &Database, token: &str) -> Result<(), SessionError> {
        db.conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE token = ?1",
            params![token],
        )?;
        Ok(())
    }
}
