//! Review handlers — factory CRUD + `setUserIdAndTourId` (Natours `reviewController`).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use serde_json::Value;

use crate::handlers::handler_factory;
use crate::models::review::Review;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateReviewBody {
    pub review: String,
    pub rating: u8,
    /// Required on `POST /api/v1/reviews`; omitted when nested under a tour.
    pub tour: Option<String>,
}

fn parse_oid(id: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(id).map_err(|e| AppError::bad_request(format!("Invalid id: {e}")))
}

pub async fn get_all_reviews(
    state: State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_all::<Review>(state, query, None).await
}

pub async fn get_all_reviews_on_tour(
    state: State<AppState>,
    Path(tour_id): Path<String>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_all::<Review>(state, query, Some(tour_id)).await
}

pub async fn get_review(
    state: State<AppState>,
    id: Path<String>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_one::<Review>(state, id).await
}

pub async fn create_review(
    state: State<AppState>,
    Extension(user): Extension<User>,
    Json(body): Json<CreateReviewBody>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let tour_id = body
        .tour
        .as_ref()
        .ok_or_else(|| AppError::bad_request("A review must belong to a tour."))?;
    create_review_impl(state, user, body.review, body.rating, parse_oid(tour_id)?).await
}

pub async fn create_review_on_tour(
    state: State<AppState>,
    Path(tour_id): Path<String>,
    Extension(user): Extension<User>,
    Json(body): Json<CreateReviewBody>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    create_review_impl(
        state,
        user,
        body.review,
        body.rating,
        parse_oid(&tour_id)?,
    )
    .await
}

async fn create_review_impl(
    state: State<AppState>,
    user: User,
    review_text: String,
    rating: u8,
    tour: ObjectId,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;

    let review = Review {
        id: None,
        review: review_text,
        rating: rating.clamp(1, 5),
        created_at: Utc::now(),
        user: user_id,
        tour,
    };

    handler_factory::create_one::<Review>(state, Json(review)).await
}

pub async fn update_review(
    state: State<AppState>,
    id: Path<String>,
    body: Json<Value>,
) -> Result<Json<Value>, AppError> {
    handler_factory::update_one::<Review>(state, id, body).await
}

pub async fn delete_review(
    state: State<AppState>,
    id: Path<String>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    handler_factory::delete_one::<Review>(state, id).await
}
