//! User HTTP handlers — factory-backed list/get/update/delete (admin routes).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use axum_extra::extract::Multipart;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::user::User;
use crate::services::user_photo::save_user_photo;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::validate::validate_email;

pub async fn get_all_users(
    state: State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_all::<User>(state, query, None).await
}

pub async fn get_user(
    state: State<AppState>,
    id: Path<String>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_one::<User>(state, id).await
}

pub async fn update_user(
    state: State<AppState>,
    id: Path<String>,
    body: Json<Value>,
) -> Result<Json<Value>, AppError> {
    handler_factory::update_one::<User>(state, id, body).await
}

pub async fn delete_user(
    state: State<AppState>,
    id: Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    handler_factory::delete_one::<User>(state, id).await
}

/// TravelAndTour `createUser` — not for public signup.
pub async fn create_user() -> Result<(StatusCode, Json<Value>), AppError> {
    Ok((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "status": "error",
            "message": "This route is not defined! Please use /signup instead"
        })),
    ))
}

/// Soft-delete current user (`active: false`).
/// `getMe` + `getUser` — full user document for current account.
pub async fn get_me(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Value>, AppError> {
    let id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;
    handler_factory::get_one::<User>(State(state), Path(id.to_hex())).await
}

/// `PATCH /users/updateUserData` — multipart name, email, optional photo (TravelAndTour `updateMe`).
pub async fn update_user_data(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;

    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut photo_bytes: Option<Vec<u8>> = None;
    let mut photo_content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Invalid form data: {e}")))?
    {
        match field.name().unwrap_or("") {
            "name" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Invalid name field: {e}")))?;
                if !text.trim().is_empty() {
                    name = Some(text);
                }
            }
            "email" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Invalid email field: {e}")))?;
                if !text.trim().is_empty() {
                    email = Some(text);
                }
            }
            "photo" => {
                photo_content_type = field.content_type().map(|c| c.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Invalid photo field: {e}")))?;
                if !bytes.is_empty() {
                    photo_bytes = Some(bytes.to_vec());
                }
            }
            _ => {}
        }
    }

    let mut set_doc = doc! {};
    if let Some(ref n) = name {
        set_doc.insert("name", n.trim());
    }
    if let Some(ref e) = email {
        let trimmed = e.trim().to_lowercase();
        validate_email(&trimmed)?;
        set_doc.insert("email", trimmed);
    }
    if let Some(bytes) = photo_bytes {
        let filename = save_user_photo(
            &state.config.users_upload_dir,
            &id,
            &bytes,
            photo_content_type.as_deref(),
        )?;
        set_doc.insert("photo", filename);
    }

    if set_doc.is_empty() {
        return Err(AppError::bad_request("No valid fields to update."));
    }

    let db = state.db();
    let users = db.collection::<User>("users");
    let opts = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let updated = users
        .find_one_and_update(doc! { "_id": id }, doc! { "$set": set_doc })
        .with_options(opts)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    Ok(Json(json!({
        "status": "success",
        "data": { "user": updated.strip_secrets_for_response() }
    })))
}

pub async fn delete_me(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;

    let db = state.db();
    let users = db.collection::<User>("users");

    users
        .update_one(doc! { "_id": id }, doc! { "$set": { "active": false } })
        .await
        .map_err(AppError::from)?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "status": "success",
            "data": null
        })),
    ))
}
