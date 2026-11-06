//! The instance: whether it is initialized (public, so the console can explain how to
//! initialize it on its host) and its settings.

use axum::extract::State;
use axum::Json;
use hub_core::instance::{Instance, InstanceSettings, InstanceStatus};

use crate::body::JsonBody;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend};

pub(crate) async fn status<B: Backend>(
    State(state): State<AppState<B>>,
) -> Result<Json<InstanceStatus>, ApiError> {
    Ok(Json(state.hub.instance_status().await?))
}

pub(crate) async fn settings<B: Backend>(
    State(state): State<AppState<B>>,
    _caller: Caller,
) -> Result<Json<Instance>, ApiError> {
    Ok(Json(state.hub.instance().await?))
}

pub(crate) async fn update_settings<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(settings): JsonBody<InstanceSettings>,
) -> Result<Json<Instance>, ApiError> {
    Ok(Json(
        state
            .hub
            .update_instance_settings(&caller.context, settings)
            .await?,
    ))
}
