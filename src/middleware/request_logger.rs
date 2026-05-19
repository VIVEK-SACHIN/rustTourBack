use axum::{
    body::to_bytes,
    extract::Request,
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Request logger middleware that logs all incoming requests
/// 
/// This middleware captures:
/// - HTTP method
/// - Request path and query parameters
/// - Request headers
/// - Request body (for methods that have bodies)
/// - Response status code
/// - Response time
pub async fn request_logger_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Collect request body bytes
    let (parts, body) = request.into_parts();
    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => axum::body::Bytes::new(),
    };
    let body_string = String::from_utf8_lossy(&body_bytes).to_string();

    // Log request details
    log_request(&method, &uri, &headers, &body_string);

    // Reconstruct the request with the collected body
    let full_body = axum::body::Body::from(body_bytes);
    let rebuilt_request = Request::from_parts(parts, full_body);

    // Call the next middleware/handler
    let response = next.run(rebuilt_request).await;

    // Log response details
    let elapsed = start_time.elapsed();
    let status = response.status();
    log_response(status, elapsed);

    response
}

fn log_request(method: &Method, uri: &Uri, headers: &HeaderMap, body: &str) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║ 📨 INCOMING REQUEST                                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    // Method and Path
    println!("Method:        {}", method);
    println!("Path:          {}", uri.path());
    
    if let Some(query) = uri.query() {
        println!("Query Params:  {}", query);
    }
    
    println!("\n--- Headers ---");
    for (key, value) in headers.iter() {
        let value_str = match value.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "[binary data]".to_string(),
        };
        // Hide sensitive headers
        if is_sensitive_header(key.as_str()) {
            println!("{}: [REDACTED]", key);
        } else {
            println!("{}: {}", key, value_str);
        }
    }
    
    if !body.is_empty() {
        println!("\n--- Body ---");
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string()));
        } else {
            println!("{}", body);
        }
    } else {
        println!("\n--- Body ---");
        println!("[Empty body]");
    }
    
    println!("────────────────────────────────────────────────────────────────\n");
}

fn log_response(status: StatusCode, elapsed: std::time::Duration) {
    let status_emoji = match status.as_u16() {
        200..=299 => "✅",
        300..=399 => "🔄",
        400..=499 => "❌",
        500..=599 => "💥",
        _ => "❓",
    };
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ 📤 RESPONSE                                                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("{} Status:       {} ({})", status_emoji, status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));
    println!("⏱️  Response Time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    println!("────────────────────────────────────────────────────────────────\n");
}

/// Check if a header should be redacted for security reasons
fn is_sensitive_header(header_name: &str) -> bool {
    matches!(
        header_name.to_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "x-api-key"
            | "x-auth-token"
            | "x-csrf-token"
            | "password"
            | "secret"
    )
}


