use crate::axum::{errors::AppError, SharedState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{magic_link::MagicLink, jwt, session::Session, totp, webauthn::WebauthnState};

#[derive(Deserialize)]
pub struct RequestMagicBody {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn request_magic(
    State(state): State<SharedState>,
    Json(body): Json<RequestMagicBody>,
) -> Result<(StatusCode, &'static str), AppError> {
    let user_id = state
        .db
        .get_or_create_user(&body.email)
        .map_err(AppError::from)?;
    let token = MagicLink::generate(&state.db, &user_id, state.cfg.magic_link_expiry_seconds)
        .map_err(|e| match e {
            crate::magic_link::MagicLinkError::Db(e) => AppError::Db(e),
            _ => AppError::BadRequest("failed to generate magic link".into()),
        })?;

    // Send via email queue
    let subject = "Your Magic Login Link";
    let magic_url = format!("{}?token={}", state.cfg.magic_link_base_url, token);
    let text = format!("Login: {}", magic_url);
    let html = format!(
        "<p>Click to login (valid briefly): <a href=\"{0}\">{0}</a></p>",
        magic_url
    );
    // enqueue directly into queue table so worker picks up
    crate::email_queue::EmailQueue::enqueue(
        &state.db,
        &body.email,
        subject,
        &text,
        Some(&html),
    )
    .map_err(|e| AppError::Email(format!("queueing failed: {}", e)))?;

    Ok((StatusCode::OK, "magic link enqueued"))
}

pub async fn verify_magic(
    State(state): State<SharedState>,
    Query(q): Query<VerifyQuery>,
) -> Result<Json<AuthResponse>, AppError> {
    let user_id = MagicLink::consume(&state.db, &q.token).map_err(|e| match e {
        crate::magic_link::MagicLinkError::Used => AppError::MagicLinkUsed,
        crate::magic_link::MagicLinkError::Invalid => AppError::InvalidMagicLink,
        crate::magic_link::MagicLinkError::Db(db_err) => AppError::Db(db_err),
    })?;

    let access = jwt::create_token(
        &user_id,
        &state.cfg.jwt_secret,
        state.cfg.access_token_expiry_seconds,
        "access",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;
    let refresh_raw =
        Session::create_refresh_token(&state.db, &user_id, state.cfg.refresh_token_expiry_seconds)
            .map_err(AppError::from)?;
    let refresh_jwt = jwt::create_token(
        &refresh_raw,
        &state.cfg.jwt_secret,
        state.cfg.refresh_token_expiry_seconds,
        "refresh",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;

    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh_jwt,
    }))
}

#[derive(Deserialize)]
pub struct TotpEnrollBody {
    pub email: String,
}

#[derive(Serialize)]
pub struct TotpEnrollResp {
    pub secret: String,
    pub otpauth_url: String,
}

pub async fn totp_enroll(
    State(state): State<SharedState>,
    Json(body): Json<TotpEnrollBody>,
) -> Result<Json<TotpEnrollResp>, AppError> {
    let user_id = state
        .db
        .get_or_create_user(&body.email)
        .map_err(AppError::from)?;
    let secret = totp::generate_secret();

    state
        .db
        .conn
        .execute(
            "UPDATE users SET totp_secret = ?1 WHERE id = ?2",
            rusqlite::params![secret, user_id],
        )
        .map_err(AppError::from)?;

    let url = totp::generate_otpauth_url(&secret, &body.email, "PasswordlessAuth");
    Ok(Json(TotpEnrollResp {
        secret,
        otpauth_url: url,
    }))
}

#[derive(Deserialize)]
pub struct TotpVerifyBody {
    pub email: String,
    pub code: String,
}

pub async fn totp_verify(
    State(state): State<SharedState>,
    Json(body): Json<TotpVerifyBody>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut stmt = state
        .db
        .conn
        .prepare("SELECT id, totp_secret FROM users WHERE email = ?1")
        .map_err(AppError::from)?;
    let mut rows = stmt
        .query(rusqlite::params![body.email])
        .map_err(AppError::from)?;
    if let Some(r) = rows.next().map_err(|e| AppError::Db(e))? {
        let user_id: String = r.get(0).map_err(AppError::from)?;
        let secret: Option<String> = r.get(1).map_err(AppError::from)?;
        if let Some(s) = secret {
            totp::verify_code(&s, &body.code).map_err(|_| AppError::TotpInvalid)?;
            let access = jwt::create_token(
                &user_id,
                &state.cfg.jwt_secret,
                state.cfg.access_token_expiry_seconds,
                "access",
            )
            .map_err(|e| AppError::Jwt(e.to_string()))?;
            let refresh_raw = Session::create_refresh_token(
                &state.db,
                &user_id,
                state.cfg.refresh_token_expiry_seconds,
            )
            .map_err(AppError::from)?;
            let refresh_jwt = jwt::create_token(
                &refresh_raw,
                &state.cfg.jwt_secret,
                state.cfg.refresh_token_expiry_seconds,
                "refresh",
            )
            .map_err(|e| AppError::Jwt(e.to_string()))?;
            return Ok(Json(AuthResponse {
                access_token: access,
                refresh_token: refresh_jwt,
            }));
        } else {
            return Err(AppError::BadRequest("totp not enrolled".into()));
        }
    }
    Err(AppError::BadRequest("user not found".into()))
}

#[derive(Deserialize)]
pub struct RefreshBody {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<SharedState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<AuthResponse>, AppError> {
    let claims = jwt::verify_token(&body.refresh_token, &state.cfg.jwt_secret)
        .map_err(|e| AppError::Jwt(e.to_string()))?;
    if claims.kind != "refresh" {
        return Err(AppError::BadRequest("invalid token kind".into()));
    }
    let raw_refresh = claims.sub;
    let user_id = Session::validate_refresh_token(&state.db, &raw_refresh)
        .map_err(|_| AppError::InvalidRefreshToken)?;
    let access = jwt::create_token(
        &user_id,
        &state.cfg.jwt_secret,
        state.cfg.access_token_expiry_seconds,
        "access",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;
    let new_refresh_raw = Session::create_refresh_token(
        &state.db,
        &user_id,
        state.cfg.refresh_token_expiry_seconds,
    )
    .map_err(AppError::from)?;
    let refresh_jwt = jwt::create_token(
        &new_refresh_raw,
        &state.cfg.jwt_secret,
        state.cfg.refresh_token_expiry_seconds,
        "refresh",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh_jwt,
    }))
}

#[derive(Deserialize)]
pub struct WebauthnRegisterOptionsBody {
    pub email: String,
}

#[derive(Deserialize)]
pub struct WebauthnRegisterCompleteBody {
    pub pending_id: String,
    pub response: serde_json::Value,
}

pub async fn webauthn_register_options(
    State(state): State<SharedState>,
    Json(body): Json<WebauthnRegisterOptionsBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = state
        .db
        .get_or_create_user(&body.email)
        .map_err(AppError::from)?;
    let opts = state
        .webauthn
        .start_registration(&state.db, &user_id, &body.email)
        .map_err(|e| AppError::WebAuthn(format!("{:?}", e)))?;
    Ok(Json(serde_json::to_value(opts).map_err(|e| AppError::WebAuthn(e.to_string()))?))
}

pub async fn webauthn_register_complete(
    State(state): State<SharedState>,
    Json(body): Json<WebauthnRegisterCompleteBody>,
) -> Result<(&'static str, StatusCode), AppError> {
    state
        .webauthn
        .finish_registration(&state.db, &body.pending_id, body.response.clone())
        .map_err(|e| AppError::WebAuthn(format!("{:?}", e)))?;
    Ok(("registered", StatusCode::OK))
}

#[derive(Deserialize)]
pub struct WebauthnLoginOptionsBody {
    pub email: String,
}

pub async fn webauthn_login_options(
    State(state): State<SharedState>,
    Json(body): Json<WebauthnLoginOptionsBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // get user id by email
    let mut stmt = state
        .db
        .conn
        .prepare("SELECT id FROM users WHERE email = ?1")
        .map_err(AppError::from)?;
    let mut rows = stmt
        .query(rusqlite::params![body.email])
        .map_err(AppError::from)?;
    if let Some(r) = rows.next().map_err(|e| AppError::Db(e))? {
        let user_id: String = r.get(0).map_err(AppError::from)?;
        let opts = state
            .webauthn
            .start_login(&state.db, &user_id)
            .map_err(|e| AppError::WebAuthn(format!("{:?}", e)))?;
        Ok(Json(serde_json::to_value(opts).map_err(|e| AppError::WebAuthn(e.to_string()))?))
    } else {
        Err(AppError::BadRequest("user not found".into()))
    }
}

#[derive(Deserialize)]
pub struct WebauthnLoginCompleteBody {
    pub pending_id: String,
    pub response: serde_json::Value,
}

pub async fn webauthn_login_complete(
    State(state): State<SharedState>,
    Json(body): Json<WebauthnLoginCompleteBody>,
) -> Result<Json<AuthResponse>, AppError> {
    let user_id = state
        .webauthn
        .finish_login(&state.db, &body.pending_id, body.response.clone())
        .map_err(|e| AppError::WebAuthn(format!("{:?}", e)))?;

    let access = jwt::create_token(
        &user_id,
        &state.cfg.jwt_secret,
        state.cfg.access_token_expiry_seconds,
        "access",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;
    let refresh_raw = Session::create_refresh_token(
        &state.db,
        &user_id,
        state.cfg.refresh_token_expiry_seconds,
    )
    .map_err(AppError::from)?;
    let refresh_jwt = jwt::create_token(
        &refresh_raw,
        &state.cfg.jwt_secret,
        state.cfg.refresh_token_expiry_seconds,
        "refresh",
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh_jwt,
    }))
}
