use axum::{
    middleware as axum_middleware,
    routing::get,
    Router,
};

use crate::handlers::bookings::{
    create_booking, delete_booking, get_all_bookings, get_booking, get_checkout_session,
    get_my_billing, get_my_bookings, update_booking,
};
use crate::middleware::auth::protect;
use crate::middleware::restrict_to::{restrict_to, RequireRoles};
use crate::models::booking::Booking;
use crate::models::user::UserRole;
use crate::state::AppState;

const ADMIN_LEAD: &[UserRole] = &[UserRole::Admin, UserRole::LeadGuide];

pub fn booking_routes(state: &AppState) -> Router<AppState> {
    let s = state.clone();

    let checkout = Router::new()
        .route(
            "/bookings/checkout-session/:tourId",
            get(get_checkout_session),
        )
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let my_bookings = Router::new()
        .route("/bookings/my", get(get_my_bookings))
        .route("/billing/my", get(get_my_billing))
        .route_layer(axum_middleware::from_fn_with_state(s.clone(), protect));

    let admin = Router::new()
        .route("/bookings", get(get_all_bookings::<Booking>).post(create_booking::<Booking>))
        .route(
            "/bookings/:id",
            get(get_booking::<Booking>)
                .patch(update_booking::<Booking>)
                .delete(delete_booking::<Booking>),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            RequireRoles(ADMIN_LEAD),
            restrict_to,
        ))
        .route_layer(axum_middleware::from_fn_with_state(s, protect));

    Router::new()
        .merge(checkout)
        .merge(my_bookings)
        .merge(admin)
}
