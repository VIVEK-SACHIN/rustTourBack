//! Per-IP rate limit for `/api` — Natours: 100 requests / hour.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter};

use crate::utils::error::AppError;

pub type ApiRateLimiter = Arc<
    RateLimiter<
        String,
        governor::state::keyed::DefaultKeyedStateStore<String>,
        governor::clock::DefaultClock,
    >,
>;

pub fn build_api_rate_limiter() -> ApiRateLimiter {
    let quota = Quota::per_hour(NonZeroU32::new(100).expect("100 > 0"));
    Arc::new(RateLimiter::keyed(quota))
}

pub async fn rate_limit_middleware(
    State(limiter): State<ApiRateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let ip = addr.ip().to_string();
    if limiter.check_key(&ip).is_err() {
        return Err(AppError::bad_request(
            "Too many requests from this IP, please try again in an hour!",
        ));
    }
    Ok(next.run(request).await)
}
