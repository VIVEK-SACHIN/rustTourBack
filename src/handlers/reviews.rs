//! Review handlers — factory CRUD + `setUserIdAndTourId` + rating aggregates + user populate.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use axum::extract::State as AxumState;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::review::Review;
use crate::models::Tour;
use crate::models::user::{User, UserRole};
use crate::services::review_populate::{get_review_populated, list_reviews_populated};
use crate::services::review_ratings::calc_average_ratings;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::validate::{validate_review_rating, validate_review_text};

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
    let json = list_reviews_populated(&state, &query, None).await?;
    Ok(Json(json))
}

/// `GET /api/v1/reviews/my` (protected) — current user's reviews with tour summary.
pub async fn get_my_reviews(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User missing _id"))?;

    let db = state.client.database("natours");
    let reviews_coll = db.collection::<Review>("reviews");
    let tours_coll = db.collection::<Tour>("tours");

    let cursor = reviews_coll
        .find(doc! { "user": user_id })
        .await
        .map_err(AppError::from)?;

    let reviews: Vec<Review> = cursor.try_collect().await.map_err(AppError::from)?;

    let mut docs = Vec::with_capacity(reviews.len());
    for review in reviews {
        let Some(review_id) = review.id else {
            continue;
        };

        let Some(tour) = tours_coll
            .find_one(doc! { "_id": review.tour })
            .await
            .map_err(AppError::from)?
        else {
            continue;
        };

        docs.push(json!({
            "_id": review_id,
            "review": review.review,
            "rating": review.rating,
            "createdAt": review.created_at,
            "tour": {
                "_id": tour.id,
                "name": tour.name,
                "slug": tour.slug,
                "imageCover": tour.image_cover,
            }
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    })))
}

pub async fn get_all_reviews_on_tour(
    state: State<AppState>,
    Path(tour_id): Path<String>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let json = list_reviews_populated(&state, &query, Some(&tour_id)).await?;
    Ok(Json(json))
}

pub async fn get_review(
    state: State<AppState>,
    id: Path<String>,
) -> Result<Json<Value>, AppError> {
    let json = get_review_populated(&state, &id).await?;
    Ok(Json(json))
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
    validate_review_text(&review_text)?;
    validate_review_rating(rating)?;

    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;

    let review = Review {
        id: None,
        review: review_text,
        rating,
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
    Extension(user): Extension<User>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if let Some(text) = body.get("review").and_then(|v| v.as_str()) {
        validate_review_text(text)?;
    }
    if let Some(r) = body.get("rating").and_then(|v| v.as_u64()) {
        validate_review_rating(r as u8)?;
    }

    let oid = parse_oid(&id)?;
    let existing = load_review(&state, oid).await?;
    assert_review_owner(&user, &existing)?;

    let tour_id = existing.tour;
    let client = state.client.clone();
    let resp =
        handler_factory::update_one::<Review>(AxumState(state), Path(id), Json(body)).await?;
    calc_average_ratings(&client, tour_id).await?;
    Ok(resp)
}

pub async fn delete_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(user): Extension<User>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let oid = parse_oid(&id)?;
    let existing = load_review(&state, oid).await?;
    assert_review_owner(&user, &existing)?;

    let tour_id = existing.tour;
    let client = state.client.clone();
    let resp = handler_factory::delete_one::<Review>(AxumState(state), Path(id)).await?;
    calc_average_ratings(&client, tour_id).await?;
    Ok(resp)
}

async fn load_review(state: &AppState, review_id: ObjectId) -> Result<Review, AppError> {
    let db = state.client.database("natours");
    let reviews = db.collection::<Review>("reviews");
    reviews
        .find_one(doc! { "_id": review_id })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No document found with that ID"))
}

fn assert_review_owner(user: &User, review: &Review) -> Result<(), AppError> {
    if user.role == UserRole::Admin {
        return Ok(());
    }
    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;
    if user.role == UserRole::User && review.user == user_id {
        return Ok(());
    }
    Err(AppError::forbidden(
        "You do not have permission to perform this action on this review.",
    ))
}
