//! The error object of the contract: `{"error": {"code": "...", "message": "..."}}`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hub_core::CoreError;
use serde::Serialize;

/// Body of every error response.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    /// Stable, machine-readable code.
    pub code: &'static str,
    /// Human-readable explanation, without internals.
    pub message: String,
}

/// An error answered by the API.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match &error {
            CoreError::Invalid { .. } => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid",
                error.to_string(),
            ),
            CoreError::NotFound(_) => {
                Self::new(StatusCode::NOT_FOUND, "not_found", error.to_string())
            }
            CoreError::Conflict(_) => {
                Self::new(StatusCode::CONFLICT, "conflict", error.to_string())
            }
            CoreError::NotInitialized => {
                Self::new(StatusCode::CONFLICT, "not_initialized", error.to_string())
            }
            CoreError::AlreadyInitialized => Self::new(
                StatusCode::CONFLICT,
                "already_initialized",
                error.to_string(),
            ),
            CoreError::Storage(detail) => {
                tracing::error!(%detail, "storage failure");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal error; see the Hub logs",
                )
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

pub(crate) async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such endpoint")
}
