use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tour {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub duration: u32,
    #[serde(default)]
    pub max_group_size: u32,
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub ratings_average: f64,
    #[serde(default)]
    pub ratings_quantity: u32,
    #[serde(default)]
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_discount: Option<f64>,
    #[serde(default)]
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub image_cover: String,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::bson_chrono::optional"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, with = "crate::models::bson_chrono::vec")]
    pub start_dates: Vec<DateTime<Utc>>,
    #[serde(default)]
    pub secret_tour: bool,
    #[serde(default)]
    pub start_location: Location,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub guides: Vec<ObjectId>, // ObjectIds as ObjectId type
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(default)]
    pub r#type: String, // "Point"
    #[serde(default)]
    pub coordinates: Vec<f64>, // [longitude, latitude]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Difficult,
}

impl Default for Tour {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            slug: String::new(),
            duration: 0,
            max_group_size: 0,
            difficulty: Difficulty::Easy,
            ratings_average: 4.5,
            ratings_quantity: 0,
            price: 0.0,
            price_discount: None,
            summary: String::new(),
            description: None,
            image_cover: String::new(),
            images: Vec::new(),
            created_at: Some(Utc::now()),
            start_dates: Vec::new(),
            secret_tour: false,
            start_location: Location::default(),
            locations: Vec::new(),
            guides: Vec::new(),
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            r#type: "Point".to_string(),
            coordinates: Vec::new(),
            address: None,
            description: None,
            day: None,
        }
    }
}

impl Tour {
    pub fn new(name: String, duration: u32, max_group_size: u32, difficulty: Difficulty, price: f64, summary: String, image_cover: String) -> Self {
        Self {
            name: name.clone(),
            slug: slugify(&name),
            duration,
            max_group_size,
            difficulty,
            price,
            summary,
            image_cover,
            ..Default::default()
        }
    }

    pub fn duration_weeks(&self) -> f64 {
        self.duration as f64 / 7.0
    }
}

fn slugify(name: &str) -> String {
    name.to_lowercase().replace(" ", "-").replace("_", "-")
}