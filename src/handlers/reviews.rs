//! Review handlers — factory CRUD + `setUserIdAndTourId` + rating aggregates.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use axum::extract::State as AxumState;
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;
use serde_json::Value;

use crate::handlers::handler_factory;
use crate::models::review::Review;
use crate::models::user::User;
use crate::services::review_ratings::calc_average_ratings;
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

    let (status, json) = handler_factory::create_one::<Review>(state.clone(), Json(review)).await?;
    calc_average_ratings(&state.client, tour).await?;
    Ok((status, json))
}

pub async fn update_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let oid = parse_oid(&id)?;
    let tour_id = review_tour_id(&state, oid).await?;
    let client = state.client.clone();
    let resp =
        handler_factory::update_one::<Review>(AxumState(state), Path(id), Json(body)).await?;
    calc_average_ratings(&client, tour_id).await?;
    Ok(resp)
}

pub async fn delete_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let oid = parse_oid(&id)?;
    let tour_id = review_tour_id(&state, oid).await?;
    let client = state.client.clone();
    let resp = handler_factory::delete_one::<Review>(AxumState(state), Path(id)).await?;
    calc_average_ratings(&client, tour_id).await?;
    Ok(resp)
}

async fn review_tour_id(state: &AppState, review_id: ObjectId) -> Result<ObjectId, AppError> {
    let db = state.client.database("natours");
    let reviews = db.collection::<Review>("reviews");
    let review = reviews
        .find_one(doc! { "_id": review_id })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No document found with that ID"))?;
    Ok(review.tour)
}
