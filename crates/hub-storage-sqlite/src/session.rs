use hub_core::audit::AuditEvent;
use hub_core::ports::{new_id, SessionRepository};
use hub_core::session::{ApiToken, Session, SessionKind};
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::audit::append_in;
use crate::convert::{optional_time_text, parse_id, parse_optional_time, parse_time, time_text};
use crate::error::{is_unique_violation, query_error};
use crate::SqliteStore;

const SESSION_COLUMNS: &str = "id, administrator_id, kind, token_hash, csrf_hash, created_at, \
                               last_seen_at, expires_at, source_address, user_agent";
const TOKEN_COLUMNS: &str = "id, administrator_id, name, display_prefix, token_hash, created_at, \
                             expires_at, last_used_at, revoked_at";

pub(crate) async fn insert_session_in(
    connection: &mut SqliteConnection,
    session: &Session,
) -> CoreResult<()> {
    sqlx::query(&format!(
        "INSERT INTO session ({SESSION_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))
    .bind(session.id.to_string())
    .bind(session.administrator_id.to_string())
    .bind(session.kind.as_str())
    .bind(&session.token_hash)
    .bind(&session.csrf_hash)
    .bind(time_text(session.created_at)?)
    .bind(time_text(session.last_seen_at)?)
    .bind(time_text(session.expires_at)?)
    .bind(&session.source_address)
    .bind(&session.user_agent)
    .execute(&mut *connection)
    .await
    .map(|_| ())
    .map_err(query_error)
}

fn to_session(row: &SqliteRow) -> CoreResult<Session> {
    Ok(Session {
        id: parse_id(row.get("id"))?,
        administrator_id: parse_id(row.get("administrator_id"))?,
        kind: SessionKind::parse(row.get("kind"))?,
        token_hash: row.get("token_hash"),
        csrf_hash: row.get("csrf_hash"),
        created_at: parse_time(row.get("created_at"))?,
        last_seen_at: parse_time(row.get("last_seen_at"))?,
        expires_at: parse_time(row.get("expires_at"))?,
        source_address: row.get("source_address"),
        user_agent: row.get("user_agent"),
    })
}

fn to_token(row: &SqliteRow) -> CoreResult<ApiToken> {
    Ok(ApiToken {
        id: parse_id(row.get("id"))?,
        administrator_id: parse_id(row.get("administrator_id"))?,
        name: row.get("name"),
        display_prefix: row.get("display_prefix"),
        token_hash: row.get("token_hash"),
        created_at: parse_time(row.get("created_at"))?,
        expires_at: parse_time(row.get("expires_at"))?,
        last_used_at: parse_optional_time(row.get("last_used_at"))?,
        revoked_at: parse_optional_time(row.get("revoked_at"))?,
    })
}

pub(crate) async fn replace_recovery_codes_in(
    connection: &mut SqliteConnection,
    administrator: Uuid,
    code_hashes: &[String],
) -> CoreResult<()> {
    let id = administrator.to_string();
    sqlx::query("DELETE FROM recovery_code WHERE administrator_id = ?")
        .bind(&id)
        .execute(&mut *connection)
        .await
        .map_err(query_error)?;
    for hash in code_hashes {
        sqlx::query("INSERT INTO recovery_code (id, administrator_id, code_hash) VALUES (?, ?, ?)")
            .bind(new_id().to_string())
            .bind(&id)
            .bind(hash)
            .execute(&mut *connection)
            .await
            .map_err(query_error)?;
    }
    Ok(())
}

impl SessionRepository for SqliteStore {
    async fn insert_session(&self, session: &Session, audit: &AuditEvent) -> CoreResult<()> {
        let mut write = self.write().await?;
        insert_session_in(&mut write.tx, session).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn find_session(&self, token_hash: &str) -> CoreResult<Option<Session>> {
        sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM session WHERE token_hash = ?"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?
        .map(|row| to_session(&row))
        .transpose()
    }

    async fn touch_session(&self, id: Uuid, at: OffsetDateTime) -> CoreResult<()> {
        let _write = self.writer.lock().await;
        sqlx::query("UPDATE session SET last_seen_at = ? WHERE id = ?")
            .bind(time_text(at)?)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(query_error)
    }

    async fn list_sessions(&self, administrator: Uuid) -> CoreResult<Vec<Session>> {
        sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM session WHERE administrator_id = ? \
             ORDER BY created_at DESC"
        ))
        .bind(administrator.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(query_error)?
        .iter()
        .map(to_session)
        .collect()
    }

    async fn revoke_session(
        &self,
        administrator: Uuid,
        id: Uuid,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let result = sqlx::query("DELETE FROM session WHERE id = ? AND administrator_id = ?")
            .bind(id.to_string())
            .bind(administrator.to_string())
            .execute(&mut *write.tx)
            .await
            .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("session"));
        }
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn revoke_all_sessions(&self, administrator: Uuid, audit: &AuditEvent) -> CoreResult<()> {
        let mut write = self.write().await?;
        sqlx::query("DELETE FROM session WHERE administrator_id = ?")
            .bind(administrator.to_string())
            .execute(&mut *write.tx)
            .await
            .map_err(query_error)?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn consume_recovery_code(
        &self,
        administrator: Uuid,
        code_hash: &str,
        at: OffsetDateTime,
    ) -> CoreResult<bool> {
        let mut write = self.write().await?;
        let result = sqlx::query(
            "UPDATE recovery_code SET used_at = ? \
             WHERE administrator_id = ? AND code_hash = ? AND used_at IS NULL",
        )
        .bind(time_text(at)?)
        .bind(administrator.to_string())
        .bind(code_hash)
        .execute(&mut *write.tx)
        .await
        .map_err(query_error)?;
        write.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn replace_recovery_codes(
        &self,
        administrator: Uuid,
        code_hashes: &[String],
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        replace_recovery_codes_in(&mut write.tx, administrator, code_hashes).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn insert_api_token(&self, token: &ApiToken, audit: &AuditEvent) -> CoreResult<()> {
        let mut write = self.write().await?;
        sqlx::query(&format!(
            "INSERT INTO api_token ({TOKEN_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(token.id.to_string())
        .bind(token.administrator_id.to_string())
        .bind(&token.name)
        .bind(&token.display_prefix)
        .bind(&token.token_hash)
        .bind(time_text(token.created_at)?)
        .bind(time_text(token.expires_at)?)
        .bind(optional_time_text(token.last_used_at)?)
        .bind(optional_time_text(token.revoked_at)?)
        .execute(&mut *write.tx)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                CoreError::Conflict("token collision".to_owned())
            } else {
                query_error(error)
            }
        })?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn find_api_token(&self, token_hash: &str) -> CoreResult<Option<ApiToken>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM api_token WHERE token_hash = ?"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?
        .map(|row| to_token(&row))
        .transpose()
    }

    async fn list_api_tokens(&self, administrator: Uuid) -> CoreResult<Vec<ApiToken>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM api_token WHERE administrator_id = ? \
             ORDER BY created_at DESC"
        ))
        .bind(administrator.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(query_error)?
        .iter()
        .map(to_token)
        .collect()
    }

    async fn revoke_api_token(
        &self,
        administrator: Uuid,
        id: Uuid,
        at: OffsetDateTime,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let result = sqlx::query(
            "UPDATE api_token SET revoked_at = ? \
             WHERE id = ? AND administrator_id = ? AND revoked_at IS NULL",
        )
        .bind(time_text(at)?)
        .bind(id.to_string())
        .bind(administrator.to_string())
        .execute(&mut *write.tx)
        .await
        .map_err(query_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("api token"));
        }
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn touch_api_token(&self, id: Uuid, at: OffsetDateTime) -> CoreResult<()> {
        let _write = self.writer.lock().await;
        sqlx::query("UPDATE api_token SET last_used_at = ? WHERE id = ?")
            .bind(time_text(at)?)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(query_error)
    }
}
