//! Booking + Stripe checkout handlers (TravelAndTour `bookingController.js`).

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::{json, Value};

use crate::handlers::handler_factory;
use crate::models::billing::BillingDetails;
use crate::models::booking::Booking;
use crate::models::user::User;
use crate::models::Tour;
use crate::services::stripe::{create_checkout_session, verify_webhook_event};
use crate::state::AppState;
use crate::utils::error::AppError;

fn parse_tour_id(tour_id: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(tour_id).map_err(|e| AppError::bad_request(format!("Invalid tour id: {e}")))
}

/// `GET /api/v1/bookings/checkout-session/:tourId` (protected).
pub async fn get_checkout_session(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(tour_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let tour_oid = parse_tour_id(&tour_id)?;
    let db = state.db();
    let tours = db.collection::<Tour>("tours");

    let tour = tours
        .find_one(doc! { "_id": tour_oid, "secretTour": { "$ne": true } })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("No tour found with that ID"))?;

    let session = create_checkout_session(&state.config, &tour, &tour_oid, &user.email).await?;

    Ok(Json(json!({
        "status": "success",
        "session": session
    })))
}

async fn list_my_bookings_with_tours(
    state: &AppState,
    user_id: ObjectId,
) -> Result<Vec<Value>, AppError> {
    let db = state.db();
    let bookings_coll = db.collection::<Booking>("bookings");
    let tours_coll = db.collection::<Tour>("tours");

    let cursor = bookings_coll
        .find(doc! { "user": user_id })
        .await
        .map_err(AppError::from)?;

    let bookings: Vec<Booking> = cursor.try_collect().await.map_err(AppError::from)?;

    let mut docs = Vec::with_capacity(bookings.len());
    for booking in bookings {
        let Some(booking_id) = booking.id else {
            continue;
        };

        let Some(tour) = tours_coll
            .find_one(doc! { "_id": booking.tour })
            .await
            .map_err(AppError::from)?
        else {
            continue;
        };

        docs.push(json!({
            "_id": booking_id,
            "price": booking.price,
            "paid": booking.paid,
            "createdAt": booking.created_at,
            "billing": booking.billing,
            "tour": tour,
        }));
    }

    Ok(docs)
}

/// `GET /api/v1/bookings/my` (protected) — current user's bookings with tour details.
pub async fn get_my_bookings(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User missing _id"))?;

    let docs = list_my_bookings_with_tours(&state, user_id).await?;

    Ok(Json(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    })))
}

/// `GET /api/v1/billing/my` — same records as bookings, for the account billing tab.
pub async fn get_my_billing(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .id
        .ok_or_else(|| AppError::internal("User missing _id"))?;

    let docs = list_my_bookings_with_tours(&state, user_id).await?;

    Ok(Json(json!({
        "status": "success",
        "results": docs.len(),
        "data": { "docs": docs }
    })))
}

async fn create_booking_from_checkout_session(
    state: &AppState,
    session: &Value,
) -> Result<(), AppError> {
    let tour_id = session
        .get("client_reference_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::internal("Checkout session missing client_reference_id"))?;

    let email = session
        .get("customer_email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::internal("Checkout session missing customer_email"))?;

    let price_cents = session
        .get("amount_total")
        .and_then(|v| v.as_i64())
        .or_else(|| {
            session
                .get("display_items")
                .and_then(|items| items.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("amount"))
                .and_then(|v| v.as_i64())
        })
        .ok_or_else(|| AppError::internal("Checkout session missing amount_total"))?;

    let tour_oid = parse_tour_id(tour_id)?;
    let db = state.db();
    let users = db.collection::<User>("users");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::internal("No user found for checkout customer_email"))?;

    let user_id = user.id.ok_or_else(|| AppError::internal("User missing _id"))?;

    let price = price_cents as f64 / 100.0;
    let billing = BillingDetails::from_checkout_session(session, price);

    let bookings = db.collection::<Booking>("bookings");

    if let Some(ref b) = billing {
        let existing = bookings
            .find_one(doc! { "billing.stripeSessionId": &b.stripe_session_id })
            .await
            .map_err(AppError::from)?;
        if existing.is_some() {
            return Ok(());
        }
    }

    let booking = Booking::new(tour_oid, user_id, price, billing);
    bookings
        .insert_one(&booking)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

/// `POST /webhook-checkout` — raw body, before JSON middleware (TravelAndTour `app.js`).
pub async fn webhook_checkout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::bad_request("Webhook error: missing Stripe-Signature header"))?;

    let event = verify_webhook_event(
        &body,
        signature,
        &state.config.stripe_webhook_secret,
    )?;

    if event.get("type").and_then(|v| v.as_str()) == Some("checkout.session.completed") {
        let session = event
            .get("data")
            .and_then(|d| d.get("object"))
            .ok_or_else(|| AppError::internal("Webhook event missing session object"))?;
        create_booking_from_checkout_session(&state, session).await?;
    }

    Ok(StatusCode::OK)
}

pub use handler_factory::{
    create_one as create_booking, delete_one as delete_booking, get_all as get_all_bookings,
    get_one as get_booking, update_one as update_booking,
};
