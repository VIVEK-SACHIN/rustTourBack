//! BSON `Date` ↔ `chrono::DateTime<Utc>` for documents written by Mongoose.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Required `DateTime<Utc>` field helper.
pub mod required {
    pub use bson::serde_helpers::chrono_datetime_as_bson_datetime::{
        deserialize, serialize,
    };
}

/// `Option<DateTime<Utc>>` field helper (`#[serde(with = "...")]`).
pub mod optional {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        bson::serde_helpers::chrono_datetime_as_bson_datetime_optional::deserialize(deserializer)
    }

    pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bson::serde_helpers::chrono_datetime_as_bson_datetime_optional::serialize(value, serializer)
    }
}

/// `Vec<DateTime<Utc>>` field helper.
pub mod vec {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dates: Vec<bson::DateTime> = Vec::deserialize(deserializer)?;
        Ok(dates.into_iter().map(|d| d.to_chrono()).collect())
    }

    pub fn serialize<S>(value: &Vec<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dates: Vec<bson::DateTime> = value.iter().map(|d| bson::DateTime::from_chrono(*d)).collect();
        dates.serialize(serializer)
    }
}
