use hub_core::administrator::Administrator;
use hub_core::audit::AuditEvent;
use hub_core::credential::{CredentialLink, LinkPurpose};
use hub_core::ports::CredentialLinkRepository;
use hub_core::session::Session;
use hub_core::{CoreError, CoreResult};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::administrator::update_administrator_in;
use crate::audit::append_in;
use crate::convert::{
    optional_time_text, parse_id, parse_optional_id, parse_optional_time, parse_time, time_text,
};
use crate::error::query_error;
use crate::session::{insert_session_in, replace_recovery_codes_in};
use crate::SqliteStore;

pub(crate) async fn insert_link_in(
    connection: &mut SqliteConnection,
    link: &CredentialLink,
) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO credential_link (id, administrator_id, purpose, token_hash, created_by, \
         created_at, expires_at, used_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(link.id.to_string())
    .bind(link.administrator_id.to_string())
    .bind(link.purpose.as_str())
    .bind(&link.token_hash)
    .bind(link.created_by.map(|id| id.to_string()))
    .bind(time_text(link.created_at)?)
    .bind(time_text(link.expires_at)?)
    .bind(optional_time_text(link.used_at)?)
    .execute(&mut *connection)
    .await
    .map(|_| ())
    .map_err(query_error)
}

fn to_link(row: &SqliteRow) -> CoreResult<CredentialLink> {
    Ok(CredentialLink {
        id: parse_id(row.get("id"))?,
        administrator_id: parse_id(row.get("administrator_id"))?,
        purpose: LinkPurpose::parse(row.get("purpose"))?,
        token_hash: row.get("token_hash"),
        created_by: parse_optional_id(row.get("created_by"))?,
        created_at: parse_time(row.get("created_at"))?,
        expires_at: parse_time(row.get("expires_at"))?,
        used_at: parse_optional_time(row.get("used_at"))?,
    })
}

impl CredentialLinkRepository for SqliteStore {
    async fn insert_link(&self, link: &CredentialLink, audit: &AuditEvent) -> CoreResult<()> {
        let mut write = self.write().await?;
        // A new link replaces any unused link of the same administrator.
        sqlx::query(
            "UPDATE credential_link SET used_at = ? WHERE administrator_id = ? AND used_at IS NULL",
        )
        .bind(time_text(link.created_at)?)
        .bind(link.administrator_id.to_string())
        .execute(&mut *write.tx)
        .await
        .map_err(query_error)?;
        insert_link_in(&mut write.tx, link).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }

    async fn find_link(&self, token_hash: &str) -> CoreResult<Option<CredentialLink>> {
        sqlx::query(
            "SELECT id, administrator_id, purpose, token_hash, created_by, created_at, \
             expires_at, used_at FROM credential_link WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(query_error)?
        .map(|row| to_link(&row))
        .transpose()
    }

    async fn complete_link(
        &self,
        link: &CredentialLink,
        administrator: &Administrator,
        recovery_code_hashes: &[String],
        session: &Session,
        audit: &AuditEvent,
    ) -> CoreResult<()> {
        let mut write = self.write().await?;
        let used =
            sqlx::query("UPDATE credential_link SET used_at = ? WHERE id = ? AND used_at IS NULL")
                .bind(time_text(session.created_at)?)
                .bind(link.id.to_string())
                .execute(&mut *write.tx)
                .await
                .map_err(query_error)?;
        if used.rows_affected() == 0 {
            return Err(CoreError::Conflict("the link was already used".to_owned()));
        }
        update_administrator_in(&mut write.tx, administrator).await?;
        let id = administrator.id.to_string();
        replace_recovery_codes_in(&mut write.tx, administrator.id, recovery_code_hashes).await?;
        sqlx::query("DELETE FROM session WHERE administrator_id = ?")
            .bind(&id)
            .execute(&mut *write.tx)
            .await
            .map_err(query_error)?;
        insert_session_in(&mut write.tx, session).await?;
        append_in(&mut write.tx, audit).await?;
        write.commit().await
    }
}
