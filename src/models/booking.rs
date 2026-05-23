use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::models::billing::BillingDetails;
use crate::models::factory_model::FactoryModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tour: ObjectId,
    pub user: ObjectId,
    pub price: f64,
    #[serde(
        default,
        with = "crate::models::bson_chrono::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default = "default_paid")]
    pub paid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<BillingDetails>,
}

fn default_paid() -> bool {
    true
}

impl FactoryModel for Booking {
    fn collection_name() -> &'static str {
        "bookings"
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

impl Booking {
    pub fn new(tour: ObjectId, user: ObjectId, price: f64, billing: Option<BillingDetails>) -> Self {
        let paid = billing
            .as_ref()
            .map(|b| b.payment_status == "paid")
            .unwrap_or(true);
        Self {
            id: None,
            tour,
            user,
            price,
            created_at: Some(Utc::now()),
            paid,
            billing,
        }
    }
}
