//! Tour HTTP handlers — thin wrappers around [`handler_factory`] (Natours `tourController` + factory).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

use crate::handlers::handler_factory;
use crate::models::Tour;
use crate::state::AppState;
use crate::utils::error::AppError;

pub async fn get_all_tours(
    state: State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_all::<Tour>(state, query, None).await
}

pub async fn get_tour(
    state: State<AppState>,
    id: Path<String>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_one::<Tour>(state, id).await
}

pub async fn create_tour(
    state: State<AppState>,
    body: Json<Tour>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    handler_factory::create_one::<Tour>(state, body).await
}

pub async fn update_tour(
    state: State<AppState>,
    id: Path<String>,
    body: Json<Value>,
) -> Result<Json<Value>, AppError> {
    handler_factory::update_one::<Tour>(state, id, body).await
}

pub async fn delete_tour(
    state: State<AppState>,
    id: Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    handler_factory::delete_one::<Tour>(state, id).await
}
