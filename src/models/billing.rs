//! Payment snapshot saved on each booking when Stripe checkout completes.

use serde::{Deserialize, Serialize};

/// Stripe checkout fields persisted on a booking (TravelAndTour billing tab data source).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingDetails {
    pub stripe_session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stripe_payment_intent: Option<String>,
    pub currency: String,
    pub payment_status: String,
    pub amount_total: f64,
}

impl BillingDetails {
    pub fn from_checkout_session(session: &serde_json::Value, amount_total: f64) -> Option<Self> {
        let stripe_session_id = session.get("id")?.as_str()?.to_string();
        let stripe_payment_intent = session.get("payment_intent").and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.get("id").and_then(|id| id.as_str()).map(String::from))
        });
        let currency = session
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("usd")
            .to_string();
        let payment_status = session
            .get("payment_status")
            .and_then(|v| v.as_str())
            .unwrap_or("paid")
            .to_string();

        Some(Self {
            stripe_session_id,
            stripe_payment_intent,
            currency,
            payment_status,
            amount_total,
        })
    }
}
