//! Integration tests of the SQLite adapter on a temporary database file.

use hub_core::instance;
use hub_core::organization::{self, NewOrganization, OrganizationStatus};
use hub_core::ports::SystemClock;
use hub_core::CoreError;
use hub_storage_sqlite::{SqliteStore, StorageError};

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

async fn open_temp() -> (tempfile::TempDir, SqliteStore) {
    let dir = ok(tempfile::tempdir());
    let store = ok(SqliteStore::open(&dir.path().join("hub.db")).await);
    (dir, store)
}

#[tokio::test]
async fn migrates_an_empty_database_and_initializes_once() {
    let (_dir, store) = open_temp().await;
    assert!(!ok(instance::status(&store).await).initialized);
    ok(instance::initialize(&store, &SystemClock).await);
    assert!(ok(instance::status(&store).await).initialized);
    assert_eq!(
        instance::initialize(&store, &SystemClock).await,
        Err(CoreError::AlreadyInitialized)
    );
}

#[tokio::test]
async fn organizations_round_trip() {
    let (_dir, store) = open_temp().await;
    let created = ok(organization::create(
        &store,
        &SystemClock,
        NewOrganization {
            slug: "acme".into(),
            name: "Acme".into(),
        },
    )
    .await);
    let fetched = ok(organization::get(&store, created.id).await);
    assert_eq!(fetched, created);

    ok(organization::archive(&store, created.id).await);
    let list = ok(organization::list(&store).await);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].status, OrganizationStatus::Archived);
}

#[tokio::test]
async fn duplicate_slug_is_a_conflict() {
    let (_dir, store) = open_temp().await;
    let input = || NewOrganization {
        slug: "acme".into(),
        name: "Acme".into(),
    };
    ok(organization::create(&store, &SystemClock, input()).await);
    assert!(matches!(
        organization::create(&store, &SystemClock, input()).await,
        Err(CoreError::Conflict(_))
    ));
}

#[tokio::test]
async fn reopening_keeps_data() {
    let dir = ok(tempfile::tempdir());
    let path = dir.path().join("hub.db");
    {
        let store = ok(SqliteStore::open(&path).await);
        ok(instance::initialize(&store, &SystemClock).await);
    }
    let store = ok(SqliteStore::open(&path).await);
    assert!(ok(instance::status(&store).await).initialized);
    ok(store.check_writable().await);
}

#[tokio::test]
async fn refuses_a_database_from_a_newer_hub() {
    let dir = ok(tempfile::tempdir());
    let path = dir.path().join("hub.db");
    {
        let _store = ok(SqliteStore::open(&path).await);
    }
    // Simulate a migration applied by a newer Hub.
    {
        use sqlx::Connection;
        let mut connection =
            ok(sqlx::SqliteConnection::connect(&format!("sqlite://{}", path.display())).await);
        ok(sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
             VALUES (9999, 'future', 1, x'00', 0)",
        )
        .execute(&mut connection)
        .await);
    }
    match SqliteStore::open(&path).await {
        Err(StorageError::NewerSchema { found, supported }) => {
            assert_eq!(found, 9999);
            assert_eq!(supported, SqliteStore::schema_version());
        }
        other => panic!("expected NewerSchema, got {other:?}"),
    }
}

#[tokio::test]
async fn refuses_an_unwritable_location() {
    let dir = ok(tempfile::tempdir());
    let missing = dir.path().join("does-not-exist").join("hub.db");
    assert!(matches!(
        SqliteStore::open(&missing).await,
        Err(StorageError::Open { .. })
    ));
}
