use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use uuid::Uuid;

/// Middleware to ensure an anonymous session cookie exists
/// Story 2.1: Anonymous Session Initialization
pub async fn ensure_session(
    axum::extract::State(_key): axum::extract::State<axum_extra::extract::cookie::Key>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let cookie_name = "dnd_session";

    if let Some(_) = jar.get(cookie_name) {
        let response = next.run(request).await;
        (jar, response).into_response()
    } else {
        let session_id = Uuid::new_v4().to_string();
        
        let cookie = Cookie::build((cookie_name, session_id))
            .path("/")
            .secure(false)
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        let new_jar = jar.add(cookie);
        let response = next.run(request).await;
        (new_jar, response).into_response()
    }
}
