//! Input validation mirroring key TravelAndTour Mongoose validators.

use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

use crate::models::Tour;
use crate::utils::error::AppError;

static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("valid email regex"));

pub fn validate_email(email: &str) -> Result<(), AppError> {
    let email = email.trim();
    if email.is_empty() || !EMAIL_RE.is_match(email) {
        return Err(AppError::bad_request("Please provide a valid email."));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::bad_request(
            "Password must be at least 8 characters.",
        ));
    }
    Ok(())
}

pub fn validate_review_text(review: &str) -> Result<(), AppError> {
    if review.trim().is_empty() {
        return Err(AppError::bad_request("review cannot be empty"));
    }
    Ok(())
}

pub fn validate_review_rating(rating: u8) -> Result<(), AppError> {
    if !(1..=5).contains(&rating) {
        return Err(AppError::bad_request("Rating must be between 1 and 5."));
    }
    Ok(())
}

pub fn validate_tour_create(tour: &Tour) -> Result<(), AppError> {
    if tour.name.trim().len() < 10 || tour.name.trim().len() > 40 {
        return Err(AppError::bad_request(
            "A tour name must have between 10 and 40 characters.",
        ));
    }
    if tour.duration == 0 {
        return Err(AppError::bad_request("A tour must have a duration"));
    }
    if tour.max_group_size == 0 {
        return Err(AppError::bad_request("A tour must have a group size"));
    }
    if tour.price <= 0.0 {
        return Err(AppError::bad_request("A tour must have a price"));
    }
    if tour.summary.trim().is_empty() {
        return Err(AppError::bad_request("A tour must have a description"));
    }
    if tour.image_cover.trim().is_empty() {
        return Err(AppError::bad_request("A tour must have a cover image"));
    }
    validate_price_discount(tour.price, tour.price_discount)?;
    Ok(())
}

pub fn validate_tour_update(body: &Value) -> Result<(), AppError> {
    if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
        let len = name.trim().len();
        if len < 10 || len > 40 {
            return Err(AppError::bad_request(
                "A tour name must have between 10 and 40 characters.",
            ));
        }
    }

    let price = body.get("price").and_then(|v| v.as_f64());
    let discount = body
        .get("priceDiscount")
        .or_else(|| body.get("price_discount"))
        .and_then(|v| v.as_f64());

    if let (Some(p), Some(d)) = (price, discount) {
        validate_price_discount(p, Some(d))?;
    }

    Ok(())
}

fn validate_price_discount(price: f64, discount: Option<f64>) -> Result<(), AppError> {
    if let Some(d) = discount {
        if d >= price {
            return Err(AppError::bad_request(
                "Discount price should be below regular price",
            ));
        }
    }
    Ok(())
}
