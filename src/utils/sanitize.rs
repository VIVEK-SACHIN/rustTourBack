//! Strip MongoDB operator keys from JSON (Natours `express-mongo-sanitize`).

use serde_json::Value;

/// Recursively remove object keys that start with `$`.
pub fn sanitize_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| !k.starts_with('$'));
            for v in map.values_mut() {
                sanitize_json(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                sanitize_json(v);
            }
        }
        _ => {}
    }
}

/// Reject query keys/values that look like NoSQL injection payloads.
pub fn reject_mongo_operators_in_query(query: &str) -> Result<(), &'static str> {
    if query.contains('$') {
        return Err("Query string contains forbidden characters.");
    }
    Ok(())
}
