//! SQLite adapter of the storage ports (compact profile, ADR-0003).
//!
//! One database file per Hub. Migrations are embedded and applied at startup; a database
//! written by a newer Hub is refused instead of being opened half-understood. Writes are
//! serialized in the process and every state change is committed together with its audit
//! record.

mod administrator;
mod audit;
mod convert;
mod error;
mod instance;
mod link;
mod organization;
mod session;

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use hub_core::CoreResult;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::{Mutex, MutexGuard};

pub use error::StorageError;

use crate::error::query_error;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CONNECTIONS: u32 = 8;

/// The database of one Hub.
#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
    writer: Arc<Mutex<()>>,
}

/// A write transaction holding the process-wide writer lock.
pub(crate) struct Write<'a> {
    pub(crate) tx: Transaction<'static, Sqlite>,
    _guard: MutexGuard<'a, ()>,
}

impl Write<'_> {
    pub(crate) async fn commit(self) -> CoreResult<()> {
        self.tx.commit().await.map_err(query_error)
    }
}

impl SqliteStore {
    /// Opens (creating it if needed) the database file and applies pending migrations.
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        let open_error = |source| StorageError::Open {
            path: path.display().to_string(),
            source,
        };
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(open_error)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(BUSY_TIMEOUT);
        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .map_err(open_error)?;
        migrate(&pool).await?;
        Ok(Self {
            pool,
            writer: Arc::new(Mutex::new(())),
        })
    }

    /// The schema version this build writes.
    pub fn schema_version() -> i64 {
        MIGRATOR.iter().map(|m| m.version).max().unwrap_or(0)
    }

    /// Checks that the database answers and accepts writes (readiness).
    pub async fn check_writable(&self) -> Result<(), StorageError> {
        let _guard = self.writer.lock().await;
        sqlx::query("BEGIN IMMEDIATE; ROLLBACK;")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(StorageError::Query)
    }

    pub(crate) async fn write(&self) -> CoreResult<Write<'_>> {
        let guard = self.writer.lock().await;
        // IMMEDIATE takes the write lock now, so a host command running beside the server
        // waits (busy timeout) instead of failing when it upgrades a read.
        let tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(query_error)?;
        Ok(Write { tx, _guard: guard })
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
