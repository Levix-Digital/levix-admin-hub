//! Administrative audit: append-only records chained by hash (security architecture, Audit;
//! threat model T-11, T-22). Records carry metadata only, never content or secrets.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::context::{Actor, Context};
use crate::hub::Hub;
use crate::ports::{Clock, Crypto, Store};
use crate::role::OrgAction;
use crate::{CoreError, CoreResult};

/// Hash that precedes the first record.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Outcome of the audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AuditResult {
    Success,
    Denied,
    Failed,
}

impl AuditResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "success" => Ok(Self::Success),
            "denied" => Ok(Self::Denied),
            "failed" => Ok(Self::Failed),
            other => Err(CoreError::Storage(format!(
                "unknown audit result '{other}'"
            ))),
        }
    }
}

/// One event to record. Field order is part of the hash: never reorder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditEvent {
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub organization_id: Option<Uuid>,
    pub action: String,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub result: AuditResult,
    pub source_address: Option<String>,
    pub details: BTreeMap<String, String>,
}

impl AuditEvent {
    /// A new event by the context's actor at `at`.
    pub fn new(context: &Context, at: OffsetDateTime, action: &str, result: AuditResult) -> Self {
        let (actor_kind, actor_id) = context.actor.describe();
        Self {
            at,
            actor_kind: actor_kind.to_owned(),
            actor_id,
            organization_id: None,
            action: action.to_owned(),
            target_kind: None,
            target_id: None,
            result,
            source_address: context.source_address.clone(),
            details: BTreeMap::new(),
        }
    }

    pub fn organization(mut self, organization_id: Uuid) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn target(mut self, kind: &str, id: impl ToString) -> Self {
        self.target_kind = Some(kind.to_owned());
        self.target_id = Some(id.to_string());
        self
    }

    pub fn detail(mut self, key: &str, value: impl ToString) -> Self {
        self.details.insert(key.to_owned(), value.to_string());
        self
    }
}

/// A stored record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditRecord {
    pub seq: i64,
    pub id: Uuid,
    #[serde(flatten)]
    pub event: AuditEvent,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Serialize)]
struct Canonical<'a> {
    seq: i64,
    id: &'a Uuid,
    event: &'a AuditEvent,
    prev_hash: &'a str,
}

/// Hash of a record: SHA-256 over the canonical JSON of its sequence, id, event and the
/// previous hash, in lowercase hex.
pub fn chain_hash(seq: i64, id: &Uuid, event: &AuditEvent, prev_hash: &str) -> CoreResult<String> {
    let canonical = serde_json::to_vec(&Canonical {
        seq,
        id,
        event,
        prev_hash,
    })
    .map_err(|e| CoreError::Storage(format!("serializing an audit record: {e}")))?;
    Ok(hex(&Sha256::digest(canonical)))
}

/// Builds the record that follows `prev_hash`.
pub fn seal(seq: i64, id: Uuid, event: AuditEvent, prev_hash: &str) -> CoreResult<AuditRecord> {
    let hash = chain_hash(seq, &id, &event, prev_hash)?;
    Ok(AuditRecord {
        seq,
        id,
        event,
        prev_hash: prev_hash.to_owned(),
        hash,
    })
}

/// Result of a chain verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChainReport {
    pub records: u64,
    pub intact: bool,
    /// Sequence of the first record whose link or hash does not match.
    pub first_broken_seq: Option<i64>,
}

/// Verifies records in sequence order.
pub fn verify<'a>(records: impl IntoIterator<Item = &'a AuditRecord>) -> CoreResult<ChainReport> {
    let mut previous = GENESIS_HASH.to_owned();
    let mut count = 0;
    for record in records {
        count += 1;
        let expected = chain_hash(record.seq, &record.id, &record.event, &previous)?;
        if record.prev_hash != previous || record.hash != expected {
            return Ok(ChainReport {
                records: count,
                intact: false,
                first_broken_seq: Some(record.seq),
            });
        }
        previous = record.hash.clone();
    }
    Ok(ChainReport {
        records: count,
        intact: true,
        first_broken_seq: None,
    })
}

/// Lowercase hex encoding.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Which records a caller asks for.
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// Only these organizations; `None` means every record (instance owner only).
    pub organizations: Option<Vec<Uuid>>,
    /// Records with a sequence strictly lower than this (pagination, newest first).
    pub before_seq: Option<i64>,
    pub limit: u32,
}

/// Largest page of audit records.
pub const AUDIT_PAGE_MAX: u32 = 500;

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    /// Audit records the caller may read, newest first. The instance owner reads every
    /// record; owners and auditors read the records of their organizations.
    pub async fn audit_records(
        &self,
        context: &Context,
        organization: Option<Uuid>,
        before_seq: Option<i64>,
        limit: u32,
    ) -> CoreResult<Vec<AuditRecord>> {
        let principal = context.principal()?;
        let organizations = match organization {
            Some(id) => {
                self.guard(
                    context,
                    "audit.read",
                    Some(id),
                    principal.require(id, OrgAction::ReadAudit),
                )
                .await?;
                Some(vec![id])
            }
            None if principal.is_instance_owner => None,
            None => Some(
                principal
                    .roles
                    .iter()
                    .filter(|(_, role)| crate::role::allows(**role, OrgAction::ReadAudit))
                    .map(|(id, _)| *id)
                    .collect(),
            ),
        };
        self.store
            .query(&AuditQuery {
                organizations,
                before_seq,
                limit: limit.clamp(1, AUDIT_PAGE_MAX),
            })
            .await
    }

    /// Verifies the whole chain. Anyone who may read some audit may run it; the report
    /// reveals no record.
    pub async fn verify_audit(&self, context: &Context) -> CoreResult<ChainReport> {
        if let Some(principal) = &context.principal {
            let may_read = principal.is_instance_owner
                || principal
                    .roles
                    .values()
                    .any(|role| crate::role::allows(*role, OrgAction::ReadAudit));
            if !may_read {
                return Err(CoreError::Forbidden {
                    permission: "audit.read",
                });
            }
        } else if context.actor != Actor::Host {
            return Err(CoreError::Unauthenticated);
        }
        verify(&self.store.all_records().await?)
    }
}

impl Actor {
    /// Kind and identifier written in audit records.
    pub fn describe(&self) -> (&'static str, Option<String>) {
        match self {
            Actor::Administrator { administrator, .. } => {
                ("administrator", Some(administrator.to_string()))
            }
            Actor::ApiToken {
                administrator,
                token,
            } => ("api_token", Some(format!("{administrator}/{token}"))),
            Actor::Host => ("host", None),
            Actor::Anonymous => ("anonymous", None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(action: &str) -> AuditEvent {
        AuditEvent::new(
            &Context::host(),
            time::macros::datetime!(2026-11-06 12:00 UTC),
            action,
            AuditResult::Success,
        )
        .detail("k", "v")
    }

    fn chain(n: i64) -> Vec<AuditRecord> {
        let mut records = Vec::new();
        let mut previous = GENESIS_HASH.to_owned();
        for seq in 1..=n {
            let record = seal(seq, Uuid::now_v7(), event(&format!("a{seq}")), &previous)
                .unwrap_or_else(|e| panic!("{e}"));
            previous = record.hash.clone();
            records.push(record);
        }
        records
    }

    #[test]
    fn intact_chain_verifies() {
        let records = chain(5);
        let report = verify(&records).unwrap_or_else(|e| panic!("{e}"));
        assert!(report.intact);
        assert_eq!(report.records, 5);
    }

    #[test]
    fn edited_record_is_detected() {
        let mut records = chain(5);
        records[2].event.action = "tampered".to_owned();
        let report = verify(&records).unwrap_or_else(|e| panic!("{e}"));
        assert!(!report.intact);
        assert_eq!(report.first_broken_seq, Some(3));
    }

    #[test]
    fn deleted_record_is_detected() {
        let mut records = chain(5);
        records.remove(1);
        let report = verify(&records).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(report.first_broken_seq, Some(3));
    }
}
