use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub photo: String,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_confirm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_password_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset_token_expires: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new(name: String, email: String, password: String) -> Self {
        Self {
            name,
            email,
            password: Some(password),
            ..Default::default()
        }
    }

    // Placeholder for password verification - would need bcrypt crate
    pub fn verify_password(&self, _candidate: &str) -> bool {
        // TODO: Implement bcrypt verification
        false
    }

    // Placeholder for password change check
    pub fn changed_password_after(&self, _jwt_timestamp: i64) -> bool {
        if let Some(changed_at) = self.changed_password_at {
            let changed_timestamp = changed_at.timestamp();
            _jwt_timestamp < changed_timestamp
        } else {
            false
        }
    }

    // Placeholder for password reset token creation
    pub fn create_password_reset_token(&mut self) -> String {
        // TODO: Implement crypto random token generation
        let reset_token = "placeholder_reset_token".to_string();
        // Hash the token and store it
        self.password_reset_token = Some("hashed_token".to_string());
        self.password_reset_token_expires = Some(Utc::now().timestamp() + 10 * 60); // 10 minutes
        reset_token
    }
}