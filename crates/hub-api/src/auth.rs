//! Sign-in, sign-out and credential links. These endpoints accept unauthenticated callers
//! and are limited per source address. Link tokens travel in bodies, never in URLs, so they
//! do not reach access logs.

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use hub_core::credential::{IssuedLink, LinkCompletion, LinkInfo, LinkPurpose, TotpEnrollment};
use hub_core::session::{IssuedSession, Session, SessionKind, SignIn};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::body::JsonBody;
use crate::caller::{cleared_cookie, session_cookie, user_agent, Caller, Source};
use crate::{ApiError, AppState, Backend};

/// Which client opens the session: the console gets a cookie, the CLI a bearer token.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Client {
    #[default]
    Console,
    Cli,
}

impl From<Client> for SessionKind {
    fn from(client: Client) -> Self {
        match client {
            Client::Console => SessionKind::Console,
            Client::Cli => SessionKind::Cli,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoginRequest {
    email: String,
    password: String,
    /// A TOTP code or a recovery code.
    code: String,
    #[serde(default)]
    client: Client,
}

/// A credential link as shown once to whoever must deliver it.
#[derive(Debug, Serialize)]
pub(crate) struct LinkBody {
    pub token: String,
    pub purpose: LinkPurpose,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

impl From<IssuedLink> for LinkBody {
    fn from(issued: IssuedLink) -> Self {
        Self {
            token: issued.token,
            purpose: issued.link.purpose,
            expires_at: issued.link.expires_at,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum SignInBody {
    SignedIn {
        session: Box<Session>,
        /// Bearer token of a CLI session (console sessions use the cookie).
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        /// CSRF token to send in `X-CSRF-Token` on console writes.
        #[serde(skip_serializing_if = "Option::is_none")]
        csrf_token: Option<String>,
        /// New recovery codes, shown once (after completing a link).
        #[serde(skip_serializing_if = "Option::is_none")]
        recovery_codes: Option<Vec<String>>,
    },
    MustReenroll {
        link: LinkBody,
    },
}

/// The response that opens a session: a cookie for the console, a token for the CLI.
fn signed_in(issued: IssuedSession, recovery_codes: Option<Vec<String>>) -> Response {
    let console = issued.session.kind == SessionKind::Console;
    let cookie = console.then(|| session_cookie(&issued.token)).flatten();
    let body = SignInBody::SignedIn {
        session: Box::new(issued.session),
        token: (!console).then_some(issued.token),
        csrf_token: issued.csrf_token,
        recovery_codes,
    };
    let mut response = (StatusCode::OK, Json(body)).into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    response
}

pub(crate) async fn login<B: Backend>(
    State(state): State<AppState<B>>,
    source: Source,
    headers: HeaderMap,
    JsonBody(request): JsonBody<LoginRequest>,
) -> Result<Response, ApiError> {
    state.limiter.check(source.0)?;
    let result = state
        .hub
        .sign_in(
            &source.context(),
            &request.email,
            &request.password,
            &request.code,
            request.client.into(),
            user_agent(&headers).as_deref(),
        )
        .await?;
    Ok(match result {
        SignIn::Session(issued) => signed_in(issued, None),
        SignIn::MustReenroll(link) => {
            Json(SignInBody::MustReenroll { link: link.into() }).into_response()
        }
    })
}

pub(crate) async fn logout<B: Backend>(
    State(state): State<AppState<B>>,
    caller: Caller,
) -> Result<Response, ApiError> {
    state.hub.sign_out(&caller.context).await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cleared_cookie());
    Ok(response)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkRequest {
    token: String,
}

pub(crate) async fn inspect_link<B: Backend>(
    State(state): State<AppState<B>>,
    source: Source,
    JsonBody(request): JsonBody<LinkRequest>,
) -> Result<Json<LinkInfo>, ApiError> {
    state.limiter.check(source.0)?;
    Ok(Json(
        state
            .hub
            .inspect_link(&source.context(), &request.token)
            .await?,
    ))
}

pub(crate) async fn begin_enrollment<B: Backend>(
    State(state): State<AppState<B>>,
    source: Source,
    JsonBody(request): JsonBody<LinkRequest>,
) -> Result<Json<TotpEnrollment>, ApiError> {
    state.limiter.check(source.0)?;
    Ok(Json(
        state
            .hub
            .begin_enrollment(&source.context(), &request.token)
            .await?,
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompleteRequest {
    token: String,
    password: String,
    code: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    client: Client,
}

pub(crate) async fn complete_link<B: Backend>(
    State(state): State<AppState<B>>,
    source: Source,
    headers: HeaderMap,
    JsonBody(request): JsonBody<CompleteRequest>,
) -> Result<Response, ApiError> {
    state.limiter.check(source.0)?;
    let enrolled = state
        .hub
        .complete_link(
            &source.context(),
            LinkCompletion {
                token: &request.token,
                display_name: request.display_name.as_deref(),
                password: &request.password,
                code: &request.code,
            },
            request.client.into(),
            user_agent(&headers).as_deref(),
        )
        .await?;
    Ok(signed_in(enrolled.session, Some(enrolled.recovery_codes)))
}
