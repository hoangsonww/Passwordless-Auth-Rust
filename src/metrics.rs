use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Initialize Prometheus metrics exporter
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

/// Track authentication metrics
pub struct MetricsRecorder;

impl MetricsRecorder {
    /// Record a successful authentication
    pub fn record_auth_success(method: &str) {
        counter!("auth_attempts_total", "method" => method, "status" => "success").increment(1);
    }

    /// Record a failed authentication
    pub fn record_auth_failure(method: &str, reason: &str) {
        counter!("auth_attempts_total", "method" => method, "status" => "failure", "reason" => reason)
            .increment(1);
    }

    /// Record email sent
    pub fn record_email_sent() {
        counter!("emails_sent_total").increment(1);
    }

    /// Record email failure
    pub fn record_email_failure() {
        counter!("emails_failed_total").increment(1);
    }

    /// Record token refresh
    pub fn record_token_refresh() {
        counter!("token_refreshes_total").increment(1);
    }

    /// Record session creation
    pub fn record_session_created() {
        counter!("sessions_created_total").increment(1);
        gauge!("active_sessions").increment(1.0);
    }

    /// Record session revocation
    pub fn record_session_revoked() {
        counter!("sessions_revoked_total").increment(1);
        gauge!("active_sessions").decrement(1.0);
    }

    /// Record rate limit hit
    pub fn record_rate_limit_hit(limit_type: &str) {
        counter!("rate_limit_hits_total", "type" => limit_type).increment(1);
    }

    /// Record HTTP request duration
    pub fn record_request_duration(method: &str, path: &str, status: u16, duration_secs: f64) {
        histogram!(
            "http_request_duration_seconds",
            "method" => method,
            "path" => path,
            "status" => status.to_string()
        )
        .record(duration_secs);
    }

    /// Record database query duration
    pub fn record_db_query_duration(query_type: &str, duration_secs: f64) {
        histogram!("db_query_duration_seconds", "type" => query_type).record(duration_secs);
    }
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: u64,
}

/// Application state for metrics
#[derive(Clone)]
pub struct MetricsState {
    pub start_time: SystemTime,
    pub prometheus_handle: PrometheusHandle,
}

/// Health check endpoint
pub async fn health_check(State(state): State<MetricsState>) -> impl IntoResponse {
    let now = SystemTime::now();
    let uptime = now
        .duration_since(state.start_time)
        .unwrap_or_default()
        .as_secs();
    let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp,
    };

    (StatusCode::OK, axum::Json(response))
}

/// Readiness check endpoint (for Kubernetes)
pub async fn readiness_check() -> impl IntoResponse {
    // In production, you might check database connectivity, etc.
    (StatusCode::OK, "ready")
}

/// Liveness check endpoint (for Kubernetes)
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

/// Prometheus metrics endpoint
pub async fn metrics_handler(State(state): State<MetricsState>) -> impl IntoResponse {
    state.prometheus_handle.render()
}

/// Create metrics router
pub fn metrics_router(state: MetricsState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/readiness", get(readiness_check))
        .route("/liveness", get(liveness_check))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}
