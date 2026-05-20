//! User HTTP handlers — factory-backed list/get/update/delete (admin routes).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDataBody {
    pub name: Option<String>,
    pub email: Option<String>,
}

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

/// Natours `createUser` — not for public signup.
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

/// `updateMe` — name/email only (no password, no photo upload).
pub async fn update_user_data(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(body): Json<UpdateUserDataBody>,
) -> Result<Json<Value>, AppError> {
    let id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;

    let mut set_doc = doc! {};
    if let Some(name) = body.name {
        set_doc.insert("name", name.trim());
    }
    if let Some(email) = body.email {
        set_doc.insert("email", email.trim().to_lowercase());
    }

    if set_doc.is_empty() {
        return Err(AppError::bad_request("No valid fields to update."));
    }

    let db = state.client.database("natours");
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

    let db = state.client.database("natours");
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
