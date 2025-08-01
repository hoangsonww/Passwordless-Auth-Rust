use crate::db::Database;
use crate::email::Emailer;
use rusqlite::params;
use uuid::Uuid;
use chrono::{Utc, Duration};
use thiserror::Error;
use std::thread;
use std::time::Duration as StdDuration;
use tracing::{info, error};

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
}

pub struct EmailQueue;

impl EmailQueue {
    pub fn enqueue(
        db: &Database,
        to_email: &str,
        subject: &str,
        body_text: &str,
        body_html: Option<&str>,
    ) -> Result<(), QueueError> {
        let id = Uuid::new_v4().to_string();
        let now = Database::now_ts();
        let next_try_at = now;
        db.conn.execute(
            "INSERT INTO email_queue (id, to_email, subject, body_text, body_html, attempts, next_try_at, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, 'pending')",
            params![
                id,
                to_email,
                subject,
                body_text,
                body_html.unwrap_or(""),
                next_try_at,
                now
            ],
        )?;
        Ok(())
    }

    pub fn fetch_due(db: &Database, limit: i64) -> Result<Vec<EmailTask>, QueueError> {
        let now = Database::now_ts();
        let mut stmt = db.conn.prepare(
            "SELECT id, to_email, subject, body_text, body_html, attempts FROM email_queue WHERE status IN ('pending','failed') AND next_try_at <= ?1 ORDER BY created_at ASC LIMIT ?2",
        )?;
        let mut rows = stmt.query(params![now, limit])?;
        let mut tasks = Vec::new();
        while let Some(r) = rows.next()? {
            let task = EmailTask {
                id: r.get(0)?,
                to_email: r.get(1)?,
                subject: r.get(2)?,
                body_text: r.get(3)?,
                body_html: r.get(4)?,
                attempts: r.get(5)?,
            };
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn mark_sending(db: &Database, id: &str) -> Result<(), QueueError> {
        db.conn.execute(
            "UPDATE email_queue SET status='sending' WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn mark_sent(db: &Database, id: &str) -> Result<(), QueueError> {
        let now = Database::now_ts();
        db.conn.execute(
            "UPDATE email_queue SET status='sent', sent_at=?1 WHERE id=?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn mark_failed(db: &Database, id: &str, err: &str, attempts: i64) -> Result<(), QueueError> {
        let backoff = 60 * 2_i64.pow(attempts as u32); // exponential backoff in seconds
        let next_try_at = Database::now_ts() + backoff;
        db.conn.execute(
            "UPDATE email_queue SET status='failed', last_error=?1, attempts=?2, next_try_at=?3 WHERE id=?4",
            params![err, attempts, next_try_at, id],
        )?;
        Ok(())
    }
}

pub struct EmailTask {
    pub id: String,
    pub to_email: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: String,
    pub attempts: i64,
}
