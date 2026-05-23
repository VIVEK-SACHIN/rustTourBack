//! HTTP parameter pollution — TravelAndTour `hpp` (last value + whitelist arrays).

use std::collections::HashMap;

use axum::{
    body::Body,
    http::{Request, Uri},
    middleware::Next,
    response::Response,
};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

/// Fields that may repeat in the query string and become comma-separated (`$in` in filters).
const HPP_WHITELIST: &[&str] = &[
    "duration",
    "ratingsQuantity",
    "ratingsAverage",
    "maxGroupSize",
    "difficulty",
    "price",
];

const FRAGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

pub async fn hpp_middleware(mut request: Request<Body>, next: Next) -> Response {
    if let Some(query) = request.uri().query() {
        if let Some(normalized) = normalize_query(query) {
            if let Ok(uri) = rebuild_uri(request.uri(), &normalized) {
                *request.uri_mut() = uri;
            }
        }
    }
    next.run(request).await
}

fn normalize_query(query: &str) -> Option<String> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        let key = percent_decode_str(key).decode_utf8_lossy().into_owned();
        let value = percent_decode_str(value).decode_utf8_lossy().into_owned();
        grouped.entry(key).or_default().push(value);
    }

    if grouped.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    for (key, values) in grouped {
        let encoded_key = utf8_percent_encode(&key, FRAGMENT).to_string();
        let out_value = if HPP_WHITELIST.contains(&key.as_str()) && values.len() > 1 {
            values.join(",")
        } else {
            values.last().cloned().unwrap_or_default()
        };
        let encoded_val = utf8_percent_encode(&out_value, FRAGMENT).to_string();
        parts.push(format!("{encoded_key}={encoded_val}"));
    }
    parts.sort();
    Some(parts.join("&"))
}

fn rebuild_uri(original: &Uri, query: &str) -> Result<Uri, axum::http::uri::InvalidUri> {
    let path = original.path();
    let new = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    };
    new.parse()
}
