//! Populate review `user` with `name` and `photo` (Natours review pre-find hook).

use std::collections::HashMap;

use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use mongodb::Collection;
use serde::Serialize;
use serde_json::{json, Value};

use crate::models::factory_model::FactoryModel;
use crate::models::review::Review;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::api_features::ApiFeatures;
use crate::utils::error::AppError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub photo: String,
}

impl From<User> for UserSummary {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            photo: u.photo,
        }
    }
}

pub async fn list_reviews_populated(
    state: &AppState,
    query: &HashMap<String, String>,
    tour_id: Option<&str>,
) -> Result<Value, AppError> {
    let mut base = Review::list_filter();
    if let Some(tid) = tour_id {
        let oid = ObjectId::parse_str(tid)
            .map_err(|e| AppError::bad_request(format!("Invalid tour id: {e}")))?;
        base.insert("tour", Bson::ObjectId(oid));
    }

    let features = ApiFeatures::from_query(query, base);
    let coll: Collection<Review> = state
        .client
        .database("natours")
        .collection(Review::collection_name());

    let cursor = coll
        .find(features.filter)
        .with_options(features.find_options)
        .await
        .map_err(AppError::from)?;

    let reviews: Vec<Review> = cursor.try_collect().await.map_err(AppError::from)?;
    let docs = populate_review_docs(state, reviews).await?;

    Ok(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    }))
}

pub async fn get_review_populated(state: &AppState, id: &str) -> Result<Value, AppError> {
    let oid = ObjectId::parse_str(id)
        .map_err(|e| AppError::bad_request(format!("Invalid id: {e}")))?;

    let coll: Collection<Review> = state
        .client
        .database("natours")
        .collection(Review::collection_name());

    let mut filter = doc! { "_id": oid };
    for (k, v) in Review::list_filter() {
        filter.insert(k, v);
    }

    let review = coll
        .find_one(filter)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No document found with that ID"))?;

    let docs = populate_review_docs(state, vec![review]).await?;
    let doc = docs
        .into_iter()
        .next()
        .ok_or_else(|| AppError::internal("Review missing after populate."))?;

    Ok(json!({
        "status": "success",
        "data": { "doc": doc }
    }))
}

async fn populate_review_docs(state: &AppState, reviews: Vec<Review>) -> Result<Vec<Value>, AppError> {
    if reviews.is_empty() {
        return Ok(vec![]);
    }

    let user_ids: Vec<ObjectId> = reviews.iter().map(|r| r.user).collect();
    let users_coll: Collection<User> = state.client.database("natours").collection("users");
    let users: Vec<User> = users_coll
        .find(doc! { "_id": { "$in": &user_ids } })
        .projection(doc! { "name": 1, "photo": 1 })
        .await
        .map_err(AppError::from)?
        .try_collect()
        .await
        .map_err(AppError::from)?;

    let map: HashMap<ObjectId, UserSummary> = users
        .into_iter()
        .filter_map(|u| u.id.map(|id| (id, UserSummary::from(u))))
        .collect();

    let mut out = Vec::with_capacity(reviews.len());
    for review in reviews {
        let user_id = review.user;
        let mut val = serde_json::to_value(&review).map_err(|e| AppError::internal(e.to_string()))?;
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "user".to_string(),
                map.get(&user_id)
                    .map(|u| serde_json::to_value(u).unwrap())
                    .unwrap_or(Value::Null),
            );
        }
        out.push(val);
    }
    Ok(out)
}
