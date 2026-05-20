//! Tour HTTP handlers — factory CRUD + Natours `tourController` specials.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::review::Review;
use crate::models::user::User;
use crate::models::tour::slugify;
use crate::models::Tour;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::validate::{validate_tour_create, validate_tour_update};

fn parse_oid(id: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(id).map_err(|e| AppError::bad_request(format!("Invalid id: {e}")))
}

pub async fn get_all_tours(
    state: State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    handler_factory::get_all::<Tour>(state, query, None).await
}

/// `aliasTopTours` + `getAllTours`
pub async fn get_top_5_cheap(state: State<AppState>) -> Result<Json<Value>, AppError> {
    let mut query = HashMap::new();
    query.insert("limit".into(), "5".into());
    query.insert("sort".into(), "-ratingsAverage,price".into());
    query.insert(
        "fields".into(),
        "name,price,ratingsAverage,summary,difficulty".into(),
    );
    handler_factory::get_all::<Tour>(state, Query(query), None).await
}

/// `getTour` with reviews + guides populated.
pub async fn get_tour(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let oid = parse_oid(&id)?;
    let db = state.client.database("natours");
    let tours = db.collection::<Tour>("tours");

    let tour = tours
        .find_one(doc! { "_id": oid, "secretTour": { "$ne": true } })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No Document found with that ID"))?;

    let reviews_coll = db.collection::<Review>("reviews");
    let reviews: Vec<Review> = reviews_coll
        .find(doc! { "tour": oid })
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    let mut guides: Vec<User> = Vec::new();
    if !tour.guides.is_empty() {
        let users = db.collection::<User>("users");
        guides = users
            .find(doc! { "_id": { "$in": &tour.guides }, "active": { "$ne": false } })
            .projection(doc! {
                "password": 0,
                "passwordConfirm": 0,
                "passwordResetToken": 0,
                "passwordResetTokenexpires": 0
            })
            .await
            .map_err(AppError::from)?
            .try_collect()
            .await
            .map_err(AppError::from)?;
    }

    let mut doc = serde_json::to_value(&tour).map_err(|e| AppError::internal(e.to_string()))?;
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("reviews".to_string(), serde_json::to_value(&reviews).unwrap());
        obj.insert("guides".to_string(), serde_json::to_value(&guides).unwrap());
    }

    Ok(Json(json!({
        "status": "success",
        "data": { "doc": doc }
    })))
}

pub async fn get_tour_stats(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let db = state.client.database("natours");
    let tours = db.collection::<Document>("tours");

    let pipeline = vec![
        doc! { "$match": { "ratingsAverage": { "$gte": 4.5 } } },
        doc! {
            "$group": {
                "_id": { "$toUpper": "$difficulty" },
                "numTours": { "$sum": 1 },
                "numRatings": { "$sum": "$ratingsQuantity" },
                "avgRating": { "$avg": "$ratingsAverage" },
                "avgPrice": { "$avg": "$price" },
                "minPrice": { "$min": "$price" },
                "maxPrice": { "$max": "$price" }
            }
        },
        doc! { "$sort": { "avgPrice": 1 } },
    ];

    let stats: Vec<Document> = tours
        .aggregate(pipeline)
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "status": "success",
        "data": { "stats": stats }
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct MonthlyPlanParams {
    pub year: String,
}

pub async fn get_monthly_plan(
    State(state): State<AppState>,
    Path(params): Path<MonthlyPlanParams>,
) -> Result<Json<Value>, AppError> {
    let year: i32 = params
        .year
        .parse()
        .map_err(|_| AppError::bad_request("Invalid year"))?;

    let db = state.client.database("natours");
    let tours = db.collection::<Document>("tours");

    let pipeline = vec![
        doc! { "$unwind": "$startDates" },
        doc! {
            "$match": {
                "startDates": {
                    "$gte": format!("{year}-01-01"),
                    "$lte": format!("{year}-12-31")
                }
            }
        },
        doc! {
            "$group": {
                "_id": { "$month": "$startDates" },
                "numTourStarts": { "$sum": 1 },
                "tours": { "$push": "$name" }
            }
        },
        doc! { "$addFields": { "month": "$_id" } },
        doc! { "$project": { "_id": 0 } },
        doc! { "$sort": { "numTourStarts": -1 } },
        doc! { "$limit": 12 },
    ];

    let plan: Vec<Document> = tours
        .aggregate(pipeline)
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "status": "success",
        "data": { "plan": plan }
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct ToursWithinParams {
    pub distance: String,
    pub latlng: String,
    pub unit: String,
}

pub async fn get_tours_within(
    State(state): State<AppState>,
    Path(p): Path<ToursWithinParams>,
) -> Result<Json<Value>, AppError> {
    let (lat, lng) = parse_latlng(&p.latlng)?;
    let distance: f64 = p
        .distance
        .parse()
        .map_err(|_| AppError::bad_request("Invalid distance"))?;
    let radius = if p.unit == "mi" {
        distance / 3963.2
    } else {
        distance / 6378.1
    };

    let db = state.client.database("natours");
    let tours = db.collection::<Tour>("tours");

    let filter = doc! {
        "secretTour": { "$ne": true },
        "startLocation": {
            "$geoWithin": {
                "$centerSphere": [ [lng, lat], radius ]
            }
        }
    };

    let list: Vec<Tour> = tours
        .find(filter)
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "status": "success",
        "results": list.len(),
        "data": { "tours": list }
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct DistancesParams {
    pub latlng: String,
    pub unit: String,
}

pub async fn get_distances(
    State(state): State<AppState>,
    Path(p): Path<DistancesParams>,
) -> Result<Json<Value>, AppError> {
    let (lat, lng) = parse_latlng(&p.latlng)?;
    let multiplier = if p.unit == "mi" { 0.000621371 } else { 0.001 };

    let db = state.client.database("natours");
    let tours = db.collection::<Document>("tours");

    let pipeline = vec![
        doc! {
            "$geoNear": {
                "near": { "type": "Point", "coordinates": [lng, lat] },
                "distanceField": "distance",
                "distanceMultiplier": multiplier,
                "query": { "secretTour": { "$ne": true } }
            }
        },
        doc! { "$project": { "distance": 1, "name": 1 } },
    ];

    let distances: Vec<Document> = tours
        .aggregate(pipeline)
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "status": "success",
        "data": { "data": distances }
    })))
}

fn parse_latlng(latlng: &str) -> Result<(f64, f64), AppError> {
    let mut parts = latlng.split(',');
    let lat: f64 = parts
        .next()
        .ok_or_else(|| AppError::bad_request("please specify latitude and longitude"))?
        .trim()
        .parse()
        .map_err(|_| AppError::bad_request("Invalid latitude"))?;
    let lng: f64 = parts
        .next()
        .ok_or_else(|| AppError::bad_request("please specify latitude and longitude"))?
        .trim()
        .parse()
        .map_err(|_| AppError::bad_request("Invalid longitude"))?;
    Ok((lat, lng))
}

pub async fn create_tour(
    state: State<AppState>,
    body: Json<Tour>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    validate_tour_create(&body)?;
    handler_factory::create_one::<Tour>(state, body).await
}

pub async fn update_tour(
    state: State<AppState>,
    id: Path<String>,
    Json(mut body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    validate_tour_update(&body)?;
    if let Some(name) = body.get("name").and_then(|v| v.as_str()).map(str::to_string) {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("slug".to_string(), json!(slugify(&name)));
        }
    }
    handler_factory::update_one::<Tour>(state, id, Json(body)).await
}

pub async fn delete_tour(
    state: State<AppState>,
    id: Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    handler_factory::delete_one::<Tour>(state, id).await
}
