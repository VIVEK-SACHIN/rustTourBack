use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router,
};

use crate::handlers::auth::{
    forgot_password, login, logout, me, reset_password, signup, update_password,
};
use crate::handlers::users::{
    create_user, delete_me, delete_user, get_all_users, get_user, update_user,
};
use crate::middleware::auth::protect;
use crate::middleware::restrict_to::{restrict_to, RequireRoles};
use crate::models::user::UserRole;
use crate::state::AppState;

const ADMIN_ONLY: &[UserRole] = &[UserRole::Admin];
const ADMIN_LEAD: &[UserRole] = &[UserRole::Admin, UserRole::LeadGuide];

pub fn user_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    let public = Router::new()
        .route("/users/signup", post(signup))
        .route("/users/login", post(login))
        .route("/users/logout", get(logout))
        .route("/users/forgetPassword", post(forgot_password))
        .route("/users/resetPassword/:token", patch(reset_password));

    let account = Router::new()
        .route("/users/me", get(me))
        .route("/users/updateMyPassword", patch(update_password))
        .route("/users/deleteMe", delete(delete_me))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin_get_all = Router::new()
        .route("/users", get(get_all_users))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_ONLY),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin_post_user = Router::new()
        .route("/users", post(create_user))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin_get_one = Router::new()
        .route("/users/:id", get(get_user))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin_patch = Router::new()
        .route("/users/:id", patch(update_user))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_ONLY),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin_delete = Router::new()
        .route("/users/:id", delete(delete_user))
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .merge(public)
        .merge(account)
        .merge(admin_get_all)
        .merge(admin_post_user)
        .merge(admin_get_one)
        .merge(admin_patch)
        .merge(admin_delete)
}
