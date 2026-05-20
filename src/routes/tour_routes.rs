use axum::{
    middleware as axum_middleware,
    routing::{get, patch, post},
    Router,
};

use crate::handlers::tours::{
    create_tour, delete_tour, get_all_tours, get_tour, update_tour,
};
use crate::middleware::auth::protect;
use crate::middleware::restrict_to::{restrict_to, RequireRoles};
use crate::models::user::UserRole;
use crate::routes::review_routes::nested_review_routes;
use crate::state::AppState;

const ADMIN_LEAD: &[UserRole] = &[UserRole::Admin, UserRole::LeadGuide];

pub fn tour_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    let public = Router::new()
        .route("/tours", get(get_all_tours))
        .route("/tours/:id", get(get_tour))
        .nest("/tours/:tourId/reviews", nested_review_routes(state));

    let mutate = Router::new()
        .route("/tours", post(create_tour))
        .route("/tours/:id", patch(update_tour).delete(delete_tour))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new().merge(public).merge(mutate)
}
