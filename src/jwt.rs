use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
    pub kind: String, // "access" | "refresh"
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("jwt encode error: {0}")]
    Encode(#[from] jsonwebtoken::errors::Error),
    #[error("jwt decode error: {0}")]
    Decode(#[from] jsonwebtoken::errors::Error),
}

pub fn create_token(
    user_id: &str,
    secret: &str,
    ttl_seconds: i64,
    kind: &str,
) -> Result<String, JwtError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        kind: kind.to_string(),
    };
    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}
