//! The Hub as the server and the host commands assemble it.

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use hub_core::ports::SystemClock;
use hub_core::Hub;
use hub_crypto::key_file::{self, KeyFileError, SecretKey};
use hub_crypto::HubCrypto;
use hub_storage_sqlite::SqliteStore;

use crate::paths;

pub type ServerHub = Hub<SqliteStore, HubCrypto, SystemClock>;

/// Opens the database and the secret key of `data_dir`, creating the key on first use (or
/// keeping a valid one left by an interrupted run).
pub async fn open(data_dir: &Path) -> anyhow::Result<(ServerHub, SqliteStore)> {
    paths::create_data_dir(data_dir)?;
    let store = SqliteStore::open(&paths::database(data_dir))
        .await
        .context("opening the database")?;
    let key = ensure_secret_key(&paths::secret_key(data_dir))?;
    let hub = Hub::new(
        store.clone(),
        Arc::new(HubCrypto::new(&key)),
        Arc::new(SystemClock),
    );
    Ok((hub, store))
}

fn ensure_secret_key(path: &Path) -> anyhow::Result<SecretKey> {
    match key_file::create(path) {
        Ok(key) => Ok(key),
        Err(KeyFileError::AlreadyExists(_)) => {
            key_file::load(path).context("loading the secret key")
        }
        Err(error) => Err(error).context("creating the secret key"),
    }
}
