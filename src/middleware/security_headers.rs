//! Security response headers (Natours `helmet`, CSP disabled).

use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response},
    middleware::Next,
};

pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    set(headers, "X-Content-Type-Options", "nosniff");
    set(headers, "X-Frame-Options", "SAMEORIGIN");
    set(headers, "X-DNS-Prefetch-Control", "off");
    set(headers, "Referrer-Policy", "no-referrer");
    set(headers, "Cross-Origin-Opener-Policy", "same-origin");
    set(headers, "Cross-Origin-Resource-Policy", "same-origin");
    set(headers, "Origin-Agent-Cluster", "?1");
    set(headers, "X-Download-Options", "noopen");
    set(headers, "X-Permitted-Cross-Domain-Policies", "none");

    response
}

fn set(headers: &mut axum::http::HeaderMap, name: &'static str, value: &'static str) {
    if let Ok(v) = HeaderValue::from_str(value) {
        if let Ok(h) = header::HeaderName::from_bytes(name.as_bytes()) {
            headers.insert(h, v);
        }
    }
}
