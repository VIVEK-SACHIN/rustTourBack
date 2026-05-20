//! User HTTP handlers — factory-backed list/get/update/delete (admin routes).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use mongodb::bson::doc;
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::error::AppError;

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
