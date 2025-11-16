use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::warn;
use uuid::Uuid;

/// Add security headers to all responses
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HSTS - Force HTTPS for 1 year
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // XSS Protection
    headers.insert(
        HeaderValue::from_static("X-XSS-Protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Content Security Policy
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
        ),
    );

    // Referrer Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy (formerly Feature Policy)
    headers.insert(
        HeaderValue::from_static("Permissions-Policy"),
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=(), usb=()",
        ),
    );

    response
}

/// Add request ID to all requests for tracing
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add to request extensions for use in handlers
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Add to response headers
    response.headers_mut().insert(
        HeaderValue::from_static("X-Request-ID"),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

/// Request ID extension
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Validate Content-Type for POST requests
pub async fn validate_content_type(request: Request, next: Next) -> Result<Response, Response> {
    // Only validate POST/PUT/PATCH requests
    if matches!(
        request.method().as_str(),
        "POST" | "PUT" | "PATCH"
    ) {
        // Check if Content-Type is application/json
        if let Some(content_type) = request.headers().get(header::CONTENT_TYPE) {
            if !content_type
                .to_str()
                .unwrap_or("")
                .starts_with("application/json")
            {
                warn!("Invalid Content-Type: {:?}", content_type);
                return Err((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Content-Type must be application/json",
                )
                    .into_response());
            }
        } else {
            warn!("Missing Content-Type header");
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Content-Type header is required",
            )
                .into_response());
        }
    }

    Ok(next.run(request).await)
}

/// Extract IP address from request
pub fn extract_ip_address(request: &Request) -> Option<String> {
    // Try X-Forwarded-For first (for proxies)
    if let Some(forwarded_for) = request.headers().get("X-Forwarded-For") {
        if let Ok(value) = forwarded_for.to_str() {
            // Take the first IP in the list
            if let Some(ip) = value.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }

    // Fall back to connection info
    // Note: This would need ConnectInfo in the actual handler
    None
}

/// Extract User-Agent from request
pub fn extract_user_agent(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
