//! Account actions owners take on other administrators.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::AdministratorView;
use uuid::Uuid;

use crate::auth::LinkBody;
use crate::body::PathParams;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend, ErrorBody};

#[utoipa::path(
    post,
    path = "/api/v1/administrators/{id}/suspend",
    tag = "administrators",
    summary = "Suspend an administrator and end their sessions",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = AdministratorView),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn suspend<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<AdministratorView>, ApiError> {
    Ok(Json(
        state.hub.suspend_administrator(&caller.context, id).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/administrators/{id}/reactivate",
    tag = "administrators",
    summary = "Reactivate a suspended administrator",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = AdministratorView),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn reactivate<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<AdministratorView>, ApiError> {
    Ok(Json(
        state
            .hub
            .reactivate_administrator(&caller.context, id)
            .await?,
    ))
}

/// Issues a recovery link the owner delivers; the administrator enrolls again with it.
#[utoipa::path(
    post,
    path = "/api/v1/administrators/{id}/reset-second-factor",
    tag = "administrators",
    summary = "Issue a recovery link (shown once) and end their sessions",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = LinkBody),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn reset_second_factor<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<LinkBody>, ApiError> {
    Ok(Json(
        state
            .hub
            .reset_second_factor(&caller.context, id)
            .await?
            .into(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/administrators/{id}/revoke-sessions",
    tag = "administrators",
    summary = "End every session of an administrator",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 204, description = "Done"),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn revoke_sessions<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_sessions_of(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
