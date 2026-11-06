use hub_core::audit::AuditEvent;
use hub_core::organization::{Organization, OrganizationStatus};
use hub_core::ports::OrganizationRepository;
use hub_core::role::Role;
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use uuid::Uuid;

use crate::administrator::set_role_in;
use crate::audit::append_in;
use crate::convert::{parse_id, parse_time, time_text};
use crate::error::{is_unique_violation, query_error};
use crate::SqliteStore;

const COLUMNS: &str = "id, slug, name, status, created_at";

impl OrganizationRepository for SqliteStore {
    async fn insert_organization(
        &self,
        organization: &Organization,
        owner: Uuid,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        sqlx::query(
            "INSERT INTO organization (id, slug, name, status, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(organization.id.to_string())
        .bind(&organization.slug)
        .bind(&organization.name)
        .bind(organization.status.as_str())
        .bind(time_text(organization.created_at)?)
        .execute(&mut *write.tx)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                CoreError::Conflict(format!("slug '{}' is taken", organization.slug))
            } else {
                query_error(error)
            }
        })?;
        set_role_in(
            &mut write.tx,
            owner,
            organization.id,
            Role::Owner,
            owner,
            organization.created_at,
        )
        .await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn update_organization(
        &self,
        organization: &Organization,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let result = sqlx::query("UPDATE organization SET name = ?, status = ? WHERE id = ?")
            .bind(&organization.name)
            .bind(organization.status.as_str())
            .bind(organization.id.to_string())
            .execute(&mut *write.tx)
            .await
            .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("organization"));
        }
        append_in(&mut write.tx, audit).await?;
        write.commit().await
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
