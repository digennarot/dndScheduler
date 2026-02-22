use axum::{
    body::Body,
    http::{HeaderValue, Request, Response},
    middleware::Next,
};

/// Security headers middleware
/// Implements OWASP security best practices
pub async fn security_headers(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    // Enable XSS protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Enforce HTTPS (when deployed)
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdnjs.cloudflare.com https://cdn.jsdelivr.net https://fonts.googleapis.com https://accounts.google.com; \
             style-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdn.jsdelivr.net https://fonts.googleapis.com https://accounts.google.com; \
             font-src 'self' https://fonts.gstatic.com; \
             img-src 'self' data: https:; \
             connect-src 'self' https://accounts.google.com; \
             frame-src 'self' https://accounts.google.com; \
             frame-ancestors 'none';"
        ),
    );

    // Referrer Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy (formerly Feature Policy)
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    response
}

/// Middleware to convert 429 Too Many Requests responses to JSON
/// This ensures frontend clients always receive JSON even when rate limited by tower_governor
pub async fn handle_429_json(request: Request<Body>, next: Next) -> Response<Body> {
    let response = next.run(request).await;

    if response.status() == axum::http::StatusCode::TOO_MANY_REQUESTS {
        let (parts, _body) = response.into_parts();
        let body = Body::from(r#"{"error": "Too many requests. Please try again later."}"#);

        let mut response = Response::from_parts(parts, body);
        response
            .headers_mut()
            .remove(axum::http::header::CONTENT_LENGTH);
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        return response;
    }

    response
}

/// CORS configuration
/// Restrict to specific origins in production
pub fn get_cors_layer() -> tower_http::cors::CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::CorsLayer;

    // Allow origins from environment variable or fallback to localhost for development
    let allowed_origins: Vec<HeaderValue> = match std::env::var("ALLOWED_ORIGINS") {
        Ok(val) => val
            .split(',')
            .map(|s| s.trim())
            .filter_map(|s| match s.parse::<HeaderValue>() {
                Ok(v) => Some(v),
                Err(_) => {
                    tracing::warn!("Invalid CORS origin configured and ignored: {}", s);
                    None
                }
            })
            .collect(),
        Err(_) => {
            tracing::info!("ALLOWED_ORIGINS not set, falling back to development defaults");
            vec![
                HeaderValue::from_static("http://localhost:3000"),
                HeaderValue::from_static("http://127.0.0.1:3000"),
            ]
        }
    };

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}
