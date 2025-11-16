-- Production-ready enhancements migration

-- Audit logs table for tracking all authentication events
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    user_id TEXT,
    email TEXT,
    ip_address TEXT,
    user_agent TEXT,
    metadata TEXT,
    success BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Index for efficient audit log queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);

-- Add indexes to existing tables for better performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_magic_links_expires_at ON magic_links(expires_at);
CREATE INDEX IF NOT EXISTS idx_webauthn_registrations_user_id ON webauthn_registrations(user_id);

-- IP blocklist/allowlist table
CREATE TABLE IF NOT EXISTS ip_filters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_address TEXT NOT NULL UNIQUE,
    filter_type TEXT NOT NULL CHECK(filter_type IN ('allow', 'block')),
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_ip_filters_type ON ip_filters(filter_type);

-- Failed login attempts tracking (for account lockout)
CREATE TABLE IF NOT EXISTS failed_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL,
    ip_address TEXT,
    attempt_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_failed_attempts_email ON failed_attempts(email);
CREATE INDEX IF NOT EXISTS idx_failed_attempts_created_at ON failed_attempts(created_at);

-- System configuration table for runtime settings
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
