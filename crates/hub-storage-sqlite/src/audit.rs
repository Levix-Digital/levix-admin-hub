use std::collections::BTreeMap;

use hub_core::audit::{seal, AuditEvent, AuditQuery, AuditRecord, AuditResult, GENESIS_HASH};
use hub_core::ports::{new_id, AuditRepository};
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::{QueryBuilder, Row, Sqlite, SqliteConnection};

use crate::convert::{parse_id, parse_optional_id, parse_time, time_text};
use crate::error::query_error;
use crate::SqliteStore;

const COLUMNS: &str = "seq, id, at, actor_kind, actor_id, organization_id, action, target_kind, \
                       target_id, result, source_address, details, prev_hash, hash";

/// Appends `event` after the last record, inside the caller's transaction.
pub(crate) async fn append_in(
    connection: &mut SqliteConnection,
    event: &AuditEvent,
) -> CoreResult<AuditRecord> {
    let last = sqlx::query("SELECT seq, hash FROM audit_record ORDER BY seq DESC LIMIT 1")
        .fetch_optional(&mut *connection)
        .await
        .map_err(query_error)?;
    let (seq, prev_hash) = match last {
        Some(row) => (row.get::<i64, _>("seq") + 1, row.get::<String, _>("hash")),
        None => (1, GENESIS_HASH.to_owned()),
    };
    let record = seal(seq, new_id(), event.clone(), &prev_hash)?;
    let details = serde_json::to_string(&record.event.details)
        .map_err(|e| CoreError::Storage(format!("audit details: {e}")))?;
    sqlx::query(&format!(
        "INSERT INTO audit_record ({COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))
    .bind(record.seq)
    .bind(record.id.to_string())
    .bind(time_text(record.event.at)?)
    .bind(&record.event.actor_kind)
    .bind(&record.event.actor_id)
    .bind(record.event.organization_id.map(|id| id.to_string()))
    .bind(&record.event.action)
    .bind(&record.event.target_kind)
    .bind(&record.event.target_id)
    .bind(record.event.result.as_str())
    .bind(&record.event.source_address)
    .bind(details)
    .bind(&record.prev_hash)
    .bind(&record.hash)
    .execute(&mut *connection)
    .await
    .map_err(query_error)?;
    Ok(record)
}

fn to_record(row: &SqliteRow) -> CoreResult<AuditRecord> {
    let details: BTreeMap<String, String> = serde_json::from_str(row.get("details"))
        .map_err(|e| CoreError::Storage(format!("audit details: {e}")))?;
    Ok(AuditRecord {
        seq: row.get("seq"),
        id: parse_id(row.get("id"))?,
        event: AuditEvent {
            at: parse_time(row.get("at"))?,
            actor_kind: row.get("actor_kind"),
            actor_id: row.get("actor_id"),
            organization_id: parse_optional_id(row.get("organization_id"))?,
            action: row.get("action"),
            target_kind: row.get("target_kind"),
            target_id: row.get("target_id"),
            result: AuditResult::parse(row.get("result"))?,
            source_address: row.get("source_address"),
            details,
        },
        prev_hash: row.get("prev_hash"),
        hash: row.get("hash"),
    })
}

impl AuditRepository for SqliteStore {
    async fn append(&self, event: &AuditEvent) -> CoreResult<AuditRecord> {
        let mut write = self.write().await?;
        let record = append_in(&mut write.tx, event).await?;
        write.commit().await?;
        Ok(record)
    }

    async fn query(&self, query: &AuditQuery) -> CoreResult<Vec<AuditRecord>> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(format!("SELECT {COLUMNS} FROM audit_record WHERE 1 = 1"));
        if let Some(organizations) = &query.organizations {
            if organizations.is_empty() {
                return Ok(Vec::new());
            }
            builder.push(" AND organization_id IN (");
            let mut separated = builder.separated(", ");
            for id in organizations {
                separated.push_bind(id.to_string());
            }
            builder.push(")");
        }
        if let Some(before) = query.before_seq {
            builder.push(" AND seq < ").push_bind(before);
        }
        builder
            .push(" ORDER BY seq DESC LIMIT ")
            .push_bind(i64::from(query.limit));
        builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .iter()
            .map(to_record)
            .collect()
    }

    async fn all_records(&self) -> CoreResult<Vec<AuditRecord>> {
        sqlx::query(&format!("SELECT {COLUMNS} FROM audit_record ORDER BY seq"))
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .iter()
            .map(to_record)
            .collect()
    }
}
