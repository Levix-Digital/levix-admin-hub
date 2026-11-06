//! Ports: what the core needs from the outside world. Adapters implement them.
//!
//! Every write that changes state takes the audit event that records it, so the storage
//! adapter writes both in one transaction: an action is never stored without its record.

use std::collections::BTreeMap;
use std::future::Future;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::administrator::Administrator;
use crate::audit::{AuditEvent, AuditQuery, AuditRecord};
use crate::credential::CredentialLink;
use crate::instance::{Instance, InstanceSettings};
use crate::organization::Organization;
use crate::role::Role;
use crate::session::{ApiToken, Session};
use crate::CoreResult;

/// Current time, replaceable in tests.
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// New identifiers (UUID v7, ordered by creation time).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// Password hashing (Argon2id in production).
pub trait PasswordHasher: Send + Sync {
    /// A self-describing hash string (PHC format).
    fn hash(&self, password: &str) -> CoreResult<String>;
    fn verify(&self, password: &str, hash: &str) -> bool;
}

/// Authenticated encryption of small secrets under the Hub secret key.
pub trait SecretBox: Send + Sync {
    fn seal(&self, plaintext: &[u8]) -> CoreResult<Vec<u8>>;
    fn open(&self, sealed: &[u8]) -> CoreResult<Vec<u8>>;
}

/// TOTP (RFC 6238) and random tokens.
pub trait Totp: Send + Sync {
    /// A new random TOTP secret.
    fn new_secret(&self) -> Vec<u8>;
    /// The secret in base32, as authenticator apps expect it.
    fn encode_secret(&self, secret: &[u8]) -> String;
    /// The time step whose code equals `code`, within one step of `at`, if any.
    fn matching_step(&self, secret: &[u8], code: &str, at: OffsetDateTime) -> Option<i64>;
}

/// Random tokens for links, sessions, API tokens and recovery codes.
pub trait TokenSource: Send + Sync {
    /// `prefix` followed by 256 random bits, URL-safe.
    fn token(&self, prefix: &str) -> String;
    /// A recovery code that is easy to type (groups of letters and digits).
    fn recovery_code(&self) -> String;
}

/// Persistence of the instance record.
pub trait InstanceRepository: Send + Sync {
    fn get_instance(&self) -> impl Future<Output = CoreResult<Option<Instance>>> + Send;
    /// Inserts the single instance row and the instance owner with their setup link;
    /// fails with `AlreadyInitialized` if the instance exists.
    fn create_instance(
        &self,
        instance: &Instance,
        owner: &Administrator,
        setup_link: &CredentialLink,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn update_settings(
        &self,
        settings: &InstanceSettings,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
}

/// Persistence of organizations.
pub trait OrganizationRepository: Send + Sync {
    /// Inserts a new organization and makes `owner` its owner; `Conflict` when the slug is taken.
    fn insert_organization(
        &self,
        organization: &Organization,
        owner: Uuid,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn update_organization(
        &self,
        organization: &Organization,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn get_organization(
        &self,
        id: Uuid,
    ) -> impl Future<Output = CoreResult<Option<Organization>>> + Send;
    /// Every organization, ordered by name.
    fn list_organizations(&self) -> impl Future<Output = CoreResult<Vec<Organization>>> + Send;
}

/// Persistence of administrators, their roles, recovery codes and credential links.
pub trait AdministratorRepository: Send + Sync {
    fn get_administrator(
        &self,
        id: Uuid,
    ) -> impl Future<Output = CoreResult<Option<Administrator>>> + Send;
    fn find_administrator_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = CoreResult<Option<Administrator>>> + Send;
    /// Inserts an invited administrator with a role and their invitation link.
    fn insert_invited_administrator(
        &self,
        administrator: &Administrator,
        organization: Uuid,
        role: Role,
        granted_by: Uuid,
        invitation: &CredentialLink,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    /// Stores the administrator's fields. `audit` is `None` only for bookkeeping that is
    /// itself not an action (failure counters, TOTP step), which is audited separately.
    fn update_administrator(
        &self,
        administrator: &Administrator,
        audit: Option<&AuditEvent>,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    /// Adds one failed attempt in a single atomic write and returns the new count.
    fn increment_failed_attempts(&self, id: Uuid) -> impl Future<Output = CoreResult<u32>> + Send;
    /// Sets the lock end to `until` unless the account is already locked longer.
    fn extend_lock(
        &self,
        id: Uuid,
        until: OffsetDateTime,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    /// Roles of one administrator, by organization.
    fn roles_of(
        &self,
        administrator: Uuid,
    ) -> impl Future<Output = CoreResult<BTreeMap<Uuid, Role>>> + Send;
    /// Administrators holding a role in an organization, with that role.
    fn members_of(
        &self,
        organization: Uuid,
    ) -> impl Future<Output = CoreResult<Vec<(Administrator, Role)>>> + Send;
    fn set_role(
        &self,
        administrator: Uuid,
        organization: Uuid,
        role: Role,
        granted_by: Uuid,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn remove_role(
        &self,
        administrator: Uuid,
        organization: Uuid,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    /// Active owners of an organization.
    fn count_owners(&self, organization: Uuid) -> impl Future<Output = CoreResult<u64>> + Send;
    /// Active instance owners.
    fn count_instance_owners(&self) -> impl Future<Output = CoreResult<u64>> + Send;
}

/// Persistence of single-use credential links.
pub trait CredentialLinkRepository: Send + Sync {
    fn insert_link(
        &self,
        link: &CredentialLink,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn find_link(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = CoreResult<Option<CredentialLink>>> + Send;
    /// Atomically: marks the link used (fails with `Conflict` if it already was), stores the
    /// administrator, replaces their recovery codes, revokes their other sessions and
    /// opens `session`.
    fn complete_link(
        &self,
        link: &CredentialLink,
        administrator: &Administrator,
        recovery_code_hashes: &[String],
        session: &Session,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
}

/// Persistence of sessions, recovery codes and API tokens.
pub trait SessionRepository: Send + Sync {
    fn insert_session(
        &self,
        session: &Session,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn find_session(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = CoreResult<Option<Session>>> + Send;
    fn touch_session(
        &self,
        id: Uuid,
        at: OffsetDateTime,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn list_sessions(
        &self,
        administrator: Uuid,
    ) -> impl Future<Output = CoreResult<Vec<Session>>> + Send;
    /// Revokes one session of `administrator`; `NotFound` if it is not theirs.
    fn revoke_session(
        &self,
        administrator: Uuid,
        id: Uuid,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn revoke_all_sessions(
        &self,
        administrator: Uuid,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    /// Marks an unused recovery code used; `false` if none matched.
    fn consume_recovery_code(
        &self,
        administrator: Uuid,
        code_hash: &str,
        at: OffsetDateTime,
    ) -> impl Future<Output = CoreResult<bool>> + Send;
    fn replace_recovery_codes(
        &self,
        administrator: Uuid,
        code_hashes: &[String],
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn insert_api_token(
        &self,
        token: &ApiToken,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn find_api_token(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = CoreResult<Option<ApiToken>>> + Send;
    fn list_api_tokens(
        &self,
        administrator: Uuid,
    ) -> impl Future<Output = CoreResult<Vec<ApiToken>>> + Send;
    fn revoke_api_token(
        &self,
        administrator: Uuid,
        id: Uuid,
        at: OffsetDateTime,
        audit: &AuditEvent,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn touch_api_token(
        &self,
        id: Uuid,
        at: OffsetDateTime,
    ) -> impl Future<Output = CoreResult<()>> + Send;
}

/// The administrative audit.
pub trait AuditRepository: Send + Sync {
    /// Records an event that comes with no state change (denials, failed sign-ins).
    fn append(&self, event: &AuditEvent) -> impl Future<Output = CoreResult<AuditRecord>> + Send;
    /// Records matching the query, newest first.
    fn query(
        &self,
        query: &AuditQuery,
    ) -> impl Future<Output = CoreResult<Vec<AuditRecord>>> + Send;
    /// Every record in sequence order, for verification.
    fn all_records(&self) -> impl Future<Output = CoreResult<Vec<AuditRecord>>> + Send;
}

/// Everything the use cases need, bundled so surfaces pass one value.
pub trait Store:
    InstanceRepository
    + OrganizationRepository
    + AdministratorRepository
    + CredentialLinkRepository
    + SessionRepository
    + AuditRepository
    + Clone
    + 'static
{
}

impl<T> Store for T where
    T: InstanceRepository
        + OrganizationRepository
        + AdministratorRepository
        + CredentialLinkRepository
        + SessionRepository
        + AuditRepository
        + Clone
        + 'static
{
}

/// Cryptography the use cases need.
pub trait Crypto: PasswordHasher + SecretBox + Totp + TokenSource + 'static {}

impl<T> Crypto for T where T: PasswordHasher + SecretBox + Totp + TokenSource + 'static {}
