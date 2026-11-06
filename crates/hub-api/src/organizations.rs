//! Organizations and their members.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use hub_core::administrator::Member;
use hub_core::organization::{NewOrganization, Organization};
use hub_core::role::Role;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::LinkBody;
use crate::body::{JsonBody, PathParams};
use crate::caller::Caller;
use crate::me::Items;
use crate::{ApiError, AppState, Backend, ErrorBody};

#[utoipa::path(
    get,
    path = "/api/v1/organizations",
    tag = "organizations",
    summary = "Organizations visible to the caller",
    responses(
        (status = 200, description = "OK", body = Items<Organization>),
        (status = 401, description = "Not signed in", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn list<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Json<Items<Organization>>, ApiError> {
    let items = state.hub.organizations(&caller.context).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateRequest {
    slug: String,
    name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    tag = "organizations",
    summary = "Create an organization (instance owner)",
    request_body = CreateRequest,
    responses(
        (status = 201, description = "Created", body = Organization),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{id}",
    tag = "organizations",
    summary = "One organization",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = Organization),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn get<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Organization>, ApiError> {
    Ok(Json(state.hub.organization(&caller.context, id).await?))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RenameRequest {
    name: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{id}",
    tag = "organizations",
    summary = "Rename an organization (owners)",
    request_body = RenameRequest,
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = Organization),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{id}/archive",
    tag = "organizations",
    summary = "Archive an organization (owners)",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = Organization),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn archive<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Organization>, ApiError> {
    Ok(Json(
        state.hub.archive_organization(&caller.context, id).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{id}/members",
    tag = "members",
    summary = "Administrators of an organization with their roles",
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 200, description = "OK", body = Items<Member>),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
pub(crate) async fn members<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
    PathParams(id): PathParams<Uuid>,
) -> Result<Json<Items<Member>>, ApiError> {
    let items = state.hub.members(&caller.context, id).await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddMemberRequest {
    email: String,
    role: Role,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AddedBody {
    member: Member,
    /// The invitation link, shown once, when the e-mail was not an administrator yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    invitation: Option<LinkBody>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{id}/members",
    tag = "members",
    summary = "Give a role to an e-mail; invites it when new (owners)",
    request_body = AddMemberRequest,
    params(("id" = Uuid, Path, description = "Identifier")),
    responses(
        (status = 201, description = "Created", body = AddedBody),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleRequest {
    role: Role,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{id}/members/{administrator}",
    tag = "members",
    summary = "Change a member's role (owners)",
    request_body = RoleRequest,
    params(("id" = Uuid, Path, description = "Organization"), ("administrator" = Uuid, Path, description = "Administrator")),
    responses(
        (status = 200, description = "OK", body = Member),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody),
        (status = 422, description = "Invalid input or unknown field", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{id}/members/{administrator}",
    tag = "members",
    summary = "Remove a member from an organization (owners)",
    params(("id" = Uuid, Path, description = "Organization"), ("administrator" = Uuid, Path, description = "Administrator")),
    responses(
        (status = 204, description = "Done"),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing permission or CSRF token", body = ErrorBody),
        (status = 404, description = "Not found or not visible", body = ErrorBody),
        (status = 409, description = "Conflicts with the current state", body = ErrorBody)
    ),
    security(("session_cookie" = [], "csrf_token" = []), ("bearer" = []))
)]
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
