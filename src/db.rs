use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sql(#[from] rusqlite::Error),
}

impl Database {
    pub fn open(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        // enable foreign keys
        conn.pragma_update(None, "foreign_keys", &"ON")?;
        Ok(Self { conn })
    }

    pub fn migrate(&self, sql: &str) -> Result<(), DbError> {
        self.conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn now_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    // helper for inserting user if not exists
    pub fn get_or_create_user(&self, email: &str) -> Result<String, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM users WHERE email = ?1")?;
        let mut rows = stmt.query(params![email])?;
        if let Some(r) = rows.next()? {
            let id: String = r.get(0)?;
            Ok(id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let now = Self::now_ts();
            self.conn.execute(
                "INSERT INTO users (id, email, created_at) VALUES (?1, ?2, ?3)",
                params![id, email, now],
            )?;
            Ok(id)
        }
    }
}
