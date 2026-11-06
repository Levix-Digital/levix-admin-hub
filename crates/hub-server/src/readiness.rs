//! Readiness: the database accepts writes and, once the Hub is initialized, the secret key
//! file is present and valid.

use std::path::PathBuf;

use hub_api::Readiness;
use hub_core::instance;
use hub_storage_sqlite::SqliteStore;

pub struct HubReadiness {
    pub store: SqliteStore,
    pub secret_key: PathBuf,
}

impl Readiness for HubReadiness {
    async fn check(&self) -> Result<(), String> {
        self.store.check_writable().await.map_err(|error| {
            tracing::warn!(%error, "database not writable");
            "database not writable".to_owned()
        })?;
        let initialized = instance::status(&self.store)
            .await
            .map_err(|error| {
                tracing::warn!(%error, "cannot read the instance");
                "database not readable".to_owned()
            })?
            .initialized;
        if initialized {
            hub_crypto::key_file::load(&self.secret_key).map_err(|error| {
                tracing::warn!(%error, "secret key unusable");
                "secret key file unusable".to_owned()
            })?;
        }
        Ok(())
    }
}
