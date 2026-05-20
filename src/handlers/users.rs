//! User HTTP handlers — factory-backed list/get/update/delete (admin routes).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

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
