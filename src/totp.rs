use base32::{Alphabet, encode};
use rand::Rng;
use thiserror::Error;
use totp_lite::{totp_custom, Sha1};

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
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Decode secret from base32
    let secret_bytes = match base32::decode(Alphabet::RFC4648 { padding: false }, secret) {
        Some(bytes) => bytes,
        None => return Err(TotpError::Invalid),
    };

    // time step 30s, 6 digits, SHA1 default
    let now_code = totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp);

    if format!("{:06}", now_code) == code {
        Ok(())
    } else {
        // allow +/-1 step for clock skew (30 seconds)
        let prev_code = totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp - 30);
        let next_code = totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp + 30);

        if format!("{:06}", prev_code) == code || format!("{:06}", next_code) == code {
            Ok(())
        } else {
            Err(TotpError::Invalid)
        }
    }
}
