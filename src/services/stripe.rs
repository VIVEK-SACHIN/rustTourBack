//! Stripe Checkout + webhook verification (TravelAndTour `bookingController.js`).

use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use mongodb::bson::oid::ObjectId;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use sha2::Sha256;

use crate::config::AppConfig;
use crate::models::Tour;
use crate::utils::error::AppError;

type HmacSha256 = Hmac<Sha256>;

pub type CheckoutSessionResponse = Value;

/// Stripe form bodies use `application/x-www-form-urlencoded`, not JSON.
const FORM_ENCODE: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

pub async fn create_checkout_session(
    config: &AppConfig,
    tour: &Tour,
    tour_id: &ObjectId,
    user_email: &str,
) -> Result<CheckoutSessionResponse, AppError> {
    let secret = config
        .stripe_secret_key
        .as_str()
        .trim()
        .to_string();
    if secret.is_empty() {
        return Err(AppError::internal(
            "STRIPE_SECRET_KEY is not configured on the server.",
        ));
    }

    let frontend = config.frontend_url.trim_end_matches('/');
    let tour_oid = tour_id.to_hex();
    let image_url = format!(
        "{frontend}/img/tours/{}",
        tour.image_cover
    );

    let unit_amount = (tour.price * 100.0).round() as i64;
    let product_name = format!("{} Tour", tour.name);

    let form_body = build_checkout_session_form(&[
        ("mode", "payment"),
        ("payment_method_types[]", "card"),
        ("success_url", &format!("{frontend}/me?alert=booking")),
        ("cancel_url", &format!("{frontend}/tour/{}", tour.slug)),
        ("customer_email", user_email),
        ("client_reference_id", &tour_oid),
        ("line_items[0][quantity]", "1"),
        ("line_items[0][price_data][currency]", "usd"),
        ("line_items[0][price_data][unit_amount]", &unit_amount.to_string()),
        ("line_items[0][price_data][product_data][name]", &product_name),
        ("line_items[0][price_data][product_data][description]", &tour.summary),
        ("line_items[0][price_data][product_data][images][0]", &image_url),
    ]);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .header(AUTHORIZATION, format!("Bearer {secret}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_body)
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Stripe request failed: {e}")))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::internal(format!("Stripe error: {text}")));
    }

    let session: Value = res
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Invalid Stripe response: {e}")))?;

    if session.get("id").and_then(|v| v.as_str()).is_none() {
        return Err(AppError::internal("Stripe session missing id."));
    }

    Ok(session)
}

/// Verify `Stripe-Signature` and return the parsed event JSON.
pub fn verify_webhook_event(
    payload: &[u8],
    signature_header: &str,
    webhook_secret: &str,
) -> Result<Value, AppError> {
    let secret = webhook_secret.trim();
    if secret.is_empty() {
        return Err(AppError::internal(
            "STRIPE_WEBHOOK_SECRET is not configured on the server.",
        ));
    }

    let (timestamp, signatures) = parse_stripe_signature(signature_header)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::internal(e.to_string()))?
        .as_secs();
    if timestamp.abs_diff(now) > 300 {
        return Err(AppError::bad_request("Webhook timestamp too old."));
    }

    let signed_payload = format!(
        "{timestamp}.{}",
        String::from_utf8_lossy(payload)
    );
    let expected = compute_signature(secret, &signed_payload);

    if !signatures.iter().any(|sig| constant_time_eq(sig, &expected)) {
        return Err(AppError::bad_request(format!(
            "Webhook error: signature verification failed"
        )));
    }

    serde_json::from_slice(payload)
        .map_err(|e| AppError::bad_request(format!("Webhook error: {e}")))
}

fn parse_stripe_signature(header: &str) -> Result<(u64, Vec<String>), AppError> {
    let mut timestamp = None;
    let mut signatures = Vec::new();

    for part in header.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = t.parse().ok();
        } else if let Some(v1) = part.strip_prefix("v1=") {
            signatures.push(v1.to_string());
        }
    }

    let timestamp = timestamp.ok_or_else(|| {
        AppError::bad_request("Webhook error: missing timestamp in Stripe-Signature")
    })?;

    if signatures.is_empty() {
        return Err(AppError::bad_request(
            "Webhook error: missing v1 signature in Stripe-Signature",
        ));
    }

    Ok((timestamp, signatures))
}

fn compute_signature(secret: &str, signed_payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signed_payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn form_encode(value: &str) -> String {
    utf8_percent_encode(value, FORM_ENCODE).to_string()
}

fn build_checkout_session_form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{}={}", form_encode(k), form_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}
