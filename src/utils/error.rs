//! Centralized API errors modeled after TravelAndTour `AppError` + `errorController`.
//!
//! - **Operational** errors: safe to send `message` to the client in production.
//! - **Non-operational** errors: production hides details (like Express `sendErrorProd`).
//! - **MongoDB** errors are classified similarly to Cast / duplicate / validation / JWT remaps.
//! - Call [`init_error_reporting`] once at startup so dev vs prod matches `AppConfig::is_production`.

use std::sync::OnceLock;

use axum::{
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::errors::ErrorKind as JwtErrorKind;
use mongodb::error::{Error as MongoError, ErrorKind, WriteFailure};
use serde::Serialize;
use serde_json::json;

static PRODUCTION_MODE: OnceLock<bool> = OnceLock::new();

/// Must be called from `main` after loading config (mirrors relying on `NODE_ENV` / `APP_ENV`).
pub fn init_error_reporting(is_production: bool) {
    let _ = PRODUCTION_MODE.set(is_production);
}

fn is_production() -> bool {
    *PRODUCTION_MODE.get().unwrap_or(&false)
}

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
    pub is_operational: bool,
    /// Extra detail for development responses (and for logging non-operational errors).
    pub debug_detail: Option<String>,
}

#[derive(Serialize)]
struct ErrorBodyProd {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorBodyDev {
    status: String,
    message: String,
    #[serde(rename = "statusCode")]
    status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stack: Option<String>,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: message.into(),
            is_operational: true,
            debug_detail: None,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            message: message.into(),
            is_operational: true,
            debug_detail: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::FORBIDDEN,
            message: message.into(),
            is_operational: true,
            debug_detail: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            message: message.into(),
            is_operational: true,
            debug_detail: None,
        }
    }

    /// Programming / unknown failure: hide `message` from clients in production.
    pub fn internal(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message,
            is_operational: false,
            debug_detail: None,
        }
    }

    pub fn with_debug_detail(mut self, detail: impl Into<String>) -> Self {
        self.debug_detail = Some(detail.into());
        self
    }

    fn status_label(status: StatusCode) -> &'static str {
        if status.is_server_error() {
            "error"
        } else {
            "fail"
        }
    }

    fn log_server_error(&self, client_message: &str) {
        eprintln!("ERROR 💥 {}", self.message);
        if let Some(ref d) = self.debug_detail {
            eprintln!("ERROR 💥 detail: {d}");
        }
        if client_message != self.message {
            eprintln!("ERROR 💥 client message: {client_message}");
        }
    }

    pub fn from_jwt(err: &jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            JwtErrorKind::ExpiredSignature => {
                AppError::unauthorized("Your token has expired! Please log in again.")
            }
            _ => AppError::unauthorized("Invalid token. Please log in again!"),
        }
    }
}

/// Maps MongoDB driver errors into operational 4xx where appropriate (TravelAndTour `errorController` prod branch).
pub fn app_error_from_mongo(err: MongoError) -> AppError {
    let debug = err.to_string();

    match err.kind.as_ref() {
        ErrorKind::BsonDeserialization(e) => {
            AppError::bad_request(format!("Invalid data: {e}")).with_debug_detail(debug)
        }
        ErrorKind::BsonSerialization(e) => {
            AppError::bad_request(format!("Invalid data: {e}")).with_debug_detail(debug)
        }
        ErrorKind::InvalidArgument { message, .. } => {
            AppError::bad_request(message.clone()).with_debug_detail(debug)
        }
        ErrorKind::Command(cmd) => {
            let dup = duplicate_key_message(&cmd.message);
            match cmd.code {
                11000 => AppError::bad_request(dup).with_debug_detail(debug),
                121 => AppError::bad_request(format!(
                    "Invalid input data. {}",
                    cmd.message
                ))
                .with_debug_detail(debug),
                // BadValue / type mismatch — closest to Mongoose CastError for bad ids / filters.
                2 | 14 => AppError::bad_request(format!(
                    "Invalid input data. {}",
                    cmd.message
                ))
                .with_debug_detail(debug),
                _ => AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}")),
            }
        }
        ErrorKind::Write(WriteFailure::WriteError(we)) => {
            let dup = duplicate_key_message(&we.message);
            match we.code {
                11000 => AppError::bad_request(dup).with_debug_detail(debug),
                121 => AppError::bad_request(format!(
                    "Invalid input data. {}",
                    we.message
                ))
                .with_debug_detail(debug),
                _ => AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}")),
            }
        }
        ErrorKind::Write(WriteFailure::WriteConcernError(wc)) => {
            let dup = duplicate_key_message(&wc.message);
            match wc.code {
                11000 => AppError::bad_request(dup).with_debug_detail(debug),
                _ => AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}")),
            }
        }
        ErrorKind::InsertMany(bulk) => {
            if let Some(errors) = &bulk.write_errors {
                if let Some(first) = errors.first() {
                    let dup = duplicate_key_message(&first.message);
                    return match first.code {
                        11000 => AppError::bad_request(dup).with_debug_detail(debug),
                        121 => AppError::bad_request(format!(
                            "Invalid input data. {}",
                            first.message
                        ))
                        .with_debug_detail(debug),
                        _ => AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}")),
                    };
                }
            }
            AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}"))
        }
        _ => AppError::internal(debug.clone()).with_debug_detail(format!("{err:?}")),
    }
}

/// Best-effort duplicate message like `handleDuplicateFieldsDB` (uses server `errmsg`).
fn duplicate_key_message(errmsg: &str) -> String {
    if let Some(value) = extract_first_quoted_value(errmsg) {
        format!("Duplicate field value: {value}. Please use another value!")
    } else {
        "Duplicate field value. Please use another value!".to_string()
    }
}

fn extract_first_quoted_value(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let quote = match bytes[i] {
            b'\'' | b'"' => bytes[i],
            _ => {
                i += 1;
                continue;
            }
        };
        i += 1;
        let start = i;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == quote {
                return Some(s.get(start..i)?.to_string());
            }
            i += 1;
        }
        break;
    }
    None
}

impl From<MongoError> for AppError {
    fn from(err: MongoError) -> Self {
        app_error_from_mongo(err)
    }
}

impl From<bson::oid::Error> for AppError {
    fn from(err: bson::oid::Error) -> Self {
        AppError::bad_request(format!("Invalid id: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::bad_request(format!("Invalid JSON: {err}"))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(_: bcrypt::BcryptError) -> Self {
        AppError::internal("Could not hash or verify password.")
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::from_jwt(&err)
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::bad_request(rejection.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::bad_request(rejection.body_text())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code;
        let status_label = Self::status_label(status).to_string();

        let (client_message, include_debug) = if is_production() {
            if self.is_operational {
                (self.message.clone(), false)
            } else {
                self.log_server_error("Something went very wrong!");
                (
                    "Something went very wrong!".to_string(),
                    false,
                )
            }
        } else {
            // Development: richer payload (similar to `sendErrorDev` for `/api`).
            (self.message.clone(), true)
        };

        if is_production() {
            let body = ErrorBodyProd {
                status: status_label,
                message: client_message,
            };
            return (status, Json(body)).into_response();
        }

        let stack = std::backtrace::Backtrace::force_capture();
        let stack_str = format!("{stack}");

        let error_detail = include_debug.then(|| {
            self.debug_detail
                .clone()
                .unwrap_or_else(|| format!("{self:?}"))
        });

        let body = ErrorBodyDev {
            status: status_label,
            message: client_message,
            status_code: status.as_u16(),
            error: error_detail,
            stack: include_debug.then_some(stack_str),
        };
        (status, Json(body)).into_response()
    }
}

/// JSON body for panic responses (matches generic prod API error).
pub fn panic_response_json() -> axum::Json<serde_json::Value> {
    Json(json!({
        "status": "error",
        "message": "Something went very wrong!"
    }))
}
