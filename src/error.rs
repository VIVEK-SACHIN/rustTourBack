use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
    pub is_operational: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
    #[serde(rename = "statusCode")]
    status_code: u16,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: message.into(),
            is_operational: true,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            message: message.into(),
            is_operational: true,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            is_operational: false,
        }
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code;
        let status_text = if status.is_server_error() {
            "error"
        } else {
            "fail"
        };

        let body = ErrorResponse {
            status: status_text.to_string(),
            message: self.message,
            status_code: status.as_u16(),
        };

        (status, Json(body)).into_response()
    }
}
