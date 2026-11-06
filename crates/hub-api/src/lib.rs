//! HTTP surface of the Admin Hub (surface `api`): the REST API under `/api/v1` and the
//! health endpoints. Handlers translate HTTP into core use cases and back; no rule lives here.

mod error;
mod health;
mod instance;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::routing::get;
use axum::Router;
use hub_core::ports::{InstanceRepository, OrganizationRepository};
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub use error::{ApiError, ErrorBody};

/// Largest request body accepted, in bytes.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Longest time a request may take before it is answered with 408.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Everything the storage must offer to the API.
pub trait Store: InstanceRepository + OrganizationRepository + Clone + 'static {}
impl<T: InstanceRepository + OrganizationRepository + Clone + 'static> Store for T {}

/// Readiness of the infrastructure behind the API (database, key file).
pub trait Readiness: Send + Sync + 'static {
    /// `Err` carries a short reason that is safe to show to anyone.
    fn check(&self) -> impl Future<Output = Result<(), String>> + Send;
}

/// Shared state of the handlers.
pub struct AppState<S, R> {
    pub store: S,
    pub readiness: Arc<R>,
}

impl<S: Clone, R> Clone for AppState<S, R> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            readiness: Arc::clone(&self.readiness),
        }
    }
}

/// The complete router of the Hub, with its middleware.
pub fn router<S: Store, R: Readiness>(state: AppState<S, R>) -> Router {
    let api = Router::new().route("/instance", get(instance::status::<S, R>));

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
        ));

    Router::new()
        .route("/healthz", get(health::live))
        .route("/readyz", get(health::ready::<S, R>))
        .nest("/api/v1", api)
        .fallback(error::not_found)
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware)
}
