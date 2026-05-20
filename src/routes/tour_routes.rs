use axum::{routing::get, Router};

use crate::handlers::tours::{
    create_tour, delete_tour, get_all_tours, get_tour, update_tour,
};
use crate::state::AppState;

pub fn tour_routes() -> Router<AppState> {
    Router::new()
        .route("/tours", get(get_all_tours).post(create_tour))
        .route(
            "/tours/:id",
            get(get_tour).patch(update_tour).delete(delete_tour),
        )
}
