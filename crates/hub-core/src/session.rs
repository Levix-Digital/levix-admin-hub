//! Sign-in, sessions and personal API tokens (FR-005, FR-006, FR-007; threat model T-02,
//! T-04, T-06, T-09).

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::administrator::{Administrator, AdministratorStatus};
use crate::context::{Actor, Context, Principal};
use crate::credential::{
    normalize_recovery_code, token_hash, IssuedLink, LinkPurpose, RECOVERY_CODES,
};
use crate::hub::Hub;
use crate::instance::InstanceSettings;
use crate::ports::{new_id, Clock, Crypto, Store};
use crate::{CoreError, CoreResult};

/// Longest validity of a personal API token.
pub const API_TOKEN_MAX_DAYS: i64 = 90;
const API_TOKEN_NAME_MAX_CHARS: usize = 80;
const USER_AGENT_MAX_CHARS: usize = 200;
/// Prefix of personal API tokens, so secret scanners can find leaked ones.
pub const API_TOKEN_PREFIX: &str = "lvxh_";
const SESSION_PREFIX: &str = "lvxs_";
const CSRF_PREFIX: &str = "lvxc_";
const TOKEN_DISPLAY_CHARS: usize = 12;

/// Where a session was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    /// Browser: cookie plus CSRF token.
    Console,
    /// CLI: bearer token.
    Cli,
}

impl SessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::Cli => "cli",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "console" => Ok(Self::Console),
            "cli" => Ok(Self::Cli),
            other => Err(CoreError::Storage(format!(
                "unknown session kind '{other}'"
            ))),
        }
    }
}

/// A stored session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub administrator_id: Uuid,
    pub kind: SessionKind,
    #[serde(skip)]
    pub token_hash: String,
    #[serde(skip)]
    pub csrf_hash: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_seen_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub source_address: Option<String>,
    pub user_agent: Option<String>,
}

/// A session just opened, with its secrets (shown once).
#[derive(Debug, Clone)]
pub struct IssuedSession {
    pub session: Session,
    pub token: String,
    /// For console sessions: the CSRF token the console sends on every write.
    pub csrf_token: Option<String>,
}

/// A personal API token as stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiToken {
    pub id: Uuid,
    pub administrator_id: Uuid,
    pub name: String,
    /// The first characters, to recognize the token.
    pub display_prefix: String,
    #[serde(skip)]
    pub token_hash: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub revoked_at: Option<OffsetDateTime>,
}

/// Result of a sign-in.
#[derive(Debug)]
pub enum SignIn {
    /// Password and TOTP accepted.
    Session(IssuedSession),
    /// A recovery code was used: the administrator must set a new password and TOTP through
    /// this single-use link before anything else.
    MustReenroll(IssuedLink),
}

fn clip(value: Option<&str>, max: usize) -> Option<String> {
    value.map(|v| v.chars().filter(|c| !c.is_control()).take(max).collect())
}

/// Recovery codes look like `xxxx-xxxx-xxxx`; TOTP codes are six digits.
fn looks_like_recovery_code(code: &str) -> bool {
    normalize_recovery_code(code).len() > 6
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    pub(crate) fn new_session(
        &self,
        administrator: &Administrator,
        kind: SessionKind,
        settings: &InstanceSettings,
        context: &Context,
        user_agent: Option<&str>,
    ) -> IssuedSession {
        let now = self.clock.now();
        let token = self.crypto.token(SESSION_PREFIX);
        let csrf_token = match kind {
            SessionKind::Console => Some(self.crypto.token(CSRF_PREFIX)),
            SessionKind::Cli => None,
        };
        IssuedSession {
            session: Session {
                id: new_id(),
                administrator_id: administrator.id,
                kind,
                token_hash: token_hash(&token),
                csrf_hash: csrf_token.as_deref().map(token_hash),
                created_at: now,
                last_seen_at: now,
                expires_at: now + settings.absolute(),
                source_address: context.source_address.clone(),
                user_agent: clip(user_agent, USER_AGENT_MAX_CHARS),
            },
            token,
            csrf_token,
        }
    }

    /// Checks password and second factor, updating the failure counters. Used by sign-in and
    /// by actions that need re-authentication. Accepts a TOTP code only.
    pub(crate) async fn check_password_and_code(
        &self,
        administrator: &mut Administrator,
        password: &str,
        code: &str,
    ) -> CoreResult<()> {
        let now = self.clock.now();
        administrator.check_not_locked(now)?;
        let password_ok = administrator
            .password_hash
            .as_deref()
            .is_some_and(|hash| self.crypto.verify(password, hash));
        let step = if password_ok {
            self.totp_step(administrator, code, now)?
        } else {
            None
        };
        match step {
            Some(step) => {
                administrator.totp_last_step = Some(step);
                administrator.reset_failures();
                self.store.update_administrator(administrator, None).await
            }
            None => {
                self.register_failed_attempt(administrator, now).await?;
                Err(CoreError::Unauthenticated)
            }
        }
    }

    /// Counts a failure atomically in storage, so concurrent attempts cannot reuse one count
    /// (threat model T-02), and locks the account once the free attempts are spent.
    async fn register_failed_attempt(
        &self,
        administrator: &mut Administrator,
        now: OffsetDateTime,
    ) -> CoreResult<()> {
        let count = self
            .store
            .increment_failed_attempts(administrator.id)
            .await?;
        administrator.failed_attempts = count;
        if let Some(until) = crate::administrator::lock_until(count, now) {
            self.store.extend_lock(administrator.id, until).await?;
            administrator.locked_until = Some(until);
        }
        Ok(())
    }

    /// The matching TOTP step, refusing a step already used (replay).
    fn totp_step(
        &self,
        administrator: &Administrator,
        code: &str,
        now: OffsetDateTime,
    ) -> CoreResult<Option<i64>> {
        let Some(sealed) = administrator.totp_secret_sealed.as_deref() else {
            return Ok(None);
        };
        let secret = self.crypto.open(sealed)?;
        Ok(self
            .crypto
            .matching_step(&secret, code, now)
            .filter(|step| administrator.totp_last_step.is_none_or(|last| *step > last)))
    }

    /// Signs in with e-mail, password and a TOTP or recovery code. Wrong e-mail, password or
    /// code all fail the same way; every attempt is audited.
    pub async fn sign_in(
        &self,
        context: &Context,
        email: &str,
        password: &str,
        code: &str,
        kind: SessionKind,
        user_agent: Option<&str>,
    ) -> CoreResult<SignIn> {
        const ACTION: &str = "auth.sign_in";
        let instance = self.instance().await?;
        let email = email.trim().to_lowercase();
        let found = self
            .store
            .find_administrator_by_email(&email)
            .await?
            .filter(|a| a.status == AdministratorStatus::Active);
        let Some(mut administrator) = found else {
            // Same work as a real attempt, so timing does not reveal accounts.
            self.crypto.verify(password, DUMMY_HASH);
            let error = CoreError::Unauthenticated;
            return Err(self.record_refusal(context, ACTION, None, error).await);
        };
        let audit_context = Context {
            actor: Actor::Administrator {
                administrator: administrator.id,
                session: Uuid::nil(),
            },
            ..context.clone()
        };

        if looks_like_recovery_code(code) {
            return self
                .sign_in_with_recovery_code(&audit_context, &mut administrator, password, code)
                .await;
        }

        if let Err(error) = self
            .check_password_and_code(&mut administrator, password, code)
            .await
        {
            return Err(self
                .record_refusal(&audit_context, ACTION, None, error)
                .await);
        }
        let issued = self.new_session(
            &administrator,
            kind,
            &instance.settings,
            context,
            user_agent,
        );
        let audit_context = Context {
            actor: Actor::Administrator {
                administrator: administrator.id,
                session: issued.session.id,
            },
            ..context.clone()
        };
        let audit = self
            .event(&audit_context, ACTION)
            .target("session", issued.session.id)
            .detail("kind", kind.as_str());
        self.store.insert_session(&issued.session, &audit).await?;
        Ok(SignIn::Session(issued))
    }

    async fn sign_in_with_recovery_code(
        &self,
        context: &Context,
        administrator: &mut Administrator,
        password: &str,
        code: &str,
    ) -> CoreResult<SignIn> {
        const ACTION: &str = "auth.sign_in_recovery_code";
        let now = self.clock.now();
        if let Err(error) = administrator.check_not_locked(now) {
            return Err(self.record_refusal(context, ACTION, None, error).await);
        }
        let password_ok = administrator
            .password_hash
            .as_deref()
            .is_some_and(|hash| self.crypto.verify(password, hash));
        let code_ok = password_ok
            && self
                .store
                .consume_recovery_code(
                    administrator.id,
                    &token_hash(&normalize_recovery_code(code)),
                    now,
                )
                .await?;
        if !code_ok {
            self.register_failed_attempt(administrator, now).await?;
            let error = CoreError::Unauthenticated;
            return Err(self.record_refusal(context, ACTION, None, error).await);
        }
        administrator.reset_failures();
        self.store.update_administrator(administrator, None).await?;
        let issued = self.new_link(administrator.id, LinkPurpose::Recovery, None, now);
        let audit = self
            .event(context, ACTION)
            .target("administrator", administrator.id);
        self.store.insert_link(&issued.link, &audit).await?;
        Ok(SignIn::MustReenroll(issued))
    }

    /// Resolves a session token into a context, enforcing expiry, inactivity and the
    /// administrator's status, and records activity.
    pub async fn authenticate_session(
        &self,
        token: &str,
        source_address: Option<String>,
    ) -> CoreResult<(Context, Session)> {
        let now = self.clock.now();
        let session = self
            .store
            .find_session(&token_hash(token))
            .await?
            .ok_or(CoreError::Unauthenticated)?;
        let settings = self.instance().await?.settings;
        if session.expires_at <= now || session.last_seen_at + settings.inactivity() <= now {
            return Err(CoreError::Unauthenticated);
        }
        let administrator = self.active_administrator(session.administrator_id).await?;
        self.store.touch_session(session.id, now).await?;
        let principal = self.principal_of(&administrator).await?;
        Ok((
            Context {
                actor: Actor::Administrator {
                    administrator: administrator.id,
                    session: session.id,
                },
                principal: Some(principal),
                source_address,
            },
            session,
        ))
    }

    /// Resolves a personal API token; it acts with its creator's current permissions.
    pub async fn authenticate_api_token(
        &self,
        token: &str,
        source_address: Option<String>,
    ) -> CoreResult<Context> {
        let now = self.clock.now();
        let api_token = self
            .store
            .find_api_token(&token_hash(token))
            .await?
            .filter(|t| t.revoked_at.is_none() && t.expires_at > now)
            .ok_or(CoreError::Unauthenticated)?;
        let administrator = self
            .active_administrator(api_token.administrator_id)
            .await?;
        self.store.touch_api_token(api_token.id, now).await?;
        let principal: Principal = self.principal_of(&administrator).await?;
        Ok(Context {
            actor: Actor::ApiToken {
                administrator: administrator.id,
                token: api_token.id,
            },
            principal: Some(principal),
            source_address,
        })
    }

    async fn active_administrator(&self, id: Uuid) -> CoreResult<Administrator> {
        self.store
            .get_administrator(id)
            .await?
            .filter(|a| a.status == AdministratorStatus::Active)
            .ok_or(CoreError::Unauthenticated)
    }

    /// Ends the current session.
    pub async fn sign_out(&self, context: &Context) -> CoreResult<()> {
        let Actor::Administrator {
            administrator,
            session,
        } = context.actor
        else {
            return Err(CoreError::Unauthenticated);
        };
        let audit = self
            .event(context, "auth.sign_out")
            .target("session", session);
        self.store
            .revoke_session(administrator, session, &audit)
            .await
    }

    /// The caller's open sessions.
    pub async fn my_sessions(&self, context: &Context) -> CoreResult<Vec<Session>> {
        let principal = context.principal()?;
        self.store.list_sessions(principal.administrator).await
    }

    /// Ends one of the caller's sessions.
    pub async fn revoke_my_session(&self, context: &Context, id: Uuid) -> CoreResult<()> {
        let principal = context.principal()?;
        let audit = self.event(context, "session.revoke").target("session", id);
        self.store
            .revoke_session(principal.administrator, id, &audit)
            .await
    }

    /// Replaces the caller's recovery codes; needs a TOTP code. Returns the new codes once.
    pub async fn regenerate_recovery_codes(
        &self,
        context: &Context,
        password: &str,
        code: &str,
    ) -> CoreResult<Vec<String>> {
        const ACTION: &str = "recovery_codes.regenerate";
        let principal = context.principal()?;
        let mut administrator = self.administrator_by_id(principal.administrator).await?;
        if let Err(error) = self
            .check_password_and_code(&mut administrator, password, code)
            .await
        {
            return Err(self.record_refusal(context, ACTION, None, error).await);
        }
        let codes: Vec<String> = (0..RECOVERY_CODES)
            .map(|_| self.crypto.recovery_code())
            .collect();
        let hashes: Vec<String> = codes
            .iter()
            .map(|c| token_hash(&normalize_recovery_code(c)))
            .collect();
        let audit = self
            .event(context, ACTION)
            .target("administrator", administrator.id);
        self.store
            .replace_recovery_codes(administrator.id, &hashes, &audit)
            .await?;
        Ok(codes)
    }

    /// Creates a personal API token, valid `days` days (1 to 90). Returns the secret once.
    pub async fn create_api_token(
        &self,
        context: &Context,
        name: &str,
        days: i64,
    ) -> CoreResult<(ApiToken, String)> {
        let principal = context.principal()?;
        if matches!(context.actor, Actor::ApiToken { .. }) {
            return Err(CoreError::Forbidden {
                permission: "tokens.create_from_session",
            });
        }
        let name = name.trim();
        if name.is_empty()
            || name.chars().count() > API_TOKEN_NAME_MAX_CHARS
            || name.chars().any(char::is_control)
        {
            return Err(CoreError::Invalid {
                field: "name",
                reason: "must have 1 to 80 characters".to_owned(),
            });
        }
        if !(1..=API_TOKEN_MAX_DAYS).contains(&days) {
            return Err(CoreError::Invalid {
                field: "days",
                reason: "must be between 1 and 90".to_owned(),
            });
        }
        let now = self.clock.now();
        let secret = self.crypto.token(API_TOKEN_PREFIX);
        let token = ApiToken {
            id: new_id(),
            administrator_id: principal.administrator,
            name: name.to_owned(),
            display_prefix: secret.chars().take(TOKEN_DISPLAY_CHARS).collect(),
            token_hash: token_hash(&secret),
            created_at: now,
            expires_at: now + Duration::days(days),
            last_used_at: None,
            revoked_at: None,
        };
        let audit = self
            .event(context, "token.create")
            .target("api_token", token.id)
            .detail("expires_at", token.expires_at);
        self.store.insert_api_token(&token, &audit).await?;
        Ok((token, secret))
    }

    /// The caller's API tokens (without secrets).
    pub async fn my_api_tokens(&self, context: &Context) -> CoreResult<Vec<ApiToken>> {
        let principal = context.principal()?;
        self.store.list_api_tokens(principal.administrator).await
    }

    /// Revokes one of the caller's API tokens.
    pub async fn revoke_api_token(&self, context: &Context, id: Uuid) -> CoreResult<()> {
        let principal = context.principal()?;
        let audit = self.event(context, "token.revoke").target("api_token", id);
        self.store
            .revoke_api_token(principal.administrator, id, self.clock.now(), &audit)
            .await
    }
}

/// Checks a console write's CSRF token against its session (threat model T-05).
pub fn check_csrf(session: &Session, csrf_token: Option<&str>) -> CoreResult<()> {
    {
        match (session.kind, &session.csrf_hash, csrf_token) {
            (SessionKind::Cli, _, _) => Ok(()),
            (SessionKind::Console, Some(expected), Some(given))
                if constant_time_eq(expected.as_bytes(), token_hash(given).as_bytes()) =>
            {
                Ok(())
            }
            _ => Err(CoreError::Forbidden {
                permission: "csrf_token",
            }),
        }
    }
}

/// An Argon2id hash of a random value, verified when the e-mail is unknown.
const DUMMY_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$6sWcH6/oFc8S18Uc1ioUzQ$sBqS+0NJvkriD12wwUzRWe+VWPQ23jLQ++7puTsOi5E";

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_code_shape() {
        assert!(looks_like_recovery_code("abcd-efgh-2345"));
        assert!(!looks_like_recovery_code("123456"));
    }

    #[test]
    fn constant_time_comparison() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }
}
