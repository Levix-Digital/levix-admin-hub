//! Organizations and their members.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::Member;
use hub_core::organization::{NewOrganization, Organization};
use hub_core::role::Role;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::LinkBody;
use crate::body::{JsonBody, PathParams};
use crate::caller::Caller;
use crate::me::Items;
use crate::{ApiError, AppState, Backend};

pub(crate) async fn list<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Items<Organization>>, ApiError> {
    let items = state.hub.organizations(&caller.context).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateRequest {
    slug: String,
    name: String,
}

pub(crate) async fn create<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    JsonBody(request): JsonBody<CreateRequest>,
) -> Result<(StatusCode, Json<Organization>), ApiError> {
    let organization = state
        .hub
        .create_organization(
            &caller.context,
            NewOrganization {
                slug: request.slug,
                name: request.name,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(organization)))
}

pub(crate) async fn get<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Organization>, ApiError> {
    Ok(Json(state.hub.organization(&caller.context, id).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RenameRequest {
    name: String,
}

pub(crate) async fn rename<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
    JsonBody(request): JsonBody<RenameRequest>,
) -> Result<Json<Organization>, ApiError> {
    Ok(Json(
        state
            .hub
            .rename_organization(&caller.context, id, &request.name)
            .await?,
    ))
}

pub(crate) async fn archive<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Organization>, ApiError> {
    Ok(Json(
        state.hub.archive_organization(&caller.context, id).await?,
    ))
}

pub(crate) async fn members<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Items<Member>>, ApiError> {
    let items = state.hub.members(&caller.context, id).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddMemberRequest {
    email: String,
    role: Role,
}

#[derive(Serialize)]
pub(crate) struct AddedBody {
    member: Member,
    /// The invitation link, shown once, when the e-mail was not an administrator yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    invitation: Option<LinkBody>,
}

pub(crate) async fn add_member<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
    JsonBody(request): JsonBody<AddMemberRequest>,
) -> Result<(StatusCode, Json<AddedBody>), ApiError> {
    let added = state
        .hub
        .add_member(&caller.context, id, &request.email, request.role)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(AddedBody {
            member: added.member,
            invitation: added.invitation.map(LinkBody::from),
        }),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleRequest {
    role: Role,
}

pub(crate) async fn change_role<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams((id, administrator)): PathParams<(Uuid, Uuid)>,
    JsonBody(request): JsonBody<RoleRequest>,
) -> Result<Json<Member>, ApiError> {
    Ok(Json(
        state
            .hub
            .change_role(&caller.context, id, administrator, request.role)
            .await?,
    ))
}

pub(crate) async fn remove_member<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams((id, administrator)): PathParams<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .hub
        .remove_member(&caller.context, id, administrator)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
