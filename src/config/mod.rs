use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub hello_message: String,
    pub app_env: String,
    pub log_level: String,
    pub database: String,
    pub database_local: String,
    pub database_password: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_cookie_expires_in: u64,
    pub email_username: String,
    pub email_password: String,
    pub email_host: String,
    pub email_port: u16,
    pub publish_url: String,
    pub mapbox_token: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        if let Err(err) = dotenvy::dotenv() {
            eprintln!("dotenv load warning: {err}");
        }
        
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(3000);
        let hello_message = env::var("HELLO_MESSAGE").unwrap_or_else(|_| "Hello, World!".to_string());
        let app_env = env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let database = env::var("DATABASE").unwrap_or_default();
        let database_local = env::var("DATABASE_LOCAL")
            .unwrap_or_else(|_| "mongodb://localhost:27017/natours".to_string());
        let database_password = env::var("DATABASE_PASSWORD").unwrap_or_default();
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        let jwt_expires_in = env::var("JWT_EXPIRES_IN")
            .unwrap_or_else(|_| "90d".to_string())
            .replace(' ', "");
        let jwt_cookie_expires_in = env::var("JWT_COOKIE_EXPIRES_IN")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(90);
        let email_username = env::var("EMAIL_USERNAME").unwrap_or_default();
        let email_password = env::var("EMAIL_PASSWORD").unwrap_or_default();
        let email_host = env::var("EMAIL_HOST").unwrap_or_default();
        let email_port = env::var("EMAIL_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(2525);
        let publish_url = env::var("PUBLISH_URL").unwrap_or_default();
        let mapbox_token = env::var("MAPBOX_TOKEN")
            .or_else(|_| env::var("MAPBOX_ACCESS_TOKEN"))
            .unwrap_or_default();

        Self {
            host,
            port,
            hello_message,
            app_env,
            log_level,
            database,
            database_local,
            database_password,
            jwt_secret,
            jwt_expires_in,
            jwt_cookie_expires_in,
            email_username,
            email_password,
            email_host,
            email_port,
            publish_url,
            mapbox_token,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn resolved_database_url(&self) -> String {
        if !self.database.is_empty() {
            self.database.replace("<password>", &self.database_password)
        } else {
            self.database_local.clone()
        }
    }
}
