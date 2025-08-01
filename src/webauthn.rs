use crate::config::Config;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use webauthn_rs::prelude::*;
use rusqlite::params;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Error)]
pub enum WebauthnError {
    #[error("webauthn internal error: {0}")]
    Internal(#[from] WebauthnErrorKind),
    #[error("missing pending challenge")]
    MissingChallenge,
    #[error("verification failed")]
    VerificationFailed,
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}

#[derive(Serialize, Deserialize)]
pub struct PublicKeyCredentialDescriptorSerializable {
    pub id: Vec<u8>,
    pub transports: Option<Vec<AuthenticatorTransport>>,
}

pub struct WebauthnState {
    pub rp: RelyingParty,
}

impl WebauthnState {
    pub fn new(cfg: &Config) -> Self {
        let rp = RelyingParty::builder(cfg.webauthn_rp_id.clone(), cfg.webauthn_origin.clone())
            .name(cfg.webauthn_rp_name.clone())
            .build()
            .expect("invalid RP setup");
        Self { rp }
    }

    pub fn start_registration(
        &self,
        db: &Database,
        user_id: &str,
        user_name: &str,
    ) -> Result<PublicKeyCredentialCreationOptions, WebauthnError> {
        let user = PublicKeyCredentialUserEntityBuilder::new(user_id.as_bytes().to_vec())
            .name(user_name.to_string())
            .display_name(user_name.to_string())
            .build()
            .map_err(We)??;

        let creation = self
            .rp
            .start_passkey_registration(Some(user), None)
            .map_err(We)??;

        let challenge = creation.challenge().clone();
        let id = Uuid::new_v4().to_string();
        let now = Database::now_ts();
        let expires_at = now + 300; // 5 minutes

        let serialized = serde_json::to_vec(&creation).unwrap();
        db.conn.execute(
            "INSERT INTO pending_webauthn (id, user_id, challenge, purpose, created_at, expires_at, serialized_options) VALUES (?1, ?2, ?3, 'register', ?4, ?5, ?6)",
            params![id, user_id, challenge.clone(), now, expires_at, serialized],
        )?;

        Ok(creation)
    }

    pub fn finish_registration(
        &self,
        db: &Database,
        pending_id: &str,
        response: serde_json::Value,
    ) -> Result<(), WebauthnError> {
        // load pending
        let mut stmt = db.conn.prepare(
            "SELECT user_id, challenge, serialized_options, expires_at FROM pending_webauthn WHERE id = ?1 AND purpose = 'register'"
        )?;
        let mut rows = stmt.query(params![pending_id])?;
        let row = rows.next()?.ok_or(WebauthnError::MissingChallenge)?;
        let user_id: String = row.get(0)?;
        let challenge: Vec<u8> = row.get(1)?;
        let serialized: Vec<u8> = row.get(2)?;
        let expires_at: i64 = row.get(3)?;
        if Database::now_ts() > expires_at {
            return Err(WebauthnError::VerificationFailed);
        }

        let options: PublicKeyCredentialCreationOptions =
            serde_json::from_slice(&serialized).map_err(|_| WebauthnError::VerificationFailed)?;
        let attestation_response: PublicKeyCredential =
            serde_json::from_value(response).map_err(|_| WebauthnError::VerificationFailed)?;

        let registration_info = self
            .rp
            .finish_passkey_registration(&options, &attestation_response, None)
            .map_err(We)??;

        // Persist credential
        let registration_id = Uuid::new_v4().to_string();
        let credential_id = registration_info.cred_id().clone();
        let public_key = registration_info.credential_public_key().clone();
        let sign_count = registration_info.sign_count();
        let transports = serde_json::to_string(
            &registration_info
                .transports()
                .unwrap_or(&vec![])
                .iter()
                .map(|t| t.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let now = Database::now_ts();

        db.conn.execute(
            "INSERT INTO webauthn_registrations (id, user_id, credential_id, public_key, sign_count, transports, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                registration_id,
                user_id,
                credential_id,
                public_key,
                sign_count as i64,
                transports,
                now
            ],
        )?;

        // cleanup pending
        db.conn.execute("DELETE FROM pending_webauthn WHERE id = ?1", params![pending_id])?;
        Ok(())
    }

    pub fn start_login(
        &self,
        db: &Database,
        user_id: &str,
    ) -> Result<PublicKeyCredentialRequestOptions, WebauthnError> {
        // load existing credentials to exclude none
        let mut stmt = db.conn.prepare(
            "SELECT credential_id FROM webauthn_registrations WHERE user_id = ?1",
        )?;
        let mut rows = stmt.query(params![user_id])?;
        let mut allow_list = vec![];
        while let Some(r) = rows.next()? {
            let cred_id: Vec<u8> = r.get(0)?;
            allow_list.push(PublicKeyCredentialDescriptor::new(cred_id.clone(), None));
        }
        let request = self
            .rp
            .start_passkey_authentication(Some(allow_list), None)
            .map_err(We)??;

        let challenge = request.challenge().clone();
        let id = Uuid::new_v4().to_string();
        let now = Database::now_ts();
        let expires_at = now + 300;
        let serialized = serde_json::to_vec(&request).unwrap();

        db.conn.execute(
            "INSERT INTO pending_webauthn (id, user_id, challenge, purpose, created_at, expires_at, serialized_options) VALUES (?1, ?2, ?3, 'login', ?4, ?5, ?6)",
            params![id, user_id, challenge.clone(), now, expires_at, serialized],
        )?;

        Ok(request)
    }

    pub fn finish_login(
        &self,
        db: &Database,
        pending_id: &str,
        response: serde_json::Value,
    ) -> Result<String, WebauthnError> {
        let mut stmt = db.conn.prepare(
            "SELECT user_id, serialized_options, expires_at FROM pending_webauthn WHERE id = ?1 AND purpose = 'login'",
        )?;
        let mut rows = stmt.query(params![pending_id])?;
        let row = rows.next()?.ok_or(WebauthnError::MissingChallenge)?;
        let user_id: String = row.get(0)?;
        let serialized: Vec<u8> = row.get(1)?;
        let expires_at: i64 = row.get(2)?;
        if Database::now_ts() > expires_at {
            return Err(WebauthnError::VerificationFailed);
        }
        let options: PublicKeyCredentialRequestOptions =
            serde_json::from_slice(&serialized).map_err(|_| WebauthnError::VerificationFailed)?;
        let assertion_response: PublicKeyCredential =
            serde_json::from_value(response).map_err(|_| WebauthnError::VerificationFailed)?;

        let authentication_info = self
            .rp
            .finish_passkey_authentication(&options, &assertion_response, None)
            .map_err(We)??;

        // verify credential exists and update sign_count
        let credential_id = authentication_info.cred_id().clone();
        let mut stmt2 = db.conn.prepare("SELECT id, sign_count FROM webauthn_registrations WHERE credential_id = ?1")?;
        let mut rows2 = stmt2.query(params![credential_id.clone()])?;
        if let Some(r2) = rows2.next()? {
            let reg_id: String = r2.get(0)?;
            let stored_sign_count: i64 = r2.get(1)?;
            let new_sign_count = authentication_info.sign_count() as i64;
            if new_sign_count <= stored_sign_count {
                return Err(WebauthnError::VerificationFailed);
            }
            db.conn.execute(
                "UPDATE webauthn_registrations SET sign_count = ?1 WHERE id = ?2",
                params![new_sign_count, reg_id],
            )?;
        } else {
            return Err(WebauthnError::VerificationFailed);
        }

        // cleanup pending
        db.conn.execute("DELETE FROM pending_webauthn WHERE id = ?1", params![pending_id])?;

        Ok(user_id)
    }
}

// helper to convert webauthn-rs internal errors
fn We(e: webauthn_rs::prelude::WebauthnError) -> WebauthnErrorKind {
    WebauthnErrorKind::from(e)
}
