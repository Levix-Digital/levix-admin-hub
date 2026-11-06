//! Integration tests of the SQLite adapter itself: migrations, schema refusal, constraints.

mod common;

use common::{ok, Fixture};
use hub_core::context::Context;
use hub_core::credential::LinkCompletion;
use hub_core::organization::{NewOrganization, OrganizationStatus};
use hub_core::ports::{AuditRepository, InstanceRepository};
use hub_core::session::SessionKind;
use hub_core::CoreError;
use hub_storage_sqlite::{SqliteStore, StorageError};

async fn raw(path: &std::path::Path) -> sqlx::SqliteConnection {
    use sqlx::Connection;
    ok(sqlx::SqliteConnection::connect(&format!("sqlite://{}", path.display())).await)
}

#[tokio::test]
async fn migrates_an_empty_database_and_initializes_once() {
    let fixture = Fixture::new().await;
    assert!(!ok(fixture.hub.instance_status().await).initialized);
    ok(fixture
        .hub
        .initialize(&Context::host(), "owner@example.com")
        .await);
    assert!(ok(fixture.hub.instance_status().await).initialized);
    assert!(matches!(
        fixture
            .hub
            .initialize(&Context::host(), "other@example.com")
            .await,
        Err(CoreError::AlreadyInitialized)
    ));
}

#[tokio::test]
async fn organizations_round_trip_and_slugs_are_unique() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let created = fixture.organization(&owner, "acme").await;
    let context = fixture.refresh(&owner).await;
    assert_eq!(
        ok(fixture.hub.organization(&context, created.id).await),
        created
    );
    assert!(matches!(
        fixture
            .hub
            .create_organization(
                &context,
                NewOrganization {
                    slug: "acme".into(),
                    name: "Other".into(),
                },
            )
            .await,
        Err(CoreError::Conflict(_))
    ));
    ok(fixture.hub.archive_organization(&context, created.id).await);
    let list = ok(fixture.hub.organizations(&context).await);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].status, OrganizationStatus::Archived);
}

#[tokio::test]
async fn reopening_keeps_data() {
    let dir = ok(tempfile::tempdir());
    let path = dir.path().join("hub.db");
    {
        let store = ok(SqliteStore::open(&path).await);
        assert!(ok(store.get_instance().await).is_none());
    }
    let store = ok(SqliteStore::open(&path).await);
    ok(store.check_writable().await);
    assert_eq!(SqliteStore::schema_version(), 2);
}

#[tokio::test]
async fn refuses_a_database_from_a_newer_hub() {
    let dir = ok(tempfile::tempdir());
    let path = dir.path().join("hub.db");
    {
        let _store = ok(SqliteStore::open(&path).await);
    }
    ok(sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (9999, 'future', 1, x'00', 0)",
    )
    .execute(&mut raw(&path).await)
    .await);
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

#[tokio::test]
async fn the_database_refuses_to_change_or_delete_audit_records() {
    let fixture = Fixture::new().await;
    fixture.owner().await;
    let path = fixture._dir.path().join("hub.db");
    let mut connection = raw(&path).await;
    for statement in [
        "UPDATE audit_record SET action = 'forged'",
        "DELETE FROM audit_record",
    ] {
        let error = sqlx::query(statement).execute(&mut connection).await;
        assert!(error.is_err(), "{statement} was accepted");
    }
    let records = ok(fixture.hub.store.all_records().await);
    assert!(records.len() >= 2);
    assert!(hub_core::audit::verify(&records)
        .map(|r| r.intact)
        .unwrap_or(false));
}

#[tokio::test]
async fn a_link_is_used_once_even_concurrently() {
    let fixture = Fixture::new().await;
    let (_, link) = ok(fixture
        .hub
        .initialize(&Context::host(), "owner@example.com")
        .await);
    let anonymous = fixture.anonymous();
    let enrollment = ok(fixture.hub.begin_enrollment(&anonymous, &link.token).await);
    let secret = ok(data_encoding::BASE32_NOPAD.decode(enrollment.secret.as_bytes()));
    let code = fixture.code(&secret);
    let attempt = || {
        fixture.hub.complete_link(
            &anonymous,
            LinkCompletion {
                token: &link.token,
                display_name: None,
                password: common::PASSWORD,
                code: &code,
            },
            SessionKind::Cli,
            None,
        )
    };
    let (first, second) = tokio::join!(attempt(), attempt());
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(second.is_ok()),
        1,
        "exactly one completion must win"
    );
    assert!(fixture
        .hub
        .inspect_link(&anonymous, &link.token)
        .await
        .is_err());
}
