//! Who is calling: a console session (cookie plus CSRF token on writes), a CLI session or
//! a personal API token (bearer), or nobody (threat model T-04, T-05).

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap, HeaderValue, Method};
use hub_core::context::Context;
use hub_core::session::{check_csrf, Session, SessionKind, API_TOKEN_PREFIX};

use crate::{ApiError, AppState, Backend};

/// Name of the console session cookie (`__Host-`: secure, whole site, no domain).
pub const SESSION_COOKIE: &str = "__Host-lvx_session";
/// Header carrying the console session's CSRF token on writes.
pub const CSRF_HEADER: &str = "x-csrf-token";

/// The address of the peer, when the server knows it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Source(pub Option<IpAddr>);

impl Source {
    pub(crate) fn context(self) -> Context {
        Context::anonymous(self.0.map(|ip| ip.to_string()))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Source {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|info| info.0.ip()),
        ))
    }
}

/// The client's `User-Agent`, if any.
pub(crate) fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
}

fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty())
}

/// `Set-Cookie` value opening a console session.
pub(crate) fn session_cookie(token: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={token}; Path=/; Secure; HttpOnly; SameSite=Strict"
    ))
    .ok()
}

/// `Set-Cookie` value removing the console session cookie.
pub(crate) fn cleared_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "__Host-lvx_session=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0",
    )
}

/// An authenticated caller.
#[derive(Debug, Clone)]
pub(crate) struct Caller {
    pub context: Context,
    pub session: Option<Session>,
}

impl<B: Backend> FromRequestParts<AppState<B>> for Caller {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState<B>) -> Result<Self, ApiError> {
        let Ok(Source(address)) = Source::from_request_parts(parts, state).await;
        let address = address.map(|ip| ip.to_string());
        let hub = &state.hub;

        if let Some(token) = bearer(&parts.headers) {
            if token.starts_with(API_TOKEN_PREFIX) {
                let context = hub.authenticate_api_token(token, address).await?;
                return Ok(Self {
                    context,
                    session: None,
                });
            }
            let (context, session) = hub.authenticate_session(token, address).await?;
            if session.kind != SessionKind::Cli {
                return Err(ApiError::unauthenticated());
            }
            return Ok(Self {
                context,
                session: Some(session),
            });
        }

        let token = cookie(&parts.headers, SESSION_COOKIE).ok_or_else(ApiError::unauthenticated)?;
        let (context, session) = hub.authenticate_session(token, address).await?;
        if session.kind != SessionKind::Console {
            return Err(ApiError::unauthenticated());
        }
        let safe = matches!(parts.method, Method::GET | Method::HEAD | Method::OPTIONS);
        if !safe {
            let csrf = parts.headers.get(CSRF_HEADER).and_then(|v| v.to_str().ok());
            check_csrf(&session, csrf)?;
        }
        Ok(Self {
            context,
            session: Some(session),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_named_cookie_only() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("a=1; __Host-lvx_session=lvxs_abc; b=2"),
        );
        assert_eq!(cookie(&headers, SESSION_COOKIE), Some("lvxs_abc"));
        assert_eq!(cookie(&headers, "missing"), None);
    }

    #[test]
    fn reads_bearer_tokens() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer lvxh_x"),
        );
        assert_eq!(bearer(&headers), Some("lvxh_x"));
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic abc"));
        assert_eq!(bearer(&headers), None);
    }
}
