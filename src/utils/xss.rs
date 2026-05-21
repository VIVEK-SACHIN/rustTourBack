//! Sanitize string values in JSON bodies (Natours `xss-clean`).

use serde_json::Value;

/// Escape HTML metacharacters in all string leaves.
pub fn sanitize_xss_json(value: &mut Value) {
    match value {
        Value::String(s) => {
            *s = escape_html(s);
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                sanitize_xss_json(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                sanitize_xss_json(v);
            }
        }
        _ => {}
    }
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}
