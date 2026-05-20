use axum::{
    routing::get,
    Router,
};

use crate::handlers::tours::get_all_tours;
use crate::state::AppState;

pub fn tour_routes() -> Router<AppState> {
    Router::new()
        .route("/tours", get(get_all_tours))
}