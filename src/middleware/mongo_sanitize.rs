//! Sanitize JSON bodies and query strings against NoSQL operator injection.

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;

use crate::utils::error::AppError;
use crate::utils::sanitize::{reject_mongo_operators_in_query, sanitize_json};
use crate::utils::xss::sanitize_xss_json;

const MAX_BODY: usize = 10 * 1024;

/// Stripe signs the raw webhook body; parsing/re-serializing JSON breaks verification.
fn is_stripe_webhook(request: &Request<Body>) -> bool {
    request.method() == Method::POST && request.uri().path() == "/webhook-checkout"
}

pub async fn mongo_sanitize_middleware(mut request: Request<Body>, next: Next) -> Response {
    if is_stripe_webhook(&request) {
        return next.run(request).await;
    }

    if let Some(query) = request.uri().query() {
        if reject_mongo_operators_in_query(query).is_err() {
            return AppError::bad_request("Query string contains forbidden characters.")
                .into_response();
        }
    }

    let is_json_body = matches!(
        request.method(),
        &Method::POST | &Method::PATCH | &Method::PUT
    ) && request
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);

    if is_json_body {
        let body = std::mem::take(request.body_mut());
        let bytes = match to_bytes(body, MAX_BODY).await {
            Ok(b) => b,
            Err(_) => {
                return AppError::bad_request("Could not read request body.").into_response();
            }
        };

        if !bytes.is_empty() {
            match serde_json::from_slice::<Value>(&bytes) {
                Ok(mut value) => {
                    sanitize_json(&mut value);
                    sanitize_xss_json(&mut value);
                    match serde_json::to_vec(&value) {
                        Ok(vec) => {
                            *request.body_mut() = Body::from(vec);
                        }
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Could not serialize body.",
                            )
                                .into_response();
                        }
                    }
                }
                Err(_) => {
                    // Non-JSON or invalid JSON — let the route handler reject it.
                    *request.body_mut() = Body::from(bytes);
                }
            }
        }
    }

    next.run(request).await
}
