//! A real Hub over TLS in this process, and the CLI binary run against it.

#![allow(dead_code)]

use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use hub_server::cli::{InitArgs, PublicUrl, TlsArgs};
use hub_server::{init, serve, tls};
use serde_json::Value;

pub fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

pub struct Hub {
    pub url: String,
    pub fingerprint: String,
    pub certificate: PathBuf,
    pub setup_link: String,
    _data: tempfile::TempDir,
    _runtime: tokio::runtime::Runtime,
}

impl Hub {
    pub fn start() -> Self {
        let runtime = ok(tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build());
        let data = ok(tempfile::tempdir());
        let args = InitArgs {
            owner_email: "owner@example.com".to_owned(),
            url: PublicUrl { public_url: None },
            tls: TlsArgs {
                tls_cert: None,
                tls_key: None,
                hostnames: vec!["localhost".to_owned()],
            },
        };
        let dir = data.path().to_owned();
        let (initialized, files, address) = runtime.block_on(async {
            let initialized = ok(init::initialize(&dir, &args).await);
            let files = ok(tls::prepare(&dir, &args.tls));
            let config = ok(tls::server_config(&files).await);
            let app = ok(serve::build(&dir).await);
            let listener = ok(std::net::TcpListener::bind("127.0.0.1:0"));
            let address = ok(listener.local_addr());
            ok(listener.set_nonblocking(true));
            let server = ok(axum_server::from_tcp_rustls(listener, config));
            tokio::spawn(async move {
                if let Err(error) = server
                    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                    .await
                {
                    panic!("server stopped: {error}");
                }
            });
            (initialized, files, address)
        });
        let url = format!("https://localhost:{}", address.port());
        Self {
            setup_link: init::link_url(&url, &initialized.setup_link.token),
            url,
            fingerprint: initialized.fingerprint,
            certificate: files.certificate,
            _data: data,
            _runtime: runtime,
        }
    }
}

/// One user of the CLI: their own configuration directory and authenticator.
pub struct User {
    pub config: tempfile::TempDir,
    pub secret: Vec<u8>,
    last_step: i64,
}

pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("{e}: {}\n{}", self.stdout, self.stderr))
    }

    pub fn success(self) -> Self {
        assert_eq!(
            self.code, 0,
            "stdout: {}\nstderr: {}",
            self.stdout, self.stderr
        );
        self
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl User {
    pub fn new() -> Self {
        Self {
            config: ok(tempfile::tempdir()),
            secret: Vec::new(),
            last_step: 0,
        }
    }

    pub fn session_file(&self) -> PathBuf {
        self.config.path().join("session.json")
    }

    /// A code the Hub accepts and has not seen yet (codes are single use).
    pub fn next_code(&mut self) -> String {
        loop {
            let now = ok(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH));
            let current = i64::try_from(now.as_secs()).unwrap_or_default() / 30;
            let step = current.max(self.last_step + 1);
            if step <= current + 1 {
                self.last_step = step;
                return hub_crypto::totp::code_at_step(&self.secret, step)
                    .unwrap_or_else(|| panic!("no code"));
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn run(&self, args: &[&str], stdin: Option<&str>, env: &[(&str, &str)]) -> Run {
        run_in(self.config.path(), args, stdin, env)
    }

    /// Enrolls the authenticator of a link and completes it.
    pub fn complete(&mut self, hub: &Hub, link: &str, name: &str) -> Run {
        let enrollment = self
            .run(
                &[
                    "--fingerprint",
                    &hub.fingerprint,
                    "--output",
                    "json",
                    "link",
                    "enroll",
                    link,
                ],
                None,
                &[],
            )
            .success()
            .json();
        let secret = enrollment["secret"].as_str().unwrap_or_default();
        self.secret = ok(data_encoding::BASE32_NOPAD.decode(secret.as_bytes()));
        let code = self.next_code();
        self.run(
            &[
                "--fingerprint",
                &hub.fingerprint,
                "link",
                "complete",
                link,
                "--password-stdin",
                "--code",
                &code,
                "--display-name",
                name,
            ],
            Some("correct horse battery staple\n"),
            &[],
        )
    }
}

pub fn run_in(config: &Path, args: &[&str], stdin: Option<&str>, env: &[(&str, &str)]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_levix-hub-cli"));
    for variable in [
        "LEVIX_HUB_URL",
        "LEVIX_HUB_FINGERPRINT",
        "LEVIX_HUB_CA_FILE",
        "LEVIX_HUB_TOKEN",
    ] {
        command.env_remove(variable);
    }
    command
        .env("LEVIX_HUB_CLI_CONFIG_DIR", config)
        .envs(env.iter().copied())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = ok(command.spawn());
    if let Some(mut input) = child.stdin.take() {
        if let Some(text) = stdin {
            ok(input.write_all(text.as_bytes()));
        }
    }
    let output = ok(child.wait_with_output());
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}
