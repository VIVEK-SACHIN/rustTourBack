use axum::{
    middleware as axum_middleware,
    routing::{get, patch, post},
    Router,
};

use crate::handlers::tours::{
    create_tour, delete_tour, get_all_tours, get_distances, get_monthly_plan, get_top_5_cheap,
    get_tour, get_tour_stats, get_tours_within, update_tour,
};
use crate::middleware::auth::protect;
use crate::middleware::restrict_to::{restrict_to, RequireRoles};
use crate::models::user::UserRole;
use crate::routes::review_routes::nested_review_routes;
use crate::state::AppState;

const ADMIN_LEAD: &[UserRole] = &[UserRole::Admin, UserRole::LeadGuide];
const MONTHLY_ROLES: &[UserRole] = &[UserRole::Admin, UserRole::LeadGuide, UserRole::Guide];

pub fn tour_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    // Literal paths before `/tours/:id` so `monthly-plan` is not captured as an id.
    let public = Router::new()
        .route("/tours/top-5-cheap", get(get_top_5_cheap))
        .route("/tours/tour-stats", get(get_tour_stats))
        .route(
            "/tours/tours-within/:distance/center/:latlng/unit/:unit",
            get(get_tours_within),
        )
        .route("/tours/distances/:latlng/unit/:unit", get(get_distances))
        .route("/tours", get(get_all_tours))
        .nest("/tours/:tourId/reviews", nested_review_routes(state));

    let monthly = Router::new()
        .route("/tours/monthly-plan/:year", get(get_monthly_plan))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(MONTHLY_ROLES),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let by_id = Router::new().route("/tours/:id", get(get_tour));

    let mutate = Router::new()
        .route("/tours", post(create_tour))
        .route("/tours/:id", patch(update_tour).delete(delete_tour))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .merge(public)
        .merge(monthly)
        .merge(by_id)
        .merge(mutate)
}
