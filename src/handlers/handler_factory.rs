//! Generic CRUD handlers — port of Natours `handlerFactory.js`.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, to_document, Document};
use mongodb::bson::Bson;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde_json::{json, Value};

use crate::models::factory_model::FactoryModel;
use crate::state::AppState;
use crate::utils::api_features::ApiFeatures;
use crate::utils::error::AppError;

fn parse_object_id(id: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(id).map_err(|e| AppError::bad_request(format!("Invalid id: {e}")))
}

fn collection<T: FactoryModel>(
    state: &AppState,
) -> mongodb::Collection<T> {
    state
        .client
        .database("natours")
        .collection(T::collection_name())
}

/// `getAll` — optional `tour_id` for nested review routes (`req.params.tourId`).
pub async fn get_all<T: FactoryModel>(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    tour_id: Option<String>,
) -> Result<Json<Value>, AppError> {
    let mut base = T::list_filter();
    if let Some(tid) = tour_id {
        base.insert("tour", Bson::ObjectId(parse_object_id(&tid)?));
    }

    let features = ApiFeatures::from_query(&query, base);
    let coll = collection::<T>(&state);

    let mut opts = features.find_options;
    if opts.projection.is_none() {
        opts.projection = T::list_projection();
    }

    let cursor = coll
        .find(features.filter)
        .with_options(opts)
        .await
        .map_err(AppError::from)?;

    let docs: Vec<T> = cursor.try_collect().await.map_err(AppError::from)?;

    Ok(Json(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    })))
}

/// `getOne` — `populate` is not implemented yet (Mongoose virtual populate for reviews).
pub async fn get_one<T: FactoryModel>(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let oid = parse_object_id(&id)?;
    let coll = collection::<T>(&state);

    let filter = {
        let mut f = doc! { "_id": oid };
        for (k, v) in T::list_filter() {
            f.insert(k, v);
        }
        f
    };

    let doc = coll
        .find_one(filter)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No Document found with that ID"))?;

    Ok(Json(json!({
        "status": "success",
        "data": { "doc": doc }
    })))
}

/// `createOne`
pub async fn create_one<T: FactoryModel>(
    State(state): State<AppState>,
    Json(mut body): Json<T>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    body.prepare_create();
    let coll = collection::<T>(&state);
    let result = coll.insert_one(&body).await.map_err(AppError::from)?;
    if let Some(id) = result.inserted_id.as_object_id() {
        body.set_id(id);
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "status": "success",
            "data": { "doc": body }
        })),
    ))
}

/// `updateOne` — partial JSON body merged with `$set` (like `findByIdAndUpdate` + `req.body`).
pub async fn update_one<T: FactoryModel>(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let oid = parse_object_id(&id)?;
    let mut set_doc: Document = to_document(&body)
        .map_err(|e| AppError::bad_request(format!("Invalid update data: {e}")))?;
    set_doc.remove("_id");
    set_doc.remove("id");

    if set_doc.is_empty() {
        return Err(AppError::bad_request("No valid fields to update."));
    }

    let coll = collection::<T>(&state);
    let opts = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let updated = coll
        .find_one_and_update(doc! { "_id": oid }, doc! { "$set": set_doc })
        .with_options(opts)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No document found with that ID"))?;

    Ok(Json(json!({
        "status": "success",
        "data": { "doc": updated }
    })))
}

/// `deleteOne`
pub async fn delete_one<T: FactoryModel>(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let oid = parse_object_id(&id)?;
    let coll = collection::<T>(&state);

    let result = coll
        .delete_one(doc! { "_id": oid })
        .await
        .map_err(AppError::from)?;

    if result.deleted_count == 0 {
        return Err(AppError::not_found("No document  found with that ID"));
    }

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "status": "success",
            "data": null
        })),
    ))
}
