use hub_core::organization::{Organization, OrganizationStatus};
use hub_core::ports::OrganizationRepository;
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{is_unique_violation, query_error, StorageError};
use crate::SqliteStore;

const COLUMNS: &str = "id, slug, name, status, created_at";

impl OrganizationRepository for SqliteStore {
    async fn insert_organization(&self, organization: &Organization) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO organization (id, slug, name, status, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(organization.id.to_string())
        .bind(&organization.slug)
        .bind(&organization.name)
        .bind(organization.status.as_str())
        .bind(format_time(organization.created_at)?)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| {
            if is_unique_violation(&error) {
                CoreError::Conflict(format!("slug '{}' is taken", organization.slug))
            } else {
                query_error(error)
            }
        })
    }

    async fn update_organization(&self, organization: &Organization) -> CoreResult<()> {
        let result = sqlx::query("UPDATE organization SET name = ?, status = ? WHERE id = ?")
            .bind(&organization.name)
            .bind(organization.status.as_str())
            .bind(organization.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("organization"));
        }
        Ok(())
    }

    async fn get_organization(&self, id: Uuid) -> CoreResult<Option<Organization>> {
        sqlx::query(&format!("SELECT {COLUMNS} FROM organization WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(query_error)?
            .map(|row| to_organization(&row))
            .transpose()
    }

    async fn list_organizations(&self) -> CoreResult<Vec<Organization>> {
        sqlx::query(&format!(
            "SELECT {COLUMNS} FROM organization ORDER BY name, slug"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(query_error)?
        .iter()
        .map(to_organization)
        .collect()
    }
}

fn to_organization(row: &SqliteRow) -> CoreResult<Organization> {
    Ok(Organization {
        id: parse_id(row.get("id"))?,
        slug: row.get("slug"),
        name: row.get("name"),
        status: OrganizationStatus::parse(row.get("status"))?,
        created_at: parse_time(row.get("created_at"))?,
    })
}

pub(crate) fn parse_id(value: &str) -> CoreResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|e| StorageError::Corrupt(format!("identifier '{value}': {e}")).into())
}

pub(crate) fn parse_time(value: &str) -> CoreResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|e| StorageError::Corrupt(format!("time '{value}': {e}")).into())
}

pub(crate) fn format_time(value: OffsetDateTime) -> CoreResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|e| StorageError::Corrupt(format!("time {value}: {e}")).into())
}
