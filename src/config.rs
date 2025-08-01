use serde::Deserialize;
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub access_token_expiry_seconds: i64,
    pub refresh_token_expiry_seconds: i64,
    pub magic_link_expiry_seconds: i64,
    pub magic_link_base_url: String,

    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub email_from: String,

    pub webauthn_rp_id: String,
    pub webauthn_origin: String,
    pub webauthn_rp_name: String,

    pub database_path: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Toml(#[from] toml::de::Error),
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let s = fs::read_to_string(path)?;
        let c = toml::from_str::<Config>(&s)?;
        Ok(c)
    }
}
