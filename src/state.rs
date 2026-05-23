use std::sync::Arc;

use mongodb::Client;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub config: Arc<AppConfig>,
}

impl AppState {
    /// MongoDB database handle (`DATABASE_NAME`, default `TravelAndTour`).
    pub fn db(&self) -> mongodb::Database {
        self.client.database(&self.config.database_name)
    }
}
