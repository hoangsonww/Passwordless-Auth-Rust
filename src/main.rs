mod config;
mod db;
mod models;
mod email;
mod jwt;
mod magic_link;
mod totp;
mod webauthn;
mod routes;
mod session;

use axum::{
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc, fs};
use tracing_subscriber::{fmt, EnvFilter};
use tracing::{info, error};
use crate::config::Config;
use crate::db::Database;
use crate::email::Emailer;
use crate::webauthn::WebauthnState;
use crate::routes::{router, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load config
    let cfg = match Config::load("config.toml") {
        Ok(c) => c,
        Err(e) => {
            error!("failed to load config: {}", e);
            std::process::exit(1);
        }
    };
    info!("config loaded");

    // Open DB and run migrations
    let db = match Database::open(&cfg.database_path) {
        Ok(d) => d,
        Err(e) => {
            error!("db open failed: {}", e);
            std::process::exit(1);
        }
    };
    let migration_sql =
        fs::read_to_string("migrations/init.sql").expect("failed to read migration SQL");
    db.migrate(&migration_sql).expect("migration failed");

    let emailer = Emailer::new(&cfg);
    let webauthn = WebauthnState::new(&cfg);

    let state = AppState {
        cfg: Arc::new(cfg.clone()),
        db: Arc::new(db),
        emailer: Arc::new(emailer),
        webauthn: Arc::new(webauthn),
    };

    let app = router(state.clone())
        .route("/", get(|| async { "ShadowVault Passwordless Auth Server" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
