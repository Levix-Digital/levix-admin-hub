use hub_core::CoreError;
use thiserror::Error;

/// Failures of the SQLite adapter, with the context an operator needs.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("cannot open the database at {path}: {source}")]
    Open {
        path: String,
        #[source]
        source: sqlx::Error,
    },
    #[error(
        "the database has schema version {found}, written by a newer Hub; this Hub supports up to {supported}. Run the newer Hub or restore a backup"
    )]
    NewerSchema { found: i64, supported: i64 },
    #[error("database migration failed: {0}")]
    Migrate(#[source] sqlx::migrate::MigrateError),
    #[error("database query failed: {0}")]
    Query(#[source] sqlx::Error),
    #[error("stored value is invalid: {0}")]
    Corrupt(String),
}

impl From<StorageError> for CoreError {
    fn from(error: StorageError) -> Self {
        CoreError::Storage(error.to_string())
    }
}

pub(crate) fn query_error(error: sqlx::Error) -> CoreError {
    StorageError::Query(error).into()
}

pub(crate) fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}
