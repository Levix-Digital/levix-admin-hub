//! The caller's own account: profile, password, recovery codes, sessions and API tokens.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::Me;
use hub_core::session::{ApiToken, Session};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::body::{JsonBody, PathParams};
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend};

#[derive(Serialize)]
pub(crate) struct Items<T> {
    pub items: Vec<T>,
}

pub(crate) async fn get<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Me>, ApiError> {
    Ok(Json(state.hub.me(&caller.context).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    locale: Option<String>,
}

pub(crate) async fn update<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(request): JsonBody<ProfileRequest>,
) -> Result<Json<Me>, ApiError> {
    Ok(Json(
        state
            .hub
            .update_profile(
                &caller.context,
                request.display_name.as_deref(),
                request.locale.as_deref(),
            )
            .await?,
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PasswordRequest {
    current_password: String,
    new_password: String,
    code: String,
}

pub(crate) async fn change_password<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(request): JsonBody<PasswordRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .hub
        .change_password(
            &caller.context,
            &request.current_password,
            &request.new_password,
            &request.code,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReauthRequest {
    password: String,
    code: String,
}

#[derive(Serialize)]
pub(crate) struct RecoveryCodes {
    recovery_codes: Vec<String>,
}

pub(crate) async fn regenerate_recovery_codes<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(request): JsonBody<ReauthRequest>,
) -> Result<Json<RecoveryCodes>, ApiError> {
    let recovery_codes = state
        .hub
        .regenerate_recovery_codes(&caller.context, &request.password, &request.code)
        .await?;
    Ok(Json(RecoveryCodes { recovery_codes }))
}

#[derive(Serialize)]
pub(crate) struct SessionItem {
    #[serde(flatten)]
    session: Session,
    current: bool,
}

pub(crate) async fn sessions<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Items<SessionItem>>, ApiError> {
    let current = caller.session.as_ref().map(|s| s.id);
    let items = state
        .hub
        .my_sessions(&caller.context)
        .await?
        .into_iter()
        .map(|session| SessionItem {
            current: Some(session.id) == current,
            session,
        })
        .collect();
    Ok(Json(Items { items }))
}

pub(crate) async fn revoke_session<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_my_session(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn tokens<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Items<ApiToken>>, ApiError> {
    let items = state.hub.my_api_tokens(&caller.context).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TokenRequest {
    name: String,
    days: i64,
}

#[derive(Serialize)]
pub(crate) struct CreatedToken {
    token: ApiToken,
    /// The secret, shown once.
    secret: String,
}

pub(crate) async fn create_token<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(request): JsonBody<TokenRequest>,
) -> Result<(StatusCode, Json<CreatedToken>), ApiError> {
    let (token, secret) = state
        .hub
        .create_api_token(&caller.context, &request.name, request.days)
        .await?;
    Ok((StatusCode::CREATED, Json(CreatedToken { token, secret })))
}

pub(crate) async fn revoke_token<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_api_token(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
