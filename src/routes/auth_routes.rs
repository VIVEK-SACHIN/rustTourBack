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

pub fn auth_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();
    let protected = Router::new()
        .route("/auth/me", get(me))
        .route("/auth/update-password", patch(update_password))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/logout", get(logout))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password/:token", patch(reset_password))
        .merge(protected)
}
