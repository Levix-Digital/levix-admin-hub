//! `levix-hub init`: initializes the Hub on its host, once. Holding the host is the trust
//! anchor: no account can be created over the network (threat model T-01).

use std::path::Path;

use anyhow::Context;
use hub_core::instance;
use hub_core::ports::SystemClock;
use hub_core::CoreError;
use hub_storage_sqlite::SqliteStore;

use crate::cli::TlsArgs;
use crate::{paths, tls};

/// What `init` created, for the caller and the tests.
#[derive(Debug)]
pub struct Initialized {
    pub fingerprint: String,
}

pub async fn run(data_dir: &Path, args: TlsArgs) -> anyhow::Result<()> {
    let result = initialize(data_dir, &args).await?;
    println!("The Hub is initialized.");
    println!(
        "TLS certificate SHA-256 fingerprint: {}",
        result.fingerprint
    );
    println!("Check this fingerprint in your browser before signing in.");
    Ok(())
}

pub async fn initialize(data_dir: &Path, args: &TlsArgs) -> anyhow::Result<Initialized> {
    let store = SqliteStore::open(&paths::database(data_dir))
        .await
        .context("opening the database")?;
    if instance::status(&store).await?.initialized {
        anyhow::bail!(CoreError::AlreadyInitialized);
    }
    hub_crypto::key_file::create(&paths::secret_key(data_dir))
        .context("creating the secret key")?;
    let instance = instance::initialize(&store, &SystemClock).await?;
    tracing::info!(instance = %instance.id, actor = "host", action = "instance.initialize", "instance initialized");

    let files = tls::prepare(data_dir, args)?;
    Ok(Initialized {
        fingerprint: tls::fingerprint(&files.certificate)?,
    })
}
