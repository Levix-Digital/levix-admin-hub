//! The instance: whether it is initialized (public, so the console can explain how to
//! initialize it on its host) and its settings.

use axum::extract::State;
use axum::Json;
use hub_core::instance::{Instance, InstanceSettings, InstanceStatus};

use crate::body::JsonBody;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend, ErrorBody};

#[utoipa::path(
    get,
    path = "/api/v1/instance",
    tag = "instance",
    summary = "Whether the Hub is initialized (public)",
    responses(
        (status = 200, description = "OK", body = InstanceStatus)
    )
)]
pub(crate) async fn status<B: Backend>(
    State(state): State<AppState<B>>,
) -> Result<Json<InstanceStatus>, ApiError> {
    Ok(Json(state.hub.instance_status().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/instance/settings",
    tag = "instance",
    summary = "The instance and its settings",
    responses(
        (status = 200, description = "OK", body = Instance),
        (status = 401, description = "Not signed in", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn settings<B: Backend>(
    State(state): State<AppState<B>>,
    _caller: Caller,
) -> Result<Json<Instance>, ApiError> {
    Ok(Json(state.hub.instance().await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/instance/settings",
    tag = "instance",
    summary = "Change the session limits (instance owner)",
    request_body = InstanceSettings,
    responses(
        (status = 200, description = "OK", body = Instance),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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
