//! T024: the logs of an end-to-end run, at the most verbose level, contain no password,
//! TOTP secret, link, session, CSRF or API token, or recovery code.

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use hub_server::cli::{InitArgs, PublicUrl, TlsArgs};
use hub_server::{init, logging, serve};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PASSWORD: &str = "correct horse battery staple";
const WRONG_PASSWORD: &str = "a wrong but secret password";

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Buffer {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap_or_else(|e| e.into_inner())).into_owned()
    }
}

async fn request(
    address: SocketAddr,
    method: &str,
    path: &str,
    auth: Option<&str>,
    body: Option<Value>,
) -> Value {
    let mut stream = ok(TcpStream::connect(address).await);
    let body = body.map(|b| b.to_string()).unwrap_or_default();
    let auth = auth
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{auth}\
         Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    ok(stream.write_all(request.as_bytes()).await);
    let mut response = String::new();
    ok(stream.read_to_string(&mut response).await);
    let body = response.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    serde_json::from_str(body).unwrap_or(Value::Null)
}

fn totp(secret: &[u8], offset_steps: i64) -> String {
    let step = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .div_euclid(30)
        + offset_steps;
    hub_crypto::totp::code_at_step(secret, step).unwrap_or_default()
}

#[tokio::test(flavor = "current_thread")]
async fn logs_of_an_end_to_end_run_hold_no_secret() {
    let buffer = Buffer::default();
    let writer = buffer.clone();
    let _guard =
        tracing::subscriber::set_default(logging::subscriber("trace", move || writer.clone()));

    let dir = ok(tempfile::tempdir());
    let app = ok(serve::build(dir.path()).await);
    let listener = ok(TcpListener::bind("127.0.0.1:0").await);
    let address = ok(listener.local_addr());
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let args = InitArgs {
        owner_email: "owner@example.com".to_owned(),
        url: PublicUrl { public_url: None },
        tls: TlsArgs {
            tls_cert: None,
            tls_key: None,
            hostnames: vec!["localhost".to_owned()],
        },
    };
    let link = ok(init::initialize(dir.path(), &args).await)
        .setup_link
        .token;
    let mut secrets = vec![PASSWORD.to_owned(), WRONG_PASSWORD.to_owned(), link.clone()];

    let token = json!({ "token": link });
    request(
        address,
        "POST",
        "/api/v1/auth/links/inspect",
        None,
        Some(token.clone()),
    )
    .await;
    let enrollment = request(
        address,
        "POST",
        "/api/v1/auth/links/enrollment",
        None,
        Some(token),
    )
    .await;
    let encoded = enrollment["secret"].as_str().unwrap_or_default().to_owned();
    let secret = ok(data_encoding::BASE32_NOPAD.decode(encoded.as_bytes()));
    secrets.push(encoded);

    let done = request(
        address,
        "POST",
        "/api/v1/auth/links/complete",
        None,
        Some(json!({ "token": link, "password": PASSWORD, "code": totp(&secret, -1), "client": "cli" })),
    )
    .await;
    let session = done["token"].as_str().unwrap_or_default().to_owned();
    assert!(session.starts_with("lvxs_"), "{done}");
    secrets.push(session.clone());
    for code in done["recovery_codes"].as_array().into_iter().flatten() {
        secrets.push(code.as_str().unwrap_or_default().to_owned());
    }

    request(
        address,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "owner@example.com", "password": WRONG_PASSWORD, "code": "000000" })),
    )
    .await;
    let console = request(
        address,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(
            json!({ "email": "owner@example.com", "password": PASSWORD, "code": totp(&secret, 1) }),
        ),
    )
    .await;
    assert_eq!(console["status"], "signed_in", "{console}");
    secrets.push(
        console["csrf_token"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    );

    request(address, "GET", "/api/v1/me", Some(&session), None).await;
    let created = request(
        address,
        "POST",
        "/api/v1/me/tokens",
        Some(&session),
        Some(json!({ "name": "ci", "days": 7 })),
    )
    .await;
    let api_token = created["secret"].as_str().unwrap_or_default().to_owned();
    assert!(api_token.starts_with("lvxh_"), "{created}");
    secrets.push(api_token.clone());
    request(
        address,
        "GET",
        "/api/v1/organizations",
        Some(&api_token),
        None,
    )
    .await;

    let logs = buffer.text();
    assert!(
        logs.contains("instance initialized"),
        "the run logged nothing useful"
    );
    assert!(
        logs.lines().count() > 10,
        "too few log lines to be meaningful"
    );
    for secret in secrets.iter().filter(|s| !s.is_empty()) {
        assert!(!logs.contains(secret.as_str()), "a secret reached the logs");
    }
}
