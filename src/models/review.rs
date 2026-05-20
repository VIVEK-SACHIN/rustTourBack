use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub review: String,
    pub rating: u8,
    #[serde(rename = "CreatedAt", with = "crate::models::bson_chrono::required")]
    pub created_at: DateTime<Utc>,
    pub user: String, // ObjectId as string
    pub tour: String, // ObjectId as string
}

impl Default for Review {
    fn default() -> Self {
        Self {
            review: String::new(),
            rating: 5,
            created_at: Utc::now(),
            user: String::new(),
            tour: String::new(),
        }
    }
}

impl Review {
    pub fn new(review: String, rating: u8, user: String, tour: String) -> Self {
        Self {
            review,
            rating: rating.clamp(1, 5),
            user,
            tour,
            ..Default::default()
        }
    }
}

// Placeholder for calculating average ratings - would need database access
pub struct ReviewStats {
    pub tour_id: String,
    pub ratings_quantity: u32,
    pub ratings_average: f64,
}

impl ReviewStats {
    pub fn new(tour_id: String) -> Self {
        Self {
            tour_id,
            ratings_quantity: 0,
            ratings_average: 4.5,
        }
    }

    // Placeholder method - would aggregate from database
    pub fn calculate_average_ratings(&mut self, _reviews: &[Review]) {
        // TODO: Implement aggregation logic
        // For now, just set defaults
        self.ratings_quantity = 0;
        self.ratings_average = 4.5;
    }
}