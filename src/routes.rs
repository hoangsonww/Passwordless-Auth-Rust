use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use crate::{
    config::Config,
    db::Database,
    email::Emailer,
    magic_link::{MagicLink, MagicLinkError},
    jwt,
    session::Session,
    totp,
    webauthn::WebauthnState,
};
use std::sync::Arc;
use tracing::{info, error};

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub db: Arc<Database>,
    pub emailer: Arc<Emailer>,
    pub webauthn: Arc<WebauthnState>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/request/magic", post(request_magic))
        .route("/verify/magic", get(verify_magic))
        .route("/totp/enroll", post(totp_enroll))
        .route("/totp/verify", post(totp_verify))
        .route("/token/refresh", post(refresh_token))
        .route("/webauthn/register/options", post(webauthn_register_options))
        .route("/webauthn/register/complete", post(webauthn_register_complete))
        .route("/webauthn/login/options", post(webauthn_login_options))
        .route("/webauthn/login/complete", post(webauthn_login_complete))
        .with_state(state)
}

#[derive(Deserialize)]
struct RequestMagicBody {
    email: String,
}

async fn request_magic(
    State(state): State<AppState>,
    Json(body): Json<RequestMagicBody>,
) -> impl IntoResponse {
    let user_id = match state.db.get_or_create_user(&body.email) {
        Ok(id) => id,
        Err(e) => {
            error!("user creation failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };
    match MagicLink::generate(&state.db, &user_id, state.cfg.magic_link_expiry_seconds) {
        Ok(token) => {
            if let Err(e) = state.emailer.send_magic_link(&body.email, &token) {
                error!("email send failed: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "email failed").into_response();
            }
            (StatusCode::OK, "magic link sent").into_response()
        }
        Err(e) => {
            error!("magic link generation failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
        }
    }
}

#[derive(Deserialize)]
struct VerifyQuery {
    token: String,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

async fn verify_magic(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> impl IntoResponse {
    match MagicLink::consume(&state.db, &q.token) {
        Ok(user_id) => {
            // issue tokens
            let access = jwt::create_token(
                &user_id,
                &state.cfg.jwt_secret,
                state.cfg.access_token_expiry_seconds,
                "access",
            )
            .unwrap();
            let refresh =
                Session::create_refresh_token(&state.db, &user_id, state.cfg.refresh_token_expiry_seconds)
                    .unwrap();
            let refresh_jwt = jwt::create_token(
                &refresh,
                &state.cfg.jwt_secret,
                state.cfg.refresh_token_expiry_seconds,
                "refresh",
            )
            .unwrap();
            let resp = AuthResponse {
                access_token: access,
                refresh_token: refresh_jwt,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(MagicLinkError::Used) => (StatusCode::BAD_REQUEST, "link already used").into_response(),
        Err(MagicLinkError::Invalid) => (StatusCode::BAD_REQUEST, "invalid or expired").into_response(),
        Err(e) => {
            error!("verify magic error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
        }
    }
}

#[derive(Deserialize)]
struct TotpEnrollBody {
    email: String,
}

#[derive(Serialize)]
struct TotpEnrollResp {
    secret: String,
    otpauth_url: String,
}

async fn totp_enroll(
    State(state): State<AppState>,
    Json(body): Json<TotpEnrollBody>,
) -> impl IntoResponse {
    let user_id = match state.db.get_or_create_user(&body.email) {
        Ok(id) => id,
        Err(e) => {
            error!("user get/create failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };

    let secret = totp::generate_secret();
    // store in user record
    if let Err(e) = state.db.conn.execute(
        "UPDATE users SET totp_secret = ?1 WHERE id = ?2",
        rusqlite::params![secret, user_id],
    ) {
        error!("saving totp secret failed: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
    }

    let url = totp::generate_otpauth_url(&secret, &body.email, "ShadowVault");
    let resp = TotpEnrollResp {
        secret,
        otpauth_url: url,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct TotpVerifyBody {
    email: String,
    code: String,
}

async fn totp_verify(
    State(state): State<AppState>,
    Json(body): Json<TotpVerifyBody>,
) -> impl IntoResponse {
    // load user and secret
    let mut stmt = match state.db.conn.prepare("SELECT id, totp_secret FROM users WHERE email = ?1") {
        Ok(s) => s,
        Err(e) => {
            error!("db error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };
    let mut rows = match stmt.query(rusqlite::params![body.email]) {
        Ok(r) => r,
        Err(e) => {
            error!("query failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };
    if let Some(r) = rows.next().unwrap_or(None) {
        let user_id: String = r.get(0).unwrap();
        let secret: Option<String> = r.get(1).unwrap();
        if let Some(s) = secret {
            match totp::verify_code(&s, &body.code) {
                Ok(_) => {
                    let access = jwt::create_token(
                        &user_id,
                        &state.cfg.jwt_secret,
                        state.cfg.access_token_expiry_seconds,
                        "access",
                    )
                    .unwrap();
                    let refresh = Session::create_refresh_token(&state.db, &user_id, state.cfg.refresh_token_expiry_seconds)
                        .unwrap();
                    let refresh_jwt = jwt::create_token(
                        &refresh,
                        &state.cfg.jwt_secret,
                        state.cfg.refresh_token_expiry_seconds,
                        "refresh",
                    )
                    .unwrap();
                    let resp = AuthResponse {
                        access_token: access,
                        refresh_token: refresh_jwt,
                    };
                    return (StatusCode::OK, Json(resp)).into_response();
                }
                Err(_) => return (StatusCode::BAD_REQUEST, "invalid totp").into_response(),
            }
        } else {
            return (StatusCode::BAD_REQUEST, "totp not enrolled").into_response();
        }
    }
    (StatusCode::BAD_REQUEST, "user not found").into_response()
}

#[derive(Deserialize)]
struct RefreshBody {
    refresh_token: String,
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> impl IntoResponse {
    // verify JWT of refresh token
    match jwt::verify_token(&body.refresh_token, &state.cfg.jwt_secret) {
        Ok(claims) => {
            if claims.kind != "refresh" {
                return (StatusCode::BAD_REQUEST, "invalid token kind").into_response();
            }
            let raw_refresh = claims.sub;
            // validate session store
            match Session::validate_refresh_token(&state.db, &raw_refresh) {
                Ok(user_id) => {
                    let access = jwt::create_token(
                        &user_id,
                        &state.cfg.jwt_secret,
                        state.cfg.access_token_expiry_seconds,
                        "access",
                    )
                    .unwrap();
                    let refresh = Session::create_refresh_token(&state.db, &user_id, state.cfg.refresh_token_expiry_seconds)
                        .unwrap();
                    let refresh_jwt = jwt::create_token(
                        &refresh,
                        &state.cfg.jwt_secret,
                        state.cfg.refresh_token_expiry_seconds,
                        "refresh",
                    )
                    .unwrap();
                    let resp = AuthResponse {
                        access_token: access,
                        refresh_token: refresh_jwt,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(_) => (StatusCode::UNAUTHORIZED, "invalid refresh").into_response(),
            }
        }
        Err(e) => {
            error!("refresh token verify failed: {}", e);
            (StatusCode::BAD_REQUEST, "invalid token").into_response()
        }
    }
}

#[derive(Deserialize)]
struct WebauthnRegisterOptionsBody {
    email: String,
}

async fn webauthn_register_options(
    State(state): State<AppState>,
    Json(body): Json<WebauthnRegisterOptionsBody>,
) -> impl IntoResponse {
    let user_id = match state.db.get_or_create_user(&body.email) {
        Ok(id) => id,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response(),
    };
    match state
        .webauthn
        .start_registration(&state.db, &user_id, &body.email)
    {
        Ok(opts) => (StatusCode::OK, Json(opts)).into_response(),
        Err(e) => {
            error!("webauthn start reg error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
        }
    }
}

#[derive(Deserialize)]
struct WebauthnRegisterCompleteBody {
    pending_id: String,
    response: serde_json::Value,
}

async fn webauthn_register_complete(
    State(state): State<AppState>,
    Json(body): Json<WebauthnRegisterCompleteBody>,
) -> impl IntoResponse {
    match state
        .webauthn
        .finish_registration(&state.db, &body.pending_id, body.response.clone())
    {
        Ok(_) => (StatusCode::OK, "registered").into_response(),
        Err(e) => {
            error!("reg complete failed: {:?}", e);
            (StatusCode::BAD_REQUEST, "failed").into_response()
        }
    }
}

#[derive(Deserialize)]
struct WebauthnLoginOptionsBody {
    email: String,
}

async fn webauthn_login_options(
    State(state): State<AppState>,
    Json(body): Json<WebauthnLoginOptionsBody>,
) -> impl IntoResponse {
    // need user id
    let mut stmt = match state
        .db
        .conn
        .prepare("SELECT id FROM users WHERE email = ?1")
    {
        Ok(s) => s,
        Err(e) => {
            error!("db error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };
    let mut rows = stmt.query(rusqlite::params![body.email]).unwrap();
    if let Some(r) = rows.next().unwrap_or(None) {
        let user_id: String = r.get(0).unwrap();
        match state.webauthn.start_login(&state.db, &user_id) {
            Ok(opts) => (StatusCode::OK, Json(opts)).into_response(),
            Err(e) => {
                error!("webauthn start login error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, "user not found").into_response()
    }
}

#[derive(Deserialize)]
struct WebauthnLoginCompleteBody {
    pending_id: String,
    response: serde_json::Value,
}

async fn webauthn_login_complete(
    State(state): State<AppState>,
    Json(body): Json<WebauthnLoginCompleteBody>,
) -> impl IntoResponse {
    match state
        .webauthn
        .finish_login(&state.db, &body.pending_id, body.response.clone())
    {
        Ok(user_id) => {
            let access = jwt::create_token(
                &user_id,
                &state.cfg.jwt_secret,
                state.cfg.access_token_expiry_seconds,
                "access",
            )
            .unwrap();
            let refresh = Session::create_refresh_token(&state.db, &user_id, state.cfg.refresh_token_expiry_seconds)
                .unwrap();
            let refresh_jwt = jwt::create_token(
                &refresh,
                &state.cfg.jwt_secret,
                state.cfg.refresh_token_expiry_seconds,
                "refresh",
            )
            .unwrap();
            let resp = AuthResponse {
                access_token: access,
                refresh_token: refresh_jwt,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            error!("webauthn login complete failed: {:?}", e);
            (StatusCode::BAD_REQUEST, "failed").into_response()
        }
    }
}
