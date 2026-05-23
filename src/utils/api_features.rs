//! Query-string features from TravelAndTour `APIFeatures.js` (filter, sort, fields, paginate).

use std::collections::HashMap;

use mongodb::bson::{doc, Bson, Document};
use mongodb::options::FindOptions;
use regex::Regex;
use std::sync::LazyLock;

const RESERVED: &[&str] = &["page", "sort", "limit", "fields"];

static BRACKET_OP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+)\[(gte|gt|lte|lt)\]$").expect("valid regex"));

/// Parsed list query (filter + find options).
pub struct ApiFeatures {
    pub filter: Document,
    pub find_options: FindOptions,
}

impl ApiFeatures {
    /// `base_filter` is merged first (e.g. `{ tour: tourId }` for nested reviews).
    pub fn from_query(query: &HashMap<String, String>, base_filter: Document) -> Self {
        let filter = build_filter(query, base_filter);
        let find_options = FindOptions::builder()
            .sort(build_sort(query))
            .projection(build_projection(query))
            .skip(build_skip(query))
            .limit(build_limit(query))
            .build();

        Self {
            filter,
            find_options,
        }
    }
}

fn build_filter(query: &HashMap<String, String>, mut filter: Document) -> Document {
    for (key, value) in query {
        if RESERVED.contains(&key.as_str()) {
            continue;
        }
        if let Some(caps) = BRACKET_OP.captures(key) {
            let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let op = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let op_key = match op {
                "gte" => "$gte",
                "gt" => "$gt",
                "lte" => "$lte",
                "lt" => "$lt",
                _ => continue,
            };
            let parsed = parse_filter_value(value);
            let sub = filter
                .entry(field.to_string())
                .or_insert_with(|| Bson::Document(doc! {}));
            if let Bson::Document(d) = sub {
                d.insert(op_key, parsed);
            }
        } else if value.contains(',') {
            let values: Vec<Bson> = value
                .split(',')
                .map(|s| parse_filter_value(s.trim()))
                .collect();
            filter.insert(key.clone(), doc! { "$in": values });
        } else {
            filter.insert(key.clone(), parse_filter_value(value));
        }
    }
    filter
}

fn parse_filter_value(raw: &str) -> Bson {
    if let Ok(n) = raw.parse::<i64>() {
        return Bson::Int64(n);
    }
    if let Ok(n) = raw.parse::<f64>() {
        return Bson::Double(n);
    }
    match raw {
        "true" => Bson::Boolean(true),
        "false" => Bson::Boolean(false),
        _ => Bson::String(raw.to_string()),
    }
}

fn build_sort(query: &HashMap<String, String>) -> Document {
    let sort_str = query.get("sort").map(|s| s.as_str()).unwrap_or("-createdAt");
    let mut sort = Document::new();
    for part in sort_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(field) = part.strip_prefix('-') {
            sort.insert(field, -1);
        } else {
            sort.insert(part, 1);
        }
    }
    if sort.is_empty() {
        sort.insert("createdAt", -1);
    }
    sort
}

fn build_projection(query: &HashMap<String, String>) -> Option<Document> {
    query.get("fields").map(|fields| {
        let mut proj = Document::new();
        for field in fields.split(',') {
            let field = field.trim();
            if !field.is_empty() {
                proj.insert(field, 1);
            }
        }
        proj
    })
}

fn build_skip(query: &HashMap<String, String>) -> Option<u64> {
    let page = query
        .get("page")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = build_limit(query).unwrap_or(100) as u64;
    Some((page - 1) * limit)
}

fn build_limit(query: &HashMap<String, String>) -> Option<i64> {
    Some(
        query
            .get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(100)
            .clamp(1, 500),
    )
}
