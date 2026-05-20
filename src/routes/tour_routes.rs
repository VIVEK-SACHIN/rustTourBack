use axum::{
    routing::get,
    Router,
};

use crate::handlers::tours::{
    get_all_tours,
};
use mongodb::Client;

pub fn tour_routes() -> Router<Client> {
    Router::new()
        .route("/tours", get(get_all_tours))
}