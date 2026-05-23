use chrono::{DateTime, Utc};
use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::models::factory_model::FactoryModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub photo: String,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_confirm: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::bson_chrono::optional"
    )]
    pub changed_password_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset_token: Option<String>,
    /// Mongoose field name typo: `passwordResetTokenexpires`
    #[serde(rename = "passwordResetTokenexpires", skip_serializing_if = "Option::is_none")]
    pub password_reset_token_expires: Option<i64>,
    #[serde(default)]
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    User,
    Guide,
    #[serde(rename = "lead-guide")]
    LeadGuide,
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            email: String::new(),
            photo: "default.jpg".to_string(),
            role: UserRole::default(),
            password: None,
            password_confirm: None,
            changed_password_at: None,
            password_reset_token: None,
            password_reset_token_expires: None,
            active: true,
        }
    }
}

impl User {
    /// TravelAndTour `pre('save')` — invalidate JWTs issued before password change.
    pub fn touch_changed_password_at(&mut self) {
        self.changed_password_at = Some(Utc::now() - chrono::Duration::seconds(2));
    }

    pub fn verify_password(&self, candidate: &str) -> bool {
        if let Some(ref hash) = self.password {
            bcrypt::verify(candidate, hash).unwrap_or(false)
        } else {
            false
        }
    }

    /// `jwt_iat_seconds` from JWT `iat` claim (seconds).
    pub fn changed_password_after(&self, jwt_iat_seconds: i64) -> bool {
        if let Some(changed_at) = self.changed_password_at {
            let changed_ts = changed_at.timestamp();
            jwt_iat_seconds < changed_ts
        } else {
            false
        }
    }

    /// Mirrors `userSchema.methods.createPasswordResetToken` (plain token returned; hash stored).
    pub fn create_password_reset_token(&mut self) -> String {
        let bytes: [u8; 32] = rand::random();
        let reset_token = hex::encode(bytes);
        let hash = Sha256::digest(reset_token.as_bytes());
        self.password_reset_token = Some(hex::encode(hash));
        self.password_reset_token_expires =
            Some(Utc::now().timestamp_millis() + 10 * 60 * 1000);
        reset_token
    }

    pub fn strip_secrets_for_response(mut self) -> Self {
        self.password = None;
        self.password_confirm = None;
        self.password_reset_token = None;
        self.password_reset_token_expires = None;
        self
    }
}

pub fn hash_password(plain: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(plain, bcrypt::DEFAULT_COST)
}

impl FactoryModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn list_filter() -> Document {
        doc! { "active": { "$ne": false } }
    }

    fn list_projection() -> Option<Document> {
        Some(doc! {
            "password": 0,
            "passwordConfirm": 0,
            "passwordResetToken": 0,
            "passwordResetTokenexpires": 0
        })
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}
