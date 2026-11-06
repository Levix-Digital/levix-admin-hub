//! The administrative audit: pages of records, newest first, and chain verification.

use axum::extract::State;
use axum::Json;
use hub_core::audit::{AuditRecord, ChainReport, AUDIT_PAGE_MAX};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::body::QueryParams;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend, ErrorBody};

const DEFAULT_PAGE: u32 = 100;

#[derive(Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub(crate) struct AuditParams {
    organization: Option<Uuid>,
    /// Records with a sequence lower than this one (the `next_before` of the last page).
    before: Option<i64>,
    limit: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AuditPage {
    items: Vec<AuditRecord>,
    /// Pass as `before` to read the next page; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_before: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/audit",
    tag = "audit",
    summary = "Audit records the caller may read, newest first",
    params(AuditParams),
    responses(
        (status = 200, description = "OK", body = AuditPage),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn list<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    QueryParams(params): QueryParams<AuditParams>,
) -> Result<Json<AuditPage>, ApiError> {
    let limit = params
        .limit
        .unwrap_or(DEFAULT_PAGE)
        .clamp(1, AUDIT_PAGE_MAX);
    let items = state
        .hub
        .audit_records(&caller.context, params.organization, params.before, limit)
        .await?;
    let full = u32::try_from(items.len()).is_ok_and(|n| n >= limit);
    let next_before = full.then(|| items.last().map(|r| r.seq)).flatten();
    Ok(Json(AuditPage { items, next_before }))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/verify",
    tag = "audit",
    summary = "Verify the hash chain of the whole audit",
    responses(
        (status = 200, description = "OK", body = ChainReport),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn verify<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<ChainReport>, ApiError> {
    Ok(Json(state.hub.verify_audit(&caller.context).await?))
}
