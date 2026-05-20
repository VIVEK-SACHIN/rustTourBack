use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::factory_model::FactoryModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub review: String,
    pub rating: u8,
    #[serde(rename = "CreatedAt", with = "crate::models::bson_chrono::required")]
    pub created_at: DateTime<Utc>,
    pub user: ObjectId,
    pub tour: ObjectId,
}

impl Default for Review {
    fn default() -> Self {
        Self {
            id: None,
            review: String::new(),
            rating: 5,
            created_at: Utc::now(),
            user: ObjectId::new(),
            tour: ObjectId::new(),
        }
    }
}

impl FactoryModel for Review {
    fn collection_name() -> &'static str {
        "reviews"
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}
