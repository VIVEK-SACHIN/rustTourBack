use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub hello_message: String,
    pub app_env: String,
    pub log_level: String,
    /// `mongodb+srv://...` (SRV lookup — can fail on macOS with `.local` DNS search domains).
    pub database: String,
    /// Standard `mongodb://host:27017/natours` — use when SRV/DNS fails (Atlas → Connect → Drivers).
    pub database_direct: String,
    pub database_local: String,
    pub database_password: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_cookie_expires_in: u64,
    pub email_username: String,
    pub email_password: String,
    pub email_host: String,
    pub email_port: u16,
    pub email_from: String,
    pub email_from_name: String,
    /// SendGrid API key when `APP_ENV=production` (Natours `SENDGRID_API_KEY`).
    pub sendgrid_api_key: String,
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
        let database_direct = env::var("DATABASE_DIRECT").unwrap_or_default();
        let database_local = env::var("DATABASE_LOCAL")
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017/natours".to_string());
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
        let email_from =
            env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@natours.dev".to_string());
        let email_from_name =
            env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Natours".to_string());
        let sendgrid_api_key = env::var("SENDGRID_API_KEY").unwrap_or_default();
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
            database_direct,
            database_local,
            database_password,
            jwt_secret,
            jwt_expires_in,
            jwt_cookie_expires_in,
            email_username,
            email_password,
            email_host,
            email_port,
            email_from,
            email_from_name,
            sendgrid_api_key,
            publish_url,
            mapbox_token,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Matches Express `NODE_ENV === 'production'` and `APP_ENV === 'production'`.
    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
            || std::env::var("NODE_ENV")
                .map(|v| v.eq_ignore_ascii_case("production"))
                .unwrap_or(false)
    }
}
