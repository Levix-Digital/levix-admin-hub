//! End-to-end tests of the server binary's building blocks.

use std::path::Path;

use hub_server::cli::TlsArgs;
use hub_server::{init, serve};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

fn tls_args() -> TlsArgs {
    TlsArgs {
        tls_cert: None,
        tls_key: None,
        hostnames: vec!["localhost".to_owned()],
    }
}

async fn http_get(address: std::net::SocketAddr, path: &str) -> String {
    let mut stream = ok(TcpStream::connect(address).await);
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    ok(stream.write_all(request.as_bytes()).await);
    let mut response = String::new();
    ok(stream.read_to_string(&mut response).await);
    response
}

async fn spawn(data_dir: &Path) -> std::net::SocketAddr {
    let app = ok(serve::build(data_dir).await);
    let listener = ok(TcpListener::bind("127.0.0.1:0").await);
    let address = ok(listener.local_addr());
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            panic!("server stopped: {error}");
        }
    });
    address
}

#[tokio::test]
async fn init_runs_once_and_creates_the_key_and_certificate() {
    let dir = ok(tempfile::tempdir());
    let first = ok(init::initialize(dir.path(), &tls_args()).await);
    assert!(!first.fingerprint.is_empty());
    assert!(dir.path().join("keys/secret.key").exists());
    assert!(dir.path().join("tls/certificate.pem").exists());

    let second = init::initialize(dir.path(), &tls_args()).await;
    assert!(second.is_err());
}

#[tokio::test]
async fn serves_before_and_after_init() {
    let dir = ok(tempfile::tempdir());
    let address = spawn(dir.path()).await;

    let response = http_get(address, "/api/v1/instance").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains(r#""initialized":false"#), "{response}");
    assert!(http_get(address, "/readyz")
        .await
        .starts_with("HTTP/1.1 200"));

    ok(init::initialize(dir.path(), &tls_args()).await);
    let response = http_get(address, "/api/v1/instance").await;
    assert!(response.contains(r#""initialized":true"#), "{response}");
    assert!(http_get(address, "/readyz")
        .await
        .starts_with("HTTP/1.1 200"));

    // Readiness reports a missing key file once the Hub is initialized.
    ok(std::fs::remove_file(dir.path().join("keys/secret.key")));
    assert!(http_get(address, "/readyz")
        .await
        .starts_with("HTTP/1.1 503"));
}

/// The Hub opens no outbound connection (ADR-0004, FR-014): after serving requests, every TCP
/// socket this process owns is a listener or talks to the loopback address.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn opens_no_outbound_connection() {
    let dir = ok(tempfile::tempdir());
    let address = spawn(dir.path()).await;
    ok(init::initialize(dir.path(), &tls_args()).await);
    for path in ["/healthz", "/readyz", "/api/v1/instance"] {
        let _ = http_get(address, path).await;
    }
    let remotes = outbound::remote_addresses_of_this_process();
    // The listener itself must be seen, or the check reads nothing.
    assert!(!remotes.is_empty(), "no socket of this process found");
    let foreign: Vec<_> = remotes
        .iter()
        .filter(|remote| !remote.is_unspecified() && !remote.is_loopback())
        .collect();
    assert!(foreign.is_empty(), "outbound connections: {foreign:?}");
}

#[cfg(target_os = "linux")]
mod outbound {
    use std::collections::HashSet;
    use std::fs;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn socket_inodes() -> HashSet<String> {
        let mut inodes = HashSet::new();
        if let Ok(entries) = fs::read_dir("/proc/self/fd") {
            for entry in entries.flatten() {
                if let Ok(target) = fs::read_link(entry.path()) {
                    let target = target.to_string_lossy().into_owned();
                    if let Some(inode) = target
                        .strip_prefix("socket:[")
                        .and_then(|rest| rest.strip_suffix(']'))
                    {
                        inodes.insert(inode.to_owned());
                    }
                }
            }
        }
        inodes
    }

    fn parse_v4(hex: &str) -> Option<IpAddr> {
        let value = u32::from_str_radix(hex, 16).ok()?;
        Some(IpAddr::V4(Ipv4Addr::from(value.swap_bytes())))
    }

    fn parse_v6(hex: &str) -> Option<IpAddr> {
        let mut bytes = [0u8; 16];
        for (word, chunk) in hex.as_bytes().chunks(8).enumerate() {
            let value = u32::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
            bytes[word * 4..word * 4 + 4].copy_from_slice(&value.swap_bytes().to_be_bytes());
        }
        let address = Ipv6Addr::from(bytes);
        Some(match address.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(address),
        })
    }

    pub fn remote_addresses_of_this_process() -> Vec<IpAddr> {
        let inodes = socket_inodes();
        let mut remotes = Vec::new();
        for (table, parse) in [
            ("/proc/self/net/tcp", parse_v4 as fn(&str) -> Option<IpAddr>),
            ("/proc/self/net/tcp6", parse_v6),
        ] {
            let Ok(contents) = fs::read_to_string(table) else {
                continue;
            };
            for line in contents.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                let (Some(remote), Some(inode)) = (fields.get(2), fields.get(9)) else {
                    continue;
                };
                if !inodes.contains(*inode) {
                    continue;
                }
                if let Some(address) = remote.split(':').next().and_then(parse) {
                    remotes.push(address);
                }
            }
        }
        remotes
    }
}

#[tokio::test]
async fn init_resumes_after_an_interruption_before_the_instance() {
    let dir = ok(tempfile::tempdir());
    // An earlier init created the key file and stopped.
    ok(hub_crypto::key_file::create(
        &dir.path().join("keys/secret.key"),
    ));
    assert!(init::initialize(dir.path(), &tls_args()).await.is_ok());
}

#[cfg(unix)]
#[tokio::test]
async fn data_directory_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = ok(tempfile::tempdir());
    let data = dir.path().join("data");
    ok(init::initialize(&data, &tls_args()).await);
    let mode = ok(std::fs::metadata(&data)).permissions().mode();
    assert_eq!(mode & 0o077, 0);
}
