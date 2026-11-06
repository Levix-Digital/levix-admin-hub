//! `levix-hub serve`: runs the Hub until SIGTERM or Ctrl-C.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use axum::Router;
use axum_server::Handle;
use hub_api::{router, AppState, Backend};
use hub_core::ports::SystemClock;
use hub_crypto::HubCrypto;
use hub_storage_sqlite::SqliteStore;

use crate::cli::ServeArgs;
use crate::readiness::HubReadiness;
use crate::{hub, paths, tls};

/// The production adapters.
pub struct Server;

impl Backend for Server {
    type Store = SqliteStore;
    type Crypto = HubCrypto;
    type Clock = SystemClock;
    type Readiness = HubReadiness;
}

const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

/// Builds the router of a Hub whose data lives in `data_dir`.
pub async fn build(data_dir: &Path) -> anyhow::Result<Router> {
    let (hub, store) = hub::open(data_dir).await?;
    let readiness = HubReadiness {
        store,
        secret_key: paths::secret_key(data_dir),
    };
    Ok(router(AppState::<Server>::new(hub, Arc::new(readiness))))
}

pub async fn run(data_dir: &Path, args: ServeArgs) -> anyhow::Result<()> {
    let started = Instant::now();
    let app = build(data_dir).await?;
    let handle = Handle::new();
    tokio::spawn(shutdown_on_signal(handle.clone()));

    if args.insecure_behind_proxy {
        tracing::warn!(
            listen = %args.listen,
            "serving plain HTTP: only safe behind a proxy that terminates TLS"
        );
        log_ready(args.listen, started, None);
        return axum_server::bind(args.listen)
            .handle(handle)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .context("serving HTTP");
    }

    let files = tls::prepare(data_dir, &args.tls)?;
    let fingerprint = tls::fingerprint(&files.certificate)?;
    let config = tls::server_config(&files).await?;
    log_ready(args.listen, started, Some(&fingerprint));
    axum_server::bind_rustls(args.listen, config)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("serving HTTPS")
}

fn log_ready(listen: SocketAddr, started: Instant, fingerprint: Option<&str>) {
    tracing::info!(
        %listen,
        tls = fingerprint.is_some(),
        fingerprint = fingerprint.unwrap_or("-"),
        startup_ms = started.elapsed().as_millis() as u64,
        version = env!("CARGO_PKG_VERSION"),
        "listening"
    );
}

async fn shutdown_on_signal(handle: Handle<SocketAddr>) {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
    handle.graceful_shutdown(Some(SHUTDOWN_GRACE));
}
