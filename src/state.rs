use std::sync::Arc;

use mongodb::Client;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub config: Arc<AppConfig>,
}
