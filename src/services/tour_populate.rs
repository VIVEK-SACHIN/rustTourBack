//! Populate tour `guides` on list queries (Natours tour `pre(/^find/)` hook).

use std::collections::{HashMap, HashSet};

use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::Collection;
use serde_json::{json, Value};

use crate::models::factory_model::FactoryModel;
use crate::models::tour::Tour;
use crate::models::user::User;
use crate::services::review_populate::UserSummary;
use crate::state::AppState;
use crate::utils::api_features::ApiFeatures;
use crate::utils::error::AppError;

pub async fn list_tours_with_guides(
    state: &AppState,
    query: &HashMap<String, String>,
) -> Result<Value, AppError> {
    let features = ApiFeatures::from_query(query, Tour::list_filter());
    let coll: Collection<Tour> = state
        .client
        .database("natours")
        .collection(Tour::collection_name());

    let mut opts = features.find_options;
    if opts.projection.is_none() {
        opts.projection = Tour::list_projection();
    }

    let cursor = coll
        .find(features.filter)
        .with_options(opts)
        .await
        .map_err(AppError::from)?;

    let tours: Vec<Tour> = cursor.try_collect().await.map_err(AppError::from)?;
    let docs = populate_guides_on_tours(state, tours).await?;

    Ok(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    }))
}

pub async fn populate_guides_on_tours(
    state: &AppState,
    tours: Vec<Tour>,
) -> Result<Vec<Value>, AppError> {
    if tours.is_empty() {
        return Ok(vec![]);
    }

    let guide_ids: HashSet<_> = tours.iter().flat_map(|t| t.guides.iter().copied()).collect();
    let map = if guide_ids.is_empty() {
        HashMap::new()
    } else {
        let ids: Vec<_> = guide_ids.into_iter().collect();
        let users_coll: Collection<User> = state.client.database("natours").collection("users");
        let users: Vec<User> = users_coll
            .find(doc! { "_id": { "$in": &ids }, "active": { "$ne": false } })
            .projection(doc! {
                "name": 1,
                "photo": 1,
                "role": 1,
                "email": 1
            })
            .await
            .map_err(AppError::from)?
            .try_collect()
            .await
            .map_err(AppError::from)?;

        users
            .into_iter()
            .filter_map(|u| u.id.map(|id| (id, UserSummary::from(u))))
            .collect()
    };

    let mut out = Vec::with_capacity(tours.len());
    for tour in tours {
        let mut val = serde_json::to_value(&tour).map_err(|e| AppError::internal(e.to_string()))?;
        if let Some(obj) = val.as_object_mut() {
            let guides: Vec<Value> = tour
                .guides
                .iter()
                .filter_map(|id| {
                    map.get(id)
                        .map(|g| serde_json::to_value(g).unwrap_or(Value::Null))
                })
                .collect();
            obj.insert("guides".to_string(), Value::Array(guides));
        }
        out.push(val);
    }
    Ok(out)
}
