//! `GET /api/v1/instance`: whether the Hub has been initialized. Public on purpose: the
//! console uses it to explain how to initialize the Hub on its host.

use axum::extract::State;
use axum::Json;
use hub_core::instance::{self, InstanceStatus};

use crate::{ApiError, AppState, Readiness, Store};

pub(crate) async fn status<S: Store, R: Readiness>(
    State(state): State<AppState<S, R>>,
) -> Result<Json<InstanceStatus>, ApiError> {
    Ok(Json(instance::status(&state.store).await?))
}
