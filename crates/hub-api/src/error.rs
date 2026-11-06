//! The error object of the contract: `{"error": {"code": "...", "message": "..."}}`.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use hub_core::hub::error_code;
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
    retry_after_secs: Option<u64>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            retry_after_secs: None,
        }
    }

    pub(crate) fn throttled(retry_after_secs: u64) -> Self {
        Self {
            retry_after_secs: Some(retry_after_secs),
            ..Self::new(
                StatusCode::TOO_MANY_REQUESTS,
                "throttled",
                format!("too many requests; retry in {retry_after_secs} s"),
            )
        }
    }

    pub(crate) fn unauthenticated() -> Self {
        CoreError::Unauthenticated.into()
    }

    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        let status = match &error {
            CoreError::Invalid { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            CoreError::NotFound(_) => StatusCode::NOT_FOUND,
            CoreError::Conflict(_) | CoreError::NotInitialized | CoreError::AlreadyInitialized => {
                StatusCode::CONFLICT
            }
            CoreError::Unauthenticated => StatusCode::UNAUTHORIZED,
            CoreError::Forbidden { .. } => StatusCode::FORBIDDEN,
            CoreError::Throttled { retry_after_secs } => return Self::throttled(*retry_after_secs),
            CoreError::Storage(detail) => {
                tracing::error!(%detail, "storage failure");
                return Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal error; see the Hub logs",
                );
            }
        };
        Self::new(status, error_code(&error), error.to_string())
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
        let mut response = (self.status, Json(body)).into_response();
        if let Some(secs) = self.retry_after_secs {
            if let Ok(value) = HeaderValue::from_str(&secs.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

pub(crate) async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such endpoint")
}
