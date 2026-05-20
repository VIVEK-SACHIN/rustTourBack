use axum::{
    body::Body,
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use mongodb::bson::doc;

use crate::models::user::User;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::jwt_util::JwtClaims;

/// After this middleware runs, handlers may use `Extension<User>` for the current user (password cleared).
pub async fn protect(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&jar, &headers).ok_or_else(|| {
        AppError::unauthorized("You are not logged in! Please log in to get access.")
    })?;

    let secret = state.config.jwt_secret.as_bytes();
    if secret.is_empty() {
        return Err(AppError::internal(
            "JWT_SECRET is not configured on the server.",
        ));
    }

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map_err(AppError::from)?;

    let id = mongodb::bson::oid::ObjectId::parse_str(&token_data.claims.id)
        .map_err(|_| AppError::unauthorized("Invalid token. Please log in again!"))?;

    let db = state.client.database("natours");
    let users = db.collection::<User>("users");

    let filter = doc! {
        "_id": id,
        "active": { "$ne": false }
    };
    let mut user = users
        .find_one(filter)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| {
            AppError::unauthorized("The user belonging to this token does no longer exist.")
        })?;

    if user.changed_password_after(token_data.claims.iat) {
        return Err(AppError::unauthorized(
            "User recently changed password! Please log in again.",
        ));
    }

    user.password = None;
    user.password_confirm = None;

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

fn extract_token(jar: &CookieJar, headers: &HeaderMap) -> Option<String> {
    if let Some(h) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        let h = h.trim();
        if let Some(rest) = h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")) {
            let t = rest.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    jar.get("jwt").and_then(|c| {
        let v = c.value();
        if v.is_empty() || v == "loggedout" {
            None
        } else {
            Some(v.to_string())
        }
    })
}
