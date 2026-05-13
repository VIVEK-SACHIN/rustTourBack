use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub hello_message: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(3000);
        let hello_message = env::var("HELLO_MESSAGE").unwrap_or_else(|_| "Hello, World!".to_string());

        Self {
            host,
            port,
            hello_message,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
