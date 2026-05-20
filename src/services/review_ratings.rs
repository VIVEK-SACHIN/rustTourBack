//! Natours `reviewSchema.statics.calcAverageRatings` — updates tour rating fields.

use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::Client;

use crate::utils::error::AppError;

/// Recompute `ratingsQuantity` and `ratingsAverage` on a tour from its reviews.
pub async fn calc_average_ratings(client: &Client, tour_id: ObjectId) -> Result<(), AppError> {
    let db = client.database("natours");
    let reviews = db.collection::<Document>("reviews");

    let pipeline = vec![
        doc! { "$match": { "tour": tour_id } },
        doc! {
            "$group": {
                "_id": "$tour",
                "nRating": { "$sum": 1 },
                "avgRating": { "$avg": "$rating" }
            }
        },
    ];

    let cursor = reviews.aggregate(pipeline).await.map_err(AppError::from)?;
    let stats: Vec<Document> = cursor.try_collect().await.map_err(AppError::from)?;

    let tours = db.collection::<Document>("tours");

    if let Some(row) = stats.first() {
        let n = row.get("nRating").and_then(|b| b.as_i32()).unwrap_or(0);
        let avg = row
            .get("avgRating")
            .and_then(|b| b.as_f64())
            .unwrap_or(4.5);
        tours
            .update_one(
                doc! { "_id": tour_id },
                doc! { "$set": { "ratingsQuantity": n, "ratingsAverage": avg } },
            )
            .await
            .map_err(AppError::from)?;
    } else {
        tours
            .update_one(
                doc! { "_id": tour_id },
                doc! { "$set": { "ratingsQuantity": 0, "ratingsAverage": 4.5 } },
            )
            .await
            .map_err(AppError::from)?;
    }

    Ok(())
}
