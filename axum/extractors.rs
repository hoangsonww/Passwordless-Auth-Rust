use axum::middleware;
use axum::BoxError;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

/// Compose common middleware layers: tracing, timeout, etc.
pub fn common_layers() -> ServiceBuilder<impl Clone> {
    ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_request(|_request: &Request<_>, _span: &tracing::Span| {
                    // could add custom logging here
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &tracing::Span| {
                    // optional hooks
                }),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(middleware::from_fn(rate_limit))
}

/// Simple in-memory rate limiter stub (per-request global, placeholder).
async fn rate_limit<B>(
    req: Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<Response, BoxError> {
    // Placeholder: could integrate token bucket / leaky bucket here.
    // For now pass through.
    Ok(next.run(req).await)
}
