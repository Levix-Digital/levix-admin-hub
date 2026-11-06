use hub_core::instance::Instance;
use hub_core::ports::InstanceRepository;
use hub_core::{CoreError, CoreResult};
use sqlx::Row;

use crate::error::{is_unique_violation, query_error};
use crate::organization::{format_time, parse_id, parse_time};
use crate::SqliteStore;

impl InstanceRepository for SqliteStore {
    async fn get_instance(&self) -> CoreResult<Option<Instance>> {
        let row = sqlx::query("SELECT id, initialized_at FROM instance")
            .fetch_optional(&self.pool)
            .await
            .map_err(query_error)?;
        row.map(|row| {
            Ok(Instance {
                id: parse_id(row.get("id"))?,
                initialized_at: parse_time(row.get("initialized_at"))?,
            })
        })
        .transpose()
    }

    async fn create_instance(&self, instance: &Instance) -> CoreResult<()> {
        sqlx::query("INSERT INTO instance (id, initialized_at) VALUES (?, ?)")
            .bind(instance.id.to_string())
            .bind(format_time(instance.initialized_at)?)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| {
                if is_unique_violation(&error) {
                    CoreError::AlreadyInitialized
                } else {
                    query_error(error)
                }
            })
    }
}
