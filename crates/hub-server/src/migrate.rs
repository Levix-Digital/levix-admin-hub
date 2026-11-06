//! `levix-hub migrate`: applies pending migrations without serving.

use std::path::Path;

use anyhow::Context;
use hub_storage_sqlite::SqliteStore;

pub async fn run(data_dir: &Path) -> anyhow::Result<()> {
    let path = crate::paths::database(data_dir);
    SqliteStore::open(&path)
        .await
        .with_context(|| format!("migrating {}", path.display()))?;
    println!(
        "Database {} is at schema version {}.",
        path.display(),
        SqliteStore::schema_version()
    );
    Ok(())
}
