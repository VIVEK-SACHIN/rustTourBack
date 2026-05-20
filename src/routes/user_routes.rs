use axum::{
    routing::get,
    Router,
};

use crate::handlers::users::get_all_users;
use crate::state::AppState;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_all_users))
}