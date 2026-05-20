use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
    Extension,
};

use crate::models::user::{User, UserRole};
use crate::utils::error::AppError;

/// Role allow-list for [`restrict_to`] (Natours `authController.restrictTo`).
#[derive(Clone)]
pub struct RequireRoles(pub &'static [UserRole]);

/// Must run **after** [`super::auth::protect`] so `Extension<User>` is present.
pub async fn restrict_to(
    Extension(user): Extension<User>,
    State(roles): State<RequireRoles>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !roles.0.contains(&user.role) {
        return Err(AppError::forbidden(
            "You do not have permission to perform this action",
        ));
    }
    Ok(next.run(request).await)
}
