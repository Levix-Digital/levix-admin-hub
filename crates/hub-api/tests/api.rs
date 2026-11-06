//! HTTP tests of the API against a temporary SQLite database.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hub_api::{router, AppState, Readiness};
use hub_core::instance;
use hub_core::ports::SystemClock;
use hub_storage_sqlite::SqliteStore;
use tower::ServiceExt;

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

#[derive(Default)]
struct Switch(AtomicBool);

impl Readiness for Switch {
    async fn check(&self) -> Result<(), String> {
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err("database unavailable".to_owned())
        }
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    store: SqliteStore,
    readiness: Arc<Switch>,
}

impl Fixture {
    async fn new() -> Self {
        let dir = ok(tempfile::tempdir());
        let store = ok(SqliteStore::open(&dir.path().join("hub.db")).await);
        Self {
            _dir: dir,
            store,
            readiness: Arc::new(Switch(AtomicBool::new(true))),
        }
    }

    async fn get(&self, uri: &str) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
        let app = router(AppState {
            store: self.store.clone(),
            readiness: Arc::clone(&self.readiness),
        });
        let response = ok(app.oneshot(ok(Request::get(uri).body(Body::empty()))).await);
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = ok(response.into_body().collect().await).to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, headers, json)
    }
}

#[tokio::test]
async fn liveness_and_readiness() {
    let fixture = Fixture::new().await;
    let (status, _, body) = fixture.get("/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");

    let (status, _, _) = fixture.get("/readyz").await;
    assert_eq!(status, StatusCode::OK);

    fixture.readiness.0.store(false, Ordering::SeqCst);
    let (status, _, body) = fixture.get("/readyz").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "not_ready");
}

#[tokio::test]
async fn instance_status_follows_initialization() {
    let fixture = Fixture::new().await;
    let (_, _, body) = fixture.get("/api/v1/instance").await;
    assert_eq!(body["initialized"], false);

    ok(instance::initialize(&fixture.store, &SystemClock).await);
    let (_, _, body) = fixture.get("/api/v1/instance").await;
    assert_eq!(body["initialized"], true);
}

#[tokio::test]
async fn unknown_endpoint_uses_the_error_object() {
    let fixture = Fixture::new().await;
    let (status, _, body) = fixture.get("/api/v1/nothing-here").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn security_headers_and_request_id() {
    let fixture = Fixture::new().await;
    let (_, headers, _) = fixture.get("/healthz").await;
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert!(headers.contains_key("x-request-id"));
}
