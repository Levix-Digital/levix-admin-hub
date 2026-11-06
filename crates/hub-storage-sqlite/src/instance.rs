use hub_core::administrator::Administrator;
use hub_core::audit::AuditEvent;
use hub_core::credential::CredentialLink;
use hub_core::instance::{Instance, InstanceSettings};
use hub_core::ports::InstanceRepository;
use hub_core::{CoreError, CoreResult};
use sqlx::Row;

use crate::administrator::insert_administrator_in;
use crate::audit::append_in;
use crate::convert::{parse_id, parse_time, time_text};
use crate::error::{is_unique_violation, query_error};
use crate::link::insert_link_in;
use crate::SqliteStore;

fn settings_value(value: i64) -> CoreResult<u32> {
    u32::try_from(value).map_err(|_| CoreError::Storage(format!("invalid setting {value}")))
}

impl InstanceRepository for SqliteStore {
    async fn get_instance(&self) -> CoreResult<Option<Instance>> {
        let row = sqlx::query(
            "SELECT id, initialized_at, session_inactivity_minutes, session_absolute_hours \
             FROM instance",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?;
        row.map(|row| {
            Ok(Instance {
                id: parse_id(row.get("id"))?,
                initialized_at: parse_time(row.get("initialized_at"))?,
                settings: InstanceSettings {
                    session_inactivity_minutes: settings_value(
                        row.get("session_inactivity_minutes"),
                    )?,
                    session_absolute_hours: settings_value(row.get("session_absolute_hours"))?,
                },
            })
        })
        .transpose()
    }

    async fn create_instance(
        &self,
        instance: &Instance,
        owner: &Administrator,
        setup_link: &CredentialLink,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        sqlx::query(
            "INSERT INTO instance (id, initialized_at, session_inactivity_minutes, \
             session_absolute_hours) VALUES (?, ?, ?, ?)",
        )
        .bind(instance.id.to_string())
        .bind(time_text(instance.initialized_at)?)
        .bind(i64::from(instance.settings.session_inactivity_minutes))
        .bind(i64::from(instance.settings.session_absolute_hours))
        .execute(&mut *write.tx)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                CoreError::AlreadyInitialized
            } else {
                query_error(error)
            }
        })?;
        insert_administrator_in(&mut write.tx, owner).await?;
        insert_link_in(&mut write.tx, setup_link).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn update_settings(
        &self,
        settings: &InstanceSettings,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let result = sqlx::query(
            "UPDATE instance SET session_inactivity_minutes = ?, session_absolute_hours = ?",
        )
        .bind(i64::from(settings.session_inactivity_minutes))
        .bind(i64::from(settings.session_absolute_hours))
        .execute(&mut *write.tx)
        .await
        .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotInitialized);
        }
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }
}
