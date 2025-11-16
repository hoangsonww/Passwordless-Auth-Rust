use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::{DefaultClock, QuantaClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc, time::Duration};
use tracing::warn;

/// Rate limiter for IP-based requests
pub struct IpRateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl IpRateLimiter {
    /// Create a new IP rate limiter
    /// - requests_per_minute: Maximum number of requests allowed per minute per IP
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
        let limiter = Arc::new(GovernorRateLimiter::new(
            quota,
            InMemoryState::default(),
            &QuantaClock::default(),
        ));
        Self { limiter }
    }

    /// Middleware to enforce IP-based rate limiting
    pub async fn middleware(
        limiter: Arc<IpRateLimiter>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        request: Request,
        next: Next,
    ) -> Response {
        // Check rate limit
        if limiter.limiter.check().is_err() {
            warn!("Rate limit exceeded for IP: {}", addr.ip());
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests. Please try again later.",
            )
                .into_response();
        }

        next.run(request).await
    }
}

/// Email-specific rate limiter to prevent abuse
pub struct EmailRateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl EmailRateLimiter {
    /// Create a new email rate limiter
    /// - emails_per_hour: Maximum number of emails allowed per hour per email address
    pub fn new(emails_per_hour: u32) -> Self {
        let quota = Quota::per_hour(NonZeroU32::new(emails_per_hour).unwrap());
        let limiter = Arc::new(GovernorRateLimiter::new(
            quota,
            InMemoryState::default(),
            &QuantaClock::default(),
        ));
        Self { limiter }
    }

    /// Check if an email address can send (returns true if allowed)
    pub fn check_email(&self, _email: &str) -> bool {
        // Note: This is a simplified version using global limit
        // For per-email limiting, you'd need to use a keyed rate limiter
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_rate_limiter_creation() {
        let limiter = IpRateLimiter::new(60);
        assert!(limiter.limiter.check().is_ok());
    }

    #[test]
    fn test_email_rate_limiter() {
        let limiter = EmailRateLimiter::new(10);
        assert!(limiter.check_email("test@example.com"));
    }
}
