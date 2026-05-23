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
    /// Standard `mongodb://host:27017/TravelAndTour` — use when SRV/DNS fails (Atlas → Connect → Drivers).
    pub database_direct: String,
    pub database_local: String,
    /// MongoDB database name (collections live under this DB on the cluster).
    pub database_name: String,
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
    /// SendGrid API key when `APP_ENV=production` (TravelAndTour `SENDGRID_API_KEY`).
    pub sendgrid_api_key: String,
    pub publish_url: String,
    pub mapbox_token: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    /// React app origin for Stripe success/cancel redirects and product images.
    pub frontend_url: String,
    /// TravelAndTour `public/` — static files (avatars under `img/users`).
    pub public_dir: std::path::PathBuf,
    /// Where resized uploads are written (`public/img/users`).
    pub users_upload_dir: std::path::PathBuf,
    /// Seeded tour photos (`public/img/tours`), served at `/img/tours/*`.
    pub tours_static_dir: std::path::PathBuf,
    /// Public API origin for absolute URLs (Stripe product images, etc.).
    pub server_public_url: String,
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
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017/TravelAndTour".to_string());
        let database_name = env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "TravelAndTour".to_string());
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
            env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@TravelAndTour.dev".to_string());
        let email_from_name =
            env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "TravelAndTour".to_string());
        let sendgrid_api_key = env::var("SENDGRID_API_KEY").unwrap_or_default();
        let publish_url = env::var("PUBLISH_URL").unwrap_or_default();
        let mapbox_token = env::var("MAPBOX_TOKEN")
            .or_else(|_| env::var("MAPBOX_ACCESS_TOKEN"))
            .unwrap_or_default();
        let stripe_secret_key = env::var("STRIPE_SECRET_KEY").unwrap_or_default();
        let stripe_webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "https://localhost:5173".to_string());
        let public_dir = env::var("PUBLIC_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public")
            });
        let users_upload_dir = env::var("USERS_UPLOAD_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| public_dir.join("img/users"));
        let tours_static_dir = env::var("TOURS_STATIC_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| public_dir.join("img/tours"));
        let server_public_url = env::var("SERVER_PUBLIC_URL")
            .unwrap_or_else(|_| format!("http://localhost:{port}"));

        Self {
            host,
            port,
            hello_message,
            app_env,
            log_level,
            database,
            database_direct,
            database_local,
            database_name,
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
            stripe_secret_key,
            stripe_webhook_secret,
            frontend_url,
            public_dir,
            users_upload_dir,
            tours_static_dir,
            server_public_url,
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
