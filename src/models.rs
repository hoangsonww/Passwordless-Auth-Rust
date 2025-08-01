use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub totp_secret: Option<String>,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct MagicLink {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
    pub used: bool,
}

#[derive(Debug)]
pub struct RefreshToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
    pub revoked: bool,
    pub created_at: i64,
}
