CREATE TABLE IF NOT EXISTS email_queue (
                                           id TEXT PRIMARY KEY,
                                           to_email TEXT NOT NULL,
                                           subject TEXT NOT NULL,
                                           body_text TEXT NOT NULL,
                                           body_html TEXT,
                                           attempts INTEGER NOT NULL DEFAULT 0,
                                           last_error TEXT,
                                           next_try_at INTEGER NOT NULL,
                                           created_at INTEGER NOT NULL,
                                           sent_at INTEGER,
                                           status TEXT NOT NULL DEFAULT 'pending' -- pending, sending, sent, failed
);
