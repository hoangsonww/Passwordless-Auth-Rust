PRAGMA journal_mode=WAL;

CREATE TABLE IF NOT EXISTS users (
                                     id TEXT PRIMARY KEY,
                                     email TEXT UNIQUE NOT NULL,
                                     totp_secret TEXT,
                                     created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS magic_links (
                                           token TEXT PRIMARY KEY,
                                           user_id TEXT NOT NULL,
                                           expires_at INTEGER NOT NULL,
                                           used INTEGER NOT NULL DEFAULT 0,
                                           FOREIGN KEY(user_id) REFERENCES users(id)
    );

CREATE TABLE IF NOT EXISTS refresh_tokens (
                                              token TEXT PRIMARY KEY,
                                              user_id TEXT NOT NULL,
                                              expires_at INTEGER NOT NULL,
                                              revoked INTEGER NOT NULL DEFAULT 0,
                                              created_at INTEGER NOT NULL,
                                              FOREIGN KEY(user_id) REFERENCES users(id)
    );

CREATE TABLE IF NOT EXISTS webauthn_registrations (
                                                      id TEXT PRIMARY KEY,
                                                      user_id TEXT NOT NULL,
                                                      credential_id BLOB NOT NULL,
                                                      public_key BLOB NOT NULL,
                                                      sign_count INTEGER NOT NULL,
                                                      transports TEXT,
                                                      created_at INTEGER NOT NULL,
                                                      FOREIGN KEY(user_id) REFERENCES users(id)
    );

CREATE TABLE IF NOT EXISTS pending_webauthn (
                                                id TEXT PRIMARY KEY,
                                                user_id TEXT NOT NULL,
                                                challenge BLOB NOT NULL,
                                                purpose TEXT NOT NULL, -- register or login
                                                created_at INTEGER NOT NULL,
                                                expires_at INTEGER NOT NULL,
                                                serialized_options BLOB NOT NULL,
                                                FOREIGN KEY(user_id) REFERENCES users(id)
    );
