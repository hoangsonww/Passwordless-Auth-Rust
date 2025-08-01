use crate::db::Database;
use crate::models::MagicLink;
use rusqlite::params;
use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MagicLinkError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("invalid or expired token")]
    Invalid,
    #[error("already used")]
    Used,
}

impl MagicLink {
    pub fn generate(
        db: &Database,
        user_id: &str,
        expiry_seconds: i64,
    ) -> Result<String, MagicLinkError> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Database::now_ts() + expiry_seconds;
        db.conn.execute(
            "INSERT INTO magic_links (token, user_id, expires_at, used) VALUES (?1, ?2, ?3, 0)",
            params![token, user_id, expires_at],
        )?;
        Ok(token)
    }

    pub fn consume(db: &Database, token: &str) -> Result<String, MagicLinkError> {
        let mut stmt = db
            .conn
            .prepare("SELECT user_id, expires_at, used FROM magic_links WHERE token = ?1")?;
        let mut rows = stmt.query(params![token])?;
        if let Some(r) = rows.next()? {
            let user_id: String = r.get(0)?;
            let expires_at: i64 = r.get(1)?;
            let used: i64 = r.get(2)?;
            let now = Database::now_ts();
            if used != 0 {
                return Err(MagicLinkError::Used);
            }
            if now > expires_at {
                return Err(MagicLinkError::Invalid);
            }
            db.conn.execute(
                "UPDATE magic_links SET used = 1 WHERE token = ?1",
                params![token],
            )?;
            Ok(user_id)
        } else {
            Err(MagicLinkError::Invalid)
        }
    }
}
