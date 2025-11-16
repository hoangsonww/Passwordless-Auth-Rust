mod admin;
mod audit;
mod config;
mod db;
mod email;
mod email_templates;
mod error;
mod jwt;
mod magic_link;
mod metrics;
mod middleware;
mod models;
mod rate_limit;
mod routes;
mod session;
mod totp;
mod webauthn;
mod webhooks;

use axum::{middleware as axum_middleware, routing::get, Router};
use std::{fs, net::SocketAddr, sync::Arc, time::SystemTime};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::admin::{admin_router, AdminState};
use crate::audit::AuditLogger;
use crate::config::Config;
use crate::db::Database;
use crate::email::Emailer;
use crate::metrics::{init_metrics, metrics_router, MetricsState};
use crate::rate_limit::IpRateLimiter;
use crate::routes::{router, AppState};
use crate::webauthn::WebauthnState;
use crate::webhooks::WebhookSender;

#[tokio::main]
async fn main() {
    // Load config first to get log level
    let cfg = match Config::load("config.toml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize structured logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&cfg.log_level)),
        )
        .init();

    info!("🚀 Starting Passwordless Auth Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from config.toml");

    // Initialize Prometheus metrics
    let prometheus_handle = if cfg.enable_metrics {
        info!("Metrics enabled");
        init_metrics()
    } else {
        info!("Metrics disabled");
        init_metrics() // Still initialize but won't expose endpoint
    };

    // Open database and run migrations
    let db = match Database::open(&cfg.database_path) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };
    info!("Database opened: {}", cfg.database_path);

    // Run migrations
    for migration_file in &["migrations/init.sql", "migrations/002_email_queue.sql", "migrations/003_production_features.sql"] {
        if let Ok(migration_sql) = fs::read_to_string(migration_file) {
            db.migrate(&migration_sql).unwrap_or_else(|e| {
                warn!("Migration {} already applied or failed: {}", migration_file, e);
            });
            info!("Applied migration: {}", migration_file);
        }
    }

    // Initialize components
    let emailer = Emailer::new(&cfg);
    let webauthn = WebauthnState::new(&cfg);
    let audit = Arc::new(AuditLogger::new());
    let webhook_sender = Arc::new(WebhookSender::new(
        cfg.webhook_url.clone(),
        cfg.webhook_secret.clone(),
    ));

    info!("Initializing rate limiter ({}req/min)", cfg.rate_limit_per_minute);
    let rate_limiter = Arc::new(IpRateLimiter::new(cfg.rate_limit_per_minute));

    // Create application state
    let app_state = AppState {
        cfg: Arc::new(cfg.clone()),
        db: Arc::new(db),
        emailer: Arc::new(emailer),
        webauthn: Arc::new(webauthn),
        audit: audit.clone(),
        webhook: webhook_sender,
    };

    // Create metrics state
    let metrics_state = MetricsState {
        start_time: SystemTime::now(),
        prometheus_handle,
    };

    // Create admin state
    let admin_state = AdminState {
        db: app_state.db.clone(),
        audit: audit.clone(),
    };

    // Configure CORS
    let cors = if cfg.cors_allow_all {
        info!("CORS: Allowing all origins");
        CorsLayer::permissive()
    } else if !cfg.cors_allowed_origins.is_empty() {
        info!("CORS: Allowing origins: {:?}", cfg.cors_allowed_origins);
        CorsLayer::new()
            .allow_origin(
                cfg.cors_allowed_origins
                    .iter()
                    .map(|o| o.parse().unwrap())
                    .collect::<Vec<_>>(),
            )
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        info!("CORS: Disabled");
        CorsLayer::new()
    };

    // Build main application router
    let app = Router::new()
        .route("/", get(|| async {
            format!("Passwordless Auth Server v{} - Production Ready 🔒", env!("CARGO_PKG_VERSION"))
        }))
        // Auth routes
        .merge(router(app_state.clone()))
        // Admin routes (prefixed with /admin)
        .nest("/admin", admin_router(admin_state))
        // Metrics and health routes
        .merge(metrics_router(metrics_state))
        // Apply middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors)
                .layer(axum_middleware::from_fn(middleware::security_headers))
                .layer(axum_middleware::from_fn(middleware::request_id)),
        );

    // Bind server
    let addr = SocketAddr::from((
        cfg.server_host.parse::<std::net::IpAddr>().unwrap_or_else(|_| {
            warn!("Invalid server host, using 0.0.0.0");
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
        }),
        cfg.server_port,
    ));

    info!("🎧 Server listening on http://{}", addr);
    info!("📊 Health check: http://{}/health", addr);
    info!("📈 Metrics: http://{}/metrics", addr);
    info!("🔧 Admin API: http://{}/admin/*", addr);

    // Create server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    info!("Server shutdown complete");
}

/// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown...");
        },
        _ = terminate => {
            info!("Received terminate signal, starting graceful shutdown...");
        },
    }
}
