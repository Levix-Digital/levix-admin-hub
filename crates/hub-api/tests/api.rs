//! HTTP tests of the API against a temporary SQLite database and the real cryptography.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use data_encoding::BASE32_NOPAD;
use http_body_util::BodyExt;
use hub_api::{router, AppState, Backend, Readiness, CSRF_HEADER, UNAUTHENTICATED_PER_MINUTE};
use hub_core::context::Context;
use hub_core::ports::Clock;
use hub_core::Hub;
use hub_crypto::{key_file, HubCrypto};
use hub_storage_sqlite::SqliteStore;
use serde_json::{json, Value};
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery staple";

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

struct TestClock(Mutex<OffsetDateTime>);

impl Clock for TestClock {
    fn now(&self) -> OffsetDateTime {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

struct Test;

impl Backend for Test {
    type Store = SqliteStore;
    type Crypto = HubCrypto;
    type Clock = TestClock;
    type Readiness = Switch;
}

struct Response {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: Value,
}

/// How a request authenticates.
#[derive(Clone, Default)]
enum Auth {
    #[default]
    None,
    Bearer(String),
    Cookie {
        token: String,
        csrf: Option<String>,
    },
}

struct Fixture {
    _dir: tempfile::TempDir,
    state: AppState<Test>,
    readiness: Arc<Switch>,
    clock: Arc<TestClock>,
}

impl Fixture {
    async fn new() -> Self {
        let dir = ok(tempfile::tempdir());
        let store = ok(SqliteStore::open(&dir.path().join("hub.db")).await);
        let key = ok(key_file::create(&dir.path().join("secret.key")));
        let clock = Arc::new(TestClock(Mutex::new(time::macros::datetime!(
            2026-11-06 12:00:00 UTC
        ))));
        let readiness = Arc::new(Switch(AtomicBool::new(true)));
        let hub = Hub::new(store, Arc::new(HubCrypto::new(&key)), Arc::clone(&clock));
        Self {
            _dir: dir,
            state: AppState::new(hub, Arc::clone(&readiness)),
            readiness,
            clock,
        }
    }

    fn advance(&self, by: Duration) {
        *self.clock.0.lock().unwrap_or_else(|e| e.into_inner()) += by;
    }

    /// The next authenticator code (moves the clock one step so codes never repeat).
    fn code(&self, secret: &[u8]) -> String {
        self.advance(Duration::seconds(30));
        let step = self.clock.now().unix_timestamp().div_euclid(30);
        hub_crypto::totp::code_at_step(secret, step).unwrap_or_default()
    }

    async fn send(&self, method: Method, uri: &str, auth: &Auth, body: Option<Value>) -> Response {
        let mut request = Request::builder().method(method).uri(uri);
        match auth {
            Auth::None => {}
            Auth::Bearer(token) => {
                request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
            }
            Auth::Cookie { token, csrf } => {
                request = request.header(header::COOKIE, format!("__Host-lvx_session={token}"));
                if let Some(csrf) = csrf {
                    request = request.header(CSRF_HEADER, csrf);
                }
            }
        }
        let request = match body {
            Some(body) => ok(request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))),
            None => ok(request.body(Body::empty())),
        };
        let response = ok(router(self.state.clone()).oneshot(request).await);
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = ok(response.into_body().collect().await).to_bytes();
        let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        Response {
            status,
            headers,
            body,
        }
    }

    async fn get(&self, uri: &str, auth: &Auth) -> Response {
        self.send(Method::GET, uri, auth, None).await
    }

    async fn post(&self, uri: &str, auth: &Auth, body: Value) -> Response {
        self.send(Method::POST, uri, auth, Some(body)).await
    }

    /// Initializes on the host and completes the setup link over HTTP.
    async fn claim(&self, client: &str) -> (Response, Vec<u8>) {
        let (_, link) = ok(self
            .state
            .hub
            .initialize(&Context::host(), "owner@example.com")
            .await);
        self.complete(&link.token, client).await
    }

    async fn complete(&self, token: &str, client: &str) -> (Response, Vec<u8>) {
        let none = Auth::None;
        let info = self
            .post(
                "/api/v1/auth/links/inspect",
                &none,
                json!({ "token": token }),
            )
            .await;
        assert_eq!(info.status, StatusCode::OK, "{}", info.body);
        let enrollment = self
            .post(
                "/api/v1/auth/links/enrollment",
                &none,
                json!({ "token": token }),
            )
            .await;
        assert!(enrollment.body["uri"]
            .as_str()
            .is_some_and(|uri| uri.starts_with("otpauth://totp/")));
        let secret = ok(BASE32_NOPAD.decode(
            enrollment.body["secret"]
                .as_str()
                .unwrap_or_default()
                .as_bytes(),
        ));
        let code = self.code(&secret);
        let done = self
            .post(
                "/api/v1/auth/links/complete",
                &none,
                json!({ "token": token, "password": PASSWORD, "code": code, "client": client }),
            )
            .await;
        assert_eq!(done.status, StatusCode::OK, "{}", done.body);
        (done, secret)
    }

    async fn cli_owner(&self) -> (Auth, Vec<u8>) {
        let (done, secret) = self.claim("cli").await;
        let token = done.body["token"].as_str().unwrap_or_default().to_owned();
        (Auth::Bearer(token), secret)
    }
}

#[tokio::test]
async fn liveness_and_readiness() {
    let fixture = Fixture::new().await;
    let response = fixture.get("/healthz", &Auth::None).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "ok");
    assert_eq!(
        fixture.get("/readyz", &Auth::None).await.status,
        StatusCode::OK
    );
    fixture.readiness.0.store(false, Ordering::SeqCst);
    let response = fixture.get("/readyz", &Auth::None).await;
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.body["error"]["code"], "not_ready");
}

#[tokio::test]
async fn instance_status_is_public_and_follows_initialization() {
    let fixture = Fixture::new().await;
    let response = fixture.get("/api/v1/instance", &Auth::None).await;
    assert_eq!(response.body["initialized"], false);
    fixture.claim("cli").await;
    let response = fixture.get("/api/v1/instance", &Auth::None).await;
    assert_eq!(response.body["initialized"], true);
}

#[tokio::test]
async fn errors_use_the_error_object() {
    let fixture = Fixture::new().await;
    let response = fixture.get("/api/v1/nothing-here", &Auth::None).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.body["error"]["code"], "not_found");
    let response = fixture.get("/api/v1/me", &Auth::None).await;
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.body["error"]["code"], "unauthenticated");
    let response = fixture
        .get("/api/v1/me", &Auth::Bearer("lvxs_forged".to_owned()))
        .await;
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn security_headers_and_request_id() {
    let fixture = Fixture::new().await;
    let headers = fixture.get("/healthz", &Auth::None).await.headers;
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("x-request-id"));
}

#[tokio::test]
async fn the_console_session_uses_a_strict_cookie_and_csrf_on_writes() {
    let fixture = Fixture::new().await;
    let (done, _) = fixture.claim("console").await;
    assert!(
        done.body.get("token").is_none(),
        "console tokens stay in the cookie"
    );
    assert_eq!(
        done.body["recovery_codes"].as_array().map(Vec::len),
        Some(10)
    );
    let cookie = done.headers[header::SET_COOKIE]
        .to_str()
        .unwrap_or_default()
        .to_owned();
    for flag in [
        "__Host-lvx_session=lvxs_",
        "Secure",
        "HttpOnly",
        "SameSite=Strict",
        "Path=/",
    ] {
        assert!(cookie.contains(flag), "{cookie} lacks {flag}");
    }
    let token = cookie
        .split(';')
        .next()
        .and_then(|pair| pair.split_once('='))
        .map(|(_, value)| value.to_owned())
        .unwrap_or_default();
    let csrf = done.body["csrf_token"].as_str().map(str::to_owned);

    let reading = Auth::Cookie {
        token: token.clone(),
        csrf: None,
    };
    assert_eq!(
        fixture.get("/api/v1/me", &reading).await.status,
        StatusCode::OK
    );
    let organization = json!({ "slug": "acme", "name": "Acme" });
    let refused = fixture
        .post("/api/v1/organizations", &reading, organization.clone())
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let forged = Auth::Cookie {
        token: token.clone(),
        csrf: Some("lvxc_forged".to_owned()),
    };
    let refused = fixture
        .post("/api/v1/organizations", &forged, organization.clone())
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let writing = Auth::Cookie {
        token: token.clone(),
        csrf,
    };
    let created = fixture
        .post("/api/v1/organizations", &writing, organization)
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);

    // A console token is not accepted as a bearer token (that would skip CSRF checks).
    assert_eq!(
        fixture.get("/api/v1/me", &Auth::Bearer(token)).await.status,
        StatusCode::UNAUTHORIZED
    );
    let out = fixture
        .post("/api/v1/auth/logout", &writing, json!({}))
        .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    assert!(out.headers[header::SET_COOKIE]
        .to_str()
        .unwrap_or_default()
        .contains("Max-Age=0"));
    assert_eq!(
        fixture.get("/api/v1/me", &reading).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn the_cli_signs_in_and_manages_organizations_members_and_audit() {
    let fixture = Fixture::new().await;
    let (owner, secret) = fixture.cli_owner().await;

    let login = fixture
        .post(
            "/api/v1/auth/login",
            &Auth::None,
            json!({ "email": "owner@example.com", "password": PASSWORD,
                    "code": fixture.code(&secret), "client": "cli" }),
        )
        .await;
    assert_eq!(login.status, StatusCode::OK, "{}", login.body);
    assert_eq!(login.body["status"], "signed_in");
    assert!(login.body["token"]
        .as_str()
        .is_some_and(|t| t.starts_with("lvxs_")));

    let created = fixture
        .post(
            "/api/v1/organizations",
            &owner,
            json!({ "slug": "acme", "name": "Acme" }),
        )
        .await;
    let id = created.body["id"].as_str().unwrap_or_default().to_owned();
    let unknown = fixture
        .post(
            "/api/v1/organizations",
            &owner,
            json!({ "slug": "beta", "name": "Beta", "content": "not stored" }),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::UNPROCESSABLE_ENTITY);

    let added = fixture
        .post(
            &format!("/api/v1/organizations/{id}/members"),
            &owner,
            json!({ "email": "auditor@example.com", "role": "auditor" }),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    let invitation = added.body["invitation"]["token"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (joined, _) = fixture.complete(&invitation, "cli").await;
    let auditor = Auth::Bearer(joined.body["token"].as_str().unwrap_or_default().to_owned());

    let members = fixture
        .get(&format!("/api/v1/organizations/{id}/members"), &auditor)
        .await;
    assert_eq!(members.body["items"].as_array().map(Vec::len), Some(2));
    let rename = fixture
        .send(
            Method::PATCH,
            &format!("/api/v1/organizations/{id}"),
            &auditor,
            Some(json!({ "name": "Renamed" })),
        )
        .await;
    assert_eq!(rename.status, StatusCode::FORBIDDEN);
    assert_eq!(rename.body["error"]["code"], "forbidden");

    let page = fixture
        .get(
            &format!("/api/v1/audit?organization={id}&limit=2"),
            &auditor,
        )
        .await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    assert_eq!(page.body["items"].as_array().map(Vec::len), Some(2));
    let before = page.body["next_before"].as_i64().unwrap_or_default();
    let rest = fixture
        .get(
            &format!("/api/v1/audit?organization={id}&before={before}"),
            &auditor,
        )
        .await;
    assert!(rest.body["items"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    let verify = fixture.get("/api/v1/audit/verify", &auditor).await;
    assert_eq!(verify.body["intact"], true);
    let everything = fixture.get("/api/v1/audit", &owner).await;
    assert!(everything.body["items"]
        .as_array()
        .is_some_and(|items| items.iter().any(|r| r["action"] == "instance.initialize")));
}

#[tokio::test]
async fn api_tokens_authenticate_and_can_be_revoked() {
    let fixture = Fixture::new().await;
    let (owner, _) = fixture.cli_owner().await;
    let created = fixture
        .post(
            "/api/v1/me/tokens",
            &owner,
            json!({ "name": "ci", "days": 30 }),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let secret = created.body["secret"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let id = created.body["token"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(created.body["token"].get("token_hash").is_none());
    let token = Auth::Bearer(secret);
    assert_eq!(
        fixture.get("/api/v1/organizations", &token).await.status,
        StatusCode::OK
    );
    let revoked = fixture
        .send(
            Method::DELETE,
            &format!("/api/v1/me/tokens/{id}"),
            &owner,
            None,
        )
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    assert_eq!(
        fixture.get("/api/v1/organizations", &token).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn unauthenticated_endpoints_are_limited_per_source_address() {
    let fixture = Fixture::new().await;
    fixture.claim("cli").await;
    let attempt = json!({ "email": "owner@example.com", "password": "wrong password here",
                          "code": "000000", "client": "cli" });
    let mut last = None;
    for _ in 0..UNAUTHENTICATED_PER_MINUTE {
        last = Some(
            fixture
                .post("/api/v1/auth/login", &Auth::None, attempt.clone())
                .await,
        );
    }
    // The account lockout answers first; then the address limit applies to any account.
    assert!(last.is_some_and(
        |r| r.status == StatusCode::TOO_MANY_REQUESTS || r.status == StatusCode::UNAUTHORIZED
    ));
    let limited = fixture
        .post(
            "/api/v1/auth/links/inspect",
            &Auth::None,
            json!({ "token": "x" }),
        )
        .await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(limited.headers.contains_key(header::RETRY_AFTER));
    assert_eq!(limited.body["error"]["code"], "throttled");
}

#[tokio::test]
async fn sessions_list_marks_the_current_one_and_settings_are_owner_only() {
    let fixture = Fixture::new().await;
    let (owner, _) = fixture.cli_owner().await;
    let sessions = fixture.get("/api/v1/me/sessions", &owner).await;
    let items = sessions.body["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["current"], true);
    assert!(items[0].get("token_hash").is_none());

    let settings = json!({ "session_inactivity_minutes": 30, "session_absolute_hours": 8 });
    let updated = fixture
        .send(
            Method::PUT,
            "/api/v1/instance/settings",
            &owner,
            Some(settings),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(updated.body["settings"]["session_absolute_hours"], 8);
    let invalid = fixture
        .send(
            Method::PUT,
            "/api/v1/instance/settings",
            &owner,
            Some(json!({ "session_inactivity_minutes": 1, "session_absolute_hours": 8 })),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
}
