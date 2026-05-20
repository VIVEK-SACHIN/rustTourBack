use axum::{extract::State, Json};
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde_json::json;

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::models::user::User;

/// Fetch all users from the users collection
pub async fn get_all_users(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.client.database("natours");
    let users_collection = db.collection::<User>("users");

    let cursor = users_collection
        .find(doc! {})
        .projection(doc! {
            "password": 0,
            "passwordConfirm": 0,
            "passwordResetToken": 0,
            "passwordResetTokenexpires": 0
        })
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
