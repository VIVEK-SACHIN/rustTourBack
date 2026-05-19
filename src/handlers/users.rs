use axum::{extract::State, Json};
use futures::TryStreamExt;
use mongodb::Client;
use serde_json::json;

use crate::error::AppError;
use crate::models::user::User;

/// Fetch all users from the users collection
pub async fn get_all_users(
    State(client): State<Client>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = client.database("natours");
    let users_collection = db.collection::<User>("users");

    let cursor = users_collection
        .find(mongodb::bson::doc! {})
        .await
        .map_err(AppError::from)?;

    let users: Vec<User> = cursor
        .try_collect()
        .await
        .map_err(|e| {
            eprintln!("❌ Database collection error: {}", e);
            AppError::internal(format!("Failed to collect users: {}", e))
        })?;

    println!("✅ Fetched {} users from database", users.len());

    Ok(Json(json!({
        "status": "success",
        "data": {
            "users": users
        }
    })))
}
