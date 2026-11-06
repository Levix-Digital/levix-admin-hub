//! SQLite adapter of the storage ports (compact profile, ADR-0003).
//!
//! One database file per Hub. Migrations are embedded and applied at startup; a database
//! written by a newer Hub is refused instead of being opened half-understood.

mod error;
mod instance;
mod organization;

use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use sqlx::migrate::{MigrateError, Migrator};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

pub use error::StorageError;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CONNECTIONS: u32 = 8;

/// The database of one Hub.
#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Opens (creating it if needed) the database file and applies pending migrations.
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(|source| StorageError::Open {
                path: path.display().to_string(),
                source,
            })?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(BUSY_TIMEOUT);
        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .map_err(|source| StorageError::Open {
                path: path.display().to_string(),
                source,
            })?;
        migrate(&pool).await?;
        Ok(Self { pool })
    }

    /// The schema version this build writes.
    pub fn schema_version() -> i64 {
        MIGRATOR.iter().map(|m| m.version).max().unwrap_or(0)
    }

    /// Checks that the database answers and accepts writes (readiness).
    pub async fn check_writable(&self) -> Result<(), StorageError> {
        sqlx::query("BEGIN IMMEDIATE; ROLLBACK;")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(StorageError::Query)
    }
}

async fn migrate(pool: &SqlitePool) -> Result<(), StorageError> {
    MIGRATOR.run(pool).await.map_err(|error| match error {
        MigrateError::VersionMissing(found) => StorageError::NewerSchema {
            found,
            supported: SqliteStore::schema_version(),
        },
        other => StorageError::Migrate(other),
    })
}
