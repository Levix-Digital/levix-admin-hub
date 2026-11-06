//! Account actions owners take on other administrators.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::AdministratorView;
use uuid::Uuid;

use crate::auth::LinkBody;
use crate::body::PathParams;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend};

pub(crate) async fn suspend<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<AdministratorView>, ApiError> {
    Ok(Json(
        state.hub.suspend_administrator(&caller.context, id).await?,
    ))
}

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

pub(crate) async fn revoke_sessions<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_sessions_of(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
