//! The caller's own account: profile, password, recovery codes, sessions and API tokens.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::Me;
use hub_core::session::{ApiToken, Session};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::body::{JsonBody, PathParams};
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend, ErrorBody};

#[derive(Serialize, ToSchema)]
pub(crate) struct Items<T> {
    pub items: Vec<T>,
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    tag = "me",
    summary = "The caller's account and roles",
    responses(
        (status = 200, description = "OK", body = Me),
        (status = 401, description = "Not signed in", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn get<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Me>, ApiError> {
    Ok(Json(state.hub.me(&caller.context).await?))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    locale: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/me",
    tag = "me",
    summary = "Change the display name or language",
    request_body = ProfileRequest,
    responses(
        (status = 200, description = "OK", body = Me),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PasswordRequest {
    current_password: String,
    new_password: String,
    code: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/me/password",
    tag = "me",
    summary = "Change the password (needs a TOTP code)",
    request_body = PasswordRequest,
    responses(
        (status = 204, description = "Done"),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody),
        (status = 429, description = "Too many requests; see Retry-After", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReauthRequest {
    password: String,
    code: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct RecoveryCodes {
    recovery_codes: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/me/recovery-codes",
    tag = "me",
    summary = "Replace the recovery codes (shown once)",
    request_body = ReauthRequest,
    responses(
        (status = 200, description = "OK", body = RecoveryCodes),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody),
        (status = 429, description = "Too many requests; see Retry-After", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[derive(Serialize, ToSchema)]
pub(crate) struct SessionItem {
    #[serde(flatten)]
    session: Session,
    current: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/me/sessions",
    tag = "me",
    summary = "The caller's open sessions",
    responses(
        (status = 200, description = "OK", body = Items<SessionItem>),
        (status = 401, description = "Not signed in", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/me/sessions/{id}",
    tag = "me",
    summary = "End one of the caller's sessions",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 204, description = "Done"),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn revoke_session<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_my_session(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/me/tokens",
    tag = "me",
    summary = "The caller's API tokens",
    responses(
        (status = 200, description = "OK", body = Items<ApiToken>),
        (status = 401, description = "Not signed in", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn tokens<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Items<ApiToken>>, ApiError> {
    let items = state.hub.my_api_tokens(&caller.context).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TokenRequest {
    name: String,
    days: i64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreatedToken {
    token: ApiToken,
    /// The secret, shown once.
    secret: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/me/tokens",
    tag = "me",
    summary = "Create a personal API token (secret shown once)",
    request_body = TokenRequest,
    responses(
        (status = 201, description = "Created", body = CreatedToken),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/me/tokens/{id}",
    tag = "me",
    summary = "Revoke one of the caller's API tokens",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 204, description = "Done"),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn revoke_token<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.hub.revoke_api_token(&caller.context, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
