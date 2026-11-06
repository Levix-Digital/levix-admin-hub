//! The administrative audit: pages of records, newest first, and chain verification.

use axum::extract::State;
use axum::Json;
use hub_core::audit::{AuditRecord, ChainReport, AUDIT_PAGE_MAX};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::body::QueryParams;
use crate::caller::Caller;
use crate::{ApiError, AppState, Backend};

const DEFAULT_PAGE: u32 = 100;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuditParams {
    organization: Option<Uuid>,
    /// Records with a sequence lower than this one (the `next_before` of the last page).
    before: Option<i64>,
    limit: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct AuditPage {
    items: Vec<AuditRecord>,
    /// Pass as `before` to read the next page; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_before: Option<i64>,
}

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

pub(crate) async fn verify<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<ChainReport>, ApiError> {
    Ok(Json(state.hub.verify_audit(&caller.context).await?))
}
