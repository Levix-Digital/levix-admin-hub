//! Liveness and readiness for container orchestrators.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{ApiError, AppState, Backend, ErrorBody, Readiness};

#[derive(Serialize, ToSchema)]
pub(crate) struct Health {
    status: &'static str,
}

/// The process is running.
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    summary = "The process runs",
    responses(
        (status = 200, description = "OK", body = Health)
    )
)]
pub(crate) async fn live() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// The database and the key file are usable.
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    summary = "The database and the key file are usable",
    responses(
        (status = 200, description = "OK", body = Health),
        (status = 503, description = "Not ready", body = ErrorBody)
    )
)]
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
