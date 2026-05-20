use axum::{
    extract::State,
    Json,
};
use serde_json::json;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::models::Tour;
use futures::TryStreamExt;

/// Fetch all tours from the tours collection
pub async fn get_all_tours(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.client.database("natours");
    let tours_collection = db.collection::<Tour>("tours");

    let cursor = tours_collection
        .find(mongodb::bson::doc! {})
        .await
        .map_err(AppError::from)?;

    let tours: Vec<Tour> = cursor
        .try_collect()
        .await
        .map_err(|e| {
            eprintln!("❌ Database collection error: {}", e);
            AppError::internal(format!("Failed to collect tours: {}", e))
        })?;

    println!("✅ Fetched {} tours from database", tours.len());

    Ok(Json(json!({
        "status": "success",
        "data": {
            "tours": tours
        }
    })))
}

