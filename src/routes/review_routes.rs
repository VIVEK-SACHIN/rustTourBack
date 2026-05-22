use axum::{
    middleware as axum_middleware,
    routing::{get, patch, post},
    Router,
};

use crate::handlers::reviews::{
    create_review, create_review_on_tour, delete_review, get_all_reviews,
    get_all_reviews_on_tour, get_my_reviews, get_review, update_review,
};
use crate::middleware::auth::protect;
use crate::middleware::restrict_to::{restrict_to, RequireRoles};
use crate::models::user::UserRole;
use crate::state::AppState;

const USER_ONLY: &[UserRole] = &[UserRole::User];
const USER_OR_ADMIN: &[UserRole] = &[UserRole::User, UserRole::Admin];

/// `GET|POST /api/v1/reviews` and `GET|PATCH|DELETE /api/v1/reviews/:id`
pub fn review_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    let my_reviews = Router::new()
        .route("/reviews/my", get(get_my_reviews))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let read = Router::new()
        .route("/reviews", get(get_all_reviews))
        .route("/reviews/:id", get(get_review))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let create = Router::new()
        .route("/reviews", post(create_review))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(USER_ONLY),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let mutate = Router::new()
        .route(
            "/reviews/:id",
            patch(update_review).delete(delete_review),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(USER_OR_ADMIN),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .merge(my_reviews)
        .merge(read)
        .merge(create)
        .merge(mutate)
}

/// Nested under `/api/v1/tours/:tourId/review` (Natours singular path).
pub fn nested_review_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    let read = Router::new()
        .route("/", get(get_all_reviews_on_tour))
        .route("/:id", get(get_review))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let create = Router::new()
        .route("/", post(create_review_on_tour))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(USER_ONLY),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let mutate = Router::new()
        .route("/:id", patch(update_review).delete(delete_review))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(USER_OR_ADMIN),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new().merge(read).merge(create).merge(mutate)
}
