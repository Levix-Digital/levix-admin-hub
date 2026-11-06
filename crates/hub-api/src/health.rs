//! Liveness and readiness for container orchestrators.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::{ApiError, AppState, Backend, Readiness};

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

/// The process is running.
pub(crate) async fn live() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// The database and the key file are usable.
pub(crate) async fn ready<B: Backend>(
    State(state): State<AppState<B>>,
) -> Result<Json<Health>, ApiError> {
    match state.readiness.check().await {
        Ok(()) => Ok(Json(Health { status: "ready" })),
        Err(reason) => {
            tracing::warn!(%reason, "not ready");
            Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "not_ready",
                reason,
            ))
        }
    }
}
