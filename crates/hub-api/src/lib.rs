//! HTTP surface of the Admin Hub (surface `api`): the REST API under `/api/v1` and the
//! health endpoints. Handlers translate HTTP into core use cases and back; no rule lives here.

mod administrators;
mod audit;
mod auth;
mod body;
mod caller;
mod error;
mod health;
mod instance;
mod limiter;
mod me;
mod openapi;
mod organizations;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use hub_core::ports::{Clock, Crypto, Store};
use hub_core::Hub;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub use caller::{CSRF_HEADER, SESSION_COOKIE};
pub use error::{ApiError, ErrorBody};
pub use limiter::{RateLimiter, UNAUTHENTICATED_PER_MINUTE};
pub use openapi::{document as openapi_document, document_json as openapi_json, CONTRACT_VERSION};

/// Largest request body accepted, in bytes.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Longest time a request may take before it is answered with 408.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Readiness of the infrastructure behind the API (database, key file).
pub trait Readiness: Send + Sync + 'static {
    /// `Err` carries a short reason that is safe to show to anyone.
    fn check(&self) -> impl Future<Output = Result<(), String>> + Send;
}

/// The adapters the API runs on, named once so handlers take one type parameter.
pub trait Backend: Send + Sync + 'static {
    type Store: Store + Send + Sync;
    type Crypto: Crypto;
    type Clock: Clock + 'static;
    type Readiness: Readiness;
}

/// Shared state of the handlers.
pub struct AppState<B: Backend> {
    pub hub: Hub<B::Store, B::Crypto, B::Clock>,
    pub readiness: Arc<B::Readiness>,
    pub limiter: Arc<RateLimiter>,
}

impl<B: Backend> AppState<B> {
    pub fn new(hub: Hub<B::Store, B::Crypto, B::Clock>, readiness: Arc<B::Readiness>) -> Self {
        Self {
            hub,
            readiness,
            limiter: Arc::new(RateLimiter::new(UNAUTHENTICATED_PER_MINUTE)),
        }
    }
}

impl<B: Backend> Clone for AppState<B> {
    fn clone(&self) -> Self {
        Self {
            hub: self.hub.clone(),
            readiness: Arc::clone(&self.readiness),
            limiter: Arc::clone(&self.limiter),
        }
    }
}

fn api<B: Backend>() -> Router<AppState<B>> {
    Router::new()
        .route("/openapi.json", get(openapi_file))
        .route("/instance", get(instance::status::<B>))
        .route(
            "/instance/settings",
            get(instance::settings::<B>).put(instance::update_settings::<B>),
        )
        .route("/auth/login", post(auth::login::<B>))
        .route("/auth/logout", post(auth::logout::<B>))
        .route("/auth/links/inspect", post(auth::inspect_link::<B>))
        .route("/auth/links/enrollment", post(auth::begin_enrollment::<B>))
        .route("/auth/links/complete", post(auth::complete_link::<B>))
        .route("/me", get(me::get::<B>).patch(me::update::<B>))
        .route("/me/password", post(me::change_password::<B>))
        .route(
            "/me/recovery-codes",
            post(me::regenerate_recovery_codes::<B>),
        )
        .route("/me/sessions", get(me::sessions::<B>))
        .route("/me/sessions/{id}", delete(me::revoke_session::<B>))
        .route(
            "/me/tokens",
            get(me::tokens::<B>).post(me::create_token::<B>),
        )
        .route("/me/tokens/{id}", delete(me::revoke_token::<B>))
        .route(
            "/organizations",
            get(organizations::list::<B>).post(organizations::create::<B>),
        )
        .route(
            "/organizations/{id}",
            get(organizations::get::<B>).patch(organizations::rename::<B>),
        )
        .route(
            "/organizations/{id}/archive",
            post(organizations::archive::<B>),
        )
        .route(
            "/organizations/{id}/members",
            get(organizations::members::<B>).post(organizations::add_member::<B>),
        )
        .route(
            "/organizations/{id}/members/{administrator}",
            put(organizations::change_role::<B>).delete(organizations::remove_member::<B>),
        )
        .route(
            "/administrators/{id}/suspend",
            post(administrators::suspend::<B>),
        )
        .route(
            "/administrators/{id}/reactivate",
            post(administrators::reactivate::<B>),
        )
        .route(
            "/administrators/{id}/reset-second-factor",
            post(administrators::reset_second_factor::<B>),
        )
        .route(
            "/administrators/{id}/revoke-sessions",
            post(administrators::revoke_sessions::<B>),
        )
        .route("/audit", get(audit::list::<B>))
        .route("/audit/verify", get(audit::verify::<B>))
        .fallback(error::not_found)
}

/// The published contract, served so integrators read exactly what this Hub accepts.
async fn openapi_file() -> impl axum::response::IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        openapi::document_json(),
    )
}

/// The complete router of the Hub, with its middleware.
pub fn router<B: Backend>(state: AppState<B>) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        ));

    Router::new()
        .route("/healthz", get(health::live))
        .route("/readyz", get(health::ready::<B>))
        .nest("/api/v1", api::<B>())
        .fallback(error::not_found)
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware)
}
