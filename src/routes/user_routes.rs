use axum::{
    middleware as axum_middleware,
    routing::{get, patch, post},
    Router,
};

use crate::handlers::auth::{
    forgot_password, login, logout, me, reset_password, signup, update_password,
};
use crate::middleware::auth::protect;
use crate::state::AppState;

pub fn user_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();
    let protected = Router::new()
        .route("/users/me", get(me))
        .route("/users/update-password", patch(update_password))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .route("/users/signup", post(signup))
        .route("/users/login", post(login))
        .route("/users/logout", get(logout))
        .route("/users/forgot-password", post(forgot_password))
        .route("/users/reset-password/:token", patch(reset_password))
        .merge(protected)

    //yet to Implement routes for user as per node     
}
