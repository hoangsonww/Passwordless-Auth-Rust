use base32::{Alphabet, encode};
use oath::{totp_custom};
use rand::{distributions::Alphanumeric, Rng};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TotpError {
    #[error("invalid code")]
    Invalid,
}

pub fn generate_secret() -> String {
    // generate 20 bytes -> base32
    let bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
    encode(Alphabet::RFC4648 { padding: false }, &bytes)
}

pub fn generate_otpauth_url(secret: &str, user_email: &str, issuer: &str) -> String {
    // Example: otpauth://totp/PasswordlessAuth:user@example.com?secret=ABCDEF&issuer=PasswordlessAuth&algorithm=SHA1&digits=6&period=30
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer, user_email, secret, issuer
    )
}

pub fn verify_code(secret: &str, code: &str) -> Result<(), TotpError> {
    // time step 30s, 6 digits, SHA1 default
    let now = oath::totp_raw_now(
        secret.as_bytes(),
        6,
        0,
        30,
    );
    let expected = format!("{:06}", now);
    if expected == code {
        Ok(())
    } else {
        // allow +/-1 step for clock skew
        let prev = oath::totp_custom(
            secret.as_bytes(),
            6,
            0,
            30,
            1,
            time::OffsetDateTime::now_utc().unix_timestamp() - 30,
        ).unwrap_or(0);
        let next = oath::totp_custom(
            secret.as_bytes(),
            6,
            0,
            30,
            1,
            time::OffsetDateTime::now_utc().unix_timestamp() + 30,
        ).unwrap_or(0);
        if format!("{:06}", prev) == code || format!("{:06}", next) == code {
            Ok(())
        } else {
            Err(TotpError::Invalid)
        }
    }
}
