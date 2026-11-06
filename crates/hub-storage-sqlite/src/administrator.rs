use std::collections::BTreeMap;

use hub_core::administrator::{Administrator, AdministratorStatus};
use hub_core::audit::AuditEvent;
use hub_core::credential::CredentialLink;
use hub_core::ports::AdministratorRepository;
use hub_core::role::Role;
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::audit::append_in;
use crate::convert::{optional_time_text, parse_id, parse_optional_time, parse_time, time_text};
use crate::error::{is_unique_violation, query_error};
use crate::link::insert_link_in;
use crate::SqliteStore;

const COLUMNS: &str = "a.id, a.email, a.display_name, a.status, a.password_hash, \
                       a.totp_secret_sealed, a.totp_pending_sealed, a.totp_confirmed_at, \
                       a.totp_last_step, a.locale, a.is_instance_owner, a.failed_attempts, \
                       a.locked_until, a.created_at";

pub(crate) fn to_administrator(row: &SqliteRow) -> CoreResult<Administrator> {
    let failed: i64 = row.get("failed_attempts");
    Ok(Administrator {
        id: parse_id(row.get("id"))?,
        email: row.get("email"),
        display_name: row.get("display_name"),
        status: AdministratorStatus::parse(row.get("status"))?,
        password_hash: row.get("password_hash"),
        totp_secret_sealed: row.get("totp_secret_sealed"),
        totp_pending_sealed: row.get("totp_pending_sealed"),
        totp_confirmed_at: parse_optional_time(row.get("totp_confirmed_at"))?,
        totp_last_step: row.get("totp_last_step"),
        locale: row.get("locale"),
        is_instance_owner: row.get::<i64, _>("is_instance_owner") == 1,
        failed_attempts: u32::try_from(failed).unwrap_or(u32::MAX),
        locked_until: parse_optional_time(row.get("locked_until"))?,
        created_at: parse_time(row.get("created_at"))?,
    })
}

pub(crate) async fn insert_administrator_in(
    connection: &mut SqliteConnection,
    a: &Administrator,
) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO administrator (id, email, display_name, status, password_hash, \
         totp_secret_sealed, totp_pending_sealed, totp_confirmed_at, totp_last_step, locale, \
         is_instance_owner, failed_attempts, locked_until, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(a.id.to_string())
    .bind(&a.email)
    .bind(&a.display_name)
    .bind(a.status.as_str())
    .bind(&a.password_hash)
    .bind(&a.totp_secret_sealed)
    .bind(&a.totp_pending_sealed)
    .bind(optional_time_text(a.totp_confirmed_at)?)
    .bind(a.totp_last_step)
    .bind(&a.locale)
    .bind(i64::from(a.is_instance_owner))
    .bind(i64::from(a.failed_attempts))
    .bind(optional_time_text(a.locked_until)?)
    .bind(time_text(a.created_at)?)
    .execute(&mut *connection)
    .await
    .map(|_| ())
    .map_err(|error| {
        if is_unique_violation(&error) {
            CoreError::Conflict(format!("{} is already an administrator", a.email))
        } else {
            query_error(error)
        }
    })
}

pub(crate) async fn update_administrator_in(
    connection: &mut SqliteConnection,
    a: &Administrator,
) -> CoreResult<()> {
    let result = sqlx::query(
        "UPDATE administrator SET display_name = ?, status = ?, password_hash = ?, \
         totp_secret_sealed = ?, totp_pending_sealed = ?, totp_confirmed_at = ?, \
         totp_last_step = ?, locale = ?, is_instance_owner = ?, failed_attempts = ?, \
         locked_until = ? WHERE id = ?",
    )
    .bind(&a.display_name)
    .bind(a.status.as_str())
    .bind(&a.password_hash)
    .bind(&a.totp_secret_sealed)
    .bind(&a.totp_pending_sealed)
    .bind(optional_time_text(a.totp_confirmed_at)?)
    .bind(a.totp_last_step)
    .bind(&a.locale)
    .bind(i64::from(a.is_instance_owner))
    .bind(i64::from(a.failed_attempts))
    .bind(optional_time_text(a.locked_until)?)
    .bind(a.id.to_string())
    .execute(&mut *connection)
    .await
    .map_err(query_error)?;
    if result.rows_affected() == 0 {
        return Err(CoreError::NotFound("administrator"));
    }
    Ok(())
}

pub(crate) async fn set_role_in(
    connection: &mut SqliteConnection,
    administrator: Uuid,
    organization: Uuid,
    role: Role,
    granted_by: Uuid,
    at: OffsetDateTime,
) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO role_assignment (administrator_id, organization_id, role, granted_by, \
         granted_at) VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT (administrator_id, organization_id) \
         DO UPDATE SET role = excluded.role, granted_by = excluded.granted_by, \
         granted_at = excluded.granted_at",
    )
    .bind(administrator.to_string())
    .bind(organization.to_string())
    .bind(role.as_str())
    .bind(granted_by.to_string())
    .bind(time_text(at)?)
    .execute(&mut *connection)
    .await
    .map(|_| ())
    .map_err(query_error)
}

fn parse_role(value: &str) -> CoreResult<Role> {
    Role::parse(value).map_err(|_| CoreError::Storage(format!("unknown role '{value}'")))
}

impl AdministratorRepository for SqliteStore {
    async fn get_administrator(&self, id: Uuid) -> CoreResult<Option<Administrator>> {
        sqlx::query(&format!(
            "SELECT {COLUMNS} FROM administrator a WHERE a.id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?
        .map(|row| to_administrator(&row))
        .transpose()
    }

    async fn find_administrator_by_email(&self, email: &str) -> CoreResult<Option<Administrator>> {
        sqlx::query(&format!(
            "SELECT {COLUMNS} FROM administrator a WHERE a.email = ?"
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?
        .map(|row| to_administrator(&row))
        .transpose()
    }

    async fn insert_invited_administrator(
        &self,
        administrator: &Administrator,
        organization: Uuid,
        role: Role,
        granted_by: Uuid,
        invitation: &CredentialLink,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        insert_administrator_in(&mut write.tx, administrator).await?;
        set_role_in(
            &mut write.tx,
            administrator.id,
            organization,
            role,
            granted_by,
            administrator.created_at,
        )
        .await?;
        insert_link_in(&mut write.tx, invitation).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn update_administrator(
        &self,
        administrator: &Administrator,
        audit: Option<&AuditEvent>,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        update_administrator_in(&mut write.tx, administrator).await?;
        if let Some(audit) = audit {
            append_in(&mut write.tx, audit).await?;
        }
        write.commit().await
    }

    async fn increment_failed_attempts(&self, id: Uuid) -> CoreResult<u32> {
        let mut write = self.write().await?;
        let count: Option<i64> = sqlx::query_scalar(
            "UPDATE administrator SET failed_attempts = failed_attempts + 1 WHERE id = ? \
             RETURNING failed_attempts",
        )
        .bind(id.to_string())
        .fetch_optional(&mut *write.tx)
        .await
        .map_err(query_error)?;
        write.commit().await?;
        let count = count.ok_or(CoreError::NotFound("administrator"))?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    async fn extend_lock(&self, id: Uuid, until: OffsetDateTime) -> CoreResult<()> {
        // Read and write inside one write transaction: no other writer runs in between.
        let mut write = self.write().await?;
        let current: Option<Option<String>> =
            sqlx::query_scalar("SELECT locked_until FROM administrator WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&mut *write.tx)
                .await
                .map_err(query_error)?;
        let current = current.ok_or(CoreError::NotFound("administrator"))?;
        if parse_optional_time(current)?.is_some_and(|locked| locked >= until) {
            return Ok(());
        }
        sqlx::query("UPDATE administrator SET locked_until = ? WHERE id = ?")
            .bind(time_text(until)?)
            .bind(id.to_string())
            .execute(&mut *write.tx)
            .await
            .map_err(query_error)?;
        write.commit().await
    }

    async fn roles_of(&self, administrator: Uuid) -> CoreResult<BTreeMap<Uuid, Role>> {
        sqlx::query("SELECT organization_id, role FROM role_assignment WHERE administrator_id = ?")
            .bind(administrator.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .iter()
            .map(|row| {
                Ok((
                    parse_id(row.get("organization_id"))?,
                    parse_role(row.get("role"))?,
                ))
            })
            .collect()
    }

    async fn members_of(&self, organization: Uuid) -> CoreResult<Vec<(Administrator, Role)>> {
        sqlx::query(&format!(
            "SELECT {COLUMNS}, r.role FROM administrator a \
             JOIN role_assignment r ON r.administrator_id = a.id \
             WHERE r.organization_id = ? ORDER BY a.email"
        ))
        .bind(organization.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(query_error)?
        .iter()
        .map(|row| Ok((to_administrator(row)?, parse_role(row.get("role"))?)))
        .collect()
    }

    async fn set_role(
        &self,
        administrator: Uuid,
        organization: Uuid,
        role: Role,
        granted_by: Uuid,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        set_role_in(
            &mut write.tx,
            administrator,
            organization,
            role,
            granted_by,
            audit.at,
        )
        .await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn remove_role(
        &self,
        administrator: Uuid,
        organization: Uuid,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let result = sqlx::query(
            "DELETE FROM role_assignment WHERE administrator_id = ? AND organization_id = ?",
        )
        .bind(administrator.to_string())
        .bind(organization.to_string())
        .execute(&mut *write.tx)
        .await
        .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("member"));
        }
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn count_owners(&self, organization: Uuid) -> CoreResult<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM role_assignment r JOIN administrator a \
             ON a.id = r.administrator_id \
             WHERE r.organization_id = ? AND r.role = 'owner' AND a.status = 'active'",
        )
        .bind(organization.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(query_error)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    async fn count_instance_owners(&self) -> CoreResult<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM administrator WHERE is_instance_owner = 1 AND status = 'active'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(query_error)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }
}
