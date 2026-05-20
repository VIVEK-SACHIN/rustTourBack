use axum::{
    routing::get,
    Router,
};

use crate::handlers::users::{
    get_all_users,
};
use mongodb::Client;

pub fn user_routes() -> Router<Client> {
    Router::new()
        .route("/users", get(get_all_users))
}