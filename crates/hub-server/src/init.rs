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
    crate::paths::create_data_dir(data_dir)?;
    let store = SqliteStore::open(&paths::database(data_dir))
        .await
        .context("opening the database")?;
    if instance::status(&store).await?.initialized {
        anyhow::bail!(CoreError::AlreadyInitialized);
    }
    ensure_secret_key(&paths::secret_key(data_dir))?;
    let instance = instance::initialize(&store, &SystemClock).await?;
    tracing::info!(instance = %instance.id, actor = "host", action = "instance.initialize", "instance initialized");

    let files = tls::prepare(data_dir, args)?;
    Ok(Initialized {
        fingerprint: tls::fingerprint(&files.certificate)?,
    })
}

/// Creates the secret key, or keeps a valid one left by an `init` interrupted before the
/// instance was recorded, so a retry succeeds.
fn ensure_secret_key(path: &Path) -> anyhow::Result<()> {
    match hub_crypto::key_file::create(path) {
        Ok(_) => Ok(()),
        Err(hub_crypto::key_file::KeyFileError::AlreadyExists(_)) => {
            hub_crypto::key_file::load(path)
                .map(|_| ())
                .context("reusing the secret key left by an interrupted initialization")
        }
        Err(error) => Err(error).context("creating the secret key"),
    }
}
