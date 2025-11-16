use serde::Deserialize;
use std::{env, fs, path::Path};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // JWT Configuration
    pub jwt_secret: String,
    pub access_token_expiry_seconds: i64,
    pub refresh_token_expiry_seconds: i64,

    // Magic Link Configuration
    pub magic_link_expiry_seconds: i64,
    pub magic_link_base_url: String,

    // SMTP Configuration
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub email_from: String,

    // WebAuthn Configuration
    pub webauthn_rp_id: String,
    pub webauthn_origin: String,
    pub webauthn_rp_name: String,

    // Database Configuration
    pub database_path: String,

    // Rate Limiting Configuration
    #[serde(default = "default_rate_limit_per_minute")]
    pub rate_limit_per_minute: u32,

    #[serde(default = "default_email_rate_limit_per_hour")]
    pub email_rate_limit_per_hour: u32,

    // CORS Configuration
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,

    #[serde(default = "default_cors_allow_all")]
    pub cors_allow_all: bool,

    // Server Configuration
    #[serde(default = "default_server_host")]
    pub server_host: String,

    #[serde(default = "default_server_port")]
    pub server_port: u16,

    // Webhook Configuration
    #[serde(default)]
    pub webhook_url: Option<String>,

    #[serde(default)]
    pub webhook_secret: Option<String>,

    // Observability
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_rate_limit_per_minute() -> u32 {
    60
}

fn default_email_rate_limit_per_hour() -> u32 {
    10
}

fn default_cors_allow_all() -> bool {
    false
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    3000
}

fn default_enable_metrics() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("environment variable error: {0}")]
    Env(String),
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        // Load .env file if it exists (optional)
        let _ = dotenvy::dotenv();

        // Load from TOML file
        let s = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&s)?;

        // Override with environment variables if present
        config.override_from_env()?;

        Ok(config)
    }

    /// Override configuration with environment variables
    fn override_from_env(&mut self) -> Result<(), ConfigError> {
        if let Ok(val) = env::var("JWT_SECRET") {
            self.jwt_secret = val;
        }
        if let Ok(val) = env::var("DATABASE_PATH") {
            self.database_path = val;
        }
        if let Ok(val) = env::var("SMTP_HOST") {
            self.smtp_host = val;
        }
        if let Ok(val) = env::var("SMTP_PORT") {
            self.smtp_port = val.parse().map_err(|_| {
                ConfigError::Env("Invalid SMTP_PORT".to_string())
            })?;
        }
        if let Ok(val) = env::var("SMTP_USERNAME") {
            self.smtp_username = val;
        }
        if let Ok(val) = env::var("SMTP_PASSWORD") {
            self.smtp_password = val;
        }
        if let Ok(val) = env::var("EMAIL_FROM") {
            self.email_from = val;
        }
        if let Ok(val) = env::var("WEBAUTHN_RP_ID") {
            self.webauthn_rp_id = val;
        }
        if let Ok(val) = env::var("WEBAUTHN_ORIGIN") {
            self.webauthn_origin = val;
        }
        if let Ok(val) = env::var("SERVER_HOST") {
            self.server_host = val;
        }
        if let Ok(val) = env::var("SERVER_PORT") {
            self.server_port = val.parse().map_err(|_| {
                ConfigError::Env("Invalid SERVER_PORT".to_string())
            })?;
        }
        if let Ok(val) = env::var("WEBHOOK_URL") {
            self.webhook_url = Some(val);
        }
        if let Ok(val) = env::var("WEBHOOK_SECRET") {
            self.webhook_secret = Some(val);
        }
        if let Ok(val) = env::var("CORS_ALLOWED_ORIGINS") {
            self.cors_allowed_origins = val.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(val) = env::var("LOG_LEVEL") {
            self.log_level = val;
        }

        Ok(())
    }
}
