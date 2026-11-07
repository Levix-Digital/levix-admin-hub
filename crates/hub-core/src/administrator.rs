//! Administrators, their roles per organization and the rules on their credentials.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::context::{Context, Principal};
use crate::credential::{IssuedLink, LinkPurpose};
use crate::hub::Hub;
use crate::ports::{new_id, Clock, Crypto, Store};
use crate::role::{OrgAction, Role};
use crate::{CoreError, CoreResult};

/// Minimum password length (FR-004).
pub const PASSWORD_MIN_CHARS: usize = 12;
const PASSWORD_MAX_CHARS: usize = 256;
const EMAIL_MAX_LEN: usize = 254;
const DISPLAY_NAME_MAX_CHARS: usize = 120;
/// Failed sign-ins tolerated before delays start.
pub const FREE_ATTEMPTS: u32 = 5;
const FIRST_DELAY_SECS: i64 = 30;
const MAX_DELAY_SECS: i64 = 15 * 60;
/// Languages of the console.
pub const LOCALES: [&str; 3] = ["en", "fr", "es"];

static COMMON_PASSWORDS: &str = include_str!("common-passwords.txt");

/// Lifecycle of an administrator account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AdministratorStatus {
    /// Created by an invitation or `init`; credentials not set yet.
    Invited,
    Active,
    Suspended,
}

impl AdministratorStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Invited => "invited",
            Self::Active => "active",
            Self::Suspended => "suspended",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "invited" => Ok(Self::Invited),
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            other => Err(CoreError::Storage(format!(
                "unknown administrator status '{other}'"
            ))),
        }
    }
}

/// A person who administers the Hub. Secrets are stored hashed or sealed, never in clear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Administrator {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub status: AdministratorStatus,
    pub password_hash: Option<String>,
    /// Confirmed TOTP secret, sealed with the Hub secret key.
    pub totp_secret_sealed: Option<Vec<u8>>,
    /// TOTP secret being enrolled through a credential link, sealed.
    pub totp_pending_sealed: Option<Vec<u8>>,
    pub totp_confirmed_at: Option<OffsetDateTime>,
    /// Last accepted TOTP time step; each step is accepted once (threat model T-06).
    pub totp_last_step: Option<i64>,
    pub locale: String,
    pub is_instance_owner: bool,
    pub failed_attempts: u32,
    pub locked_until: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl Administrator {
    /// A new invited administrator, without credentials.
    pub fn invited(email: String, now: OffsetDateTime) -> Self {
        Self {
            id: new_id(),
            display_name: email.clone(),
            email,
            status: AdministratorStatus::Invited,
            password_hash: None,
            totp_secret_sealed: None,
            totp_pending_sealed: None,
            totp_confirmed_at: None,
            totp_last_step: None,
            locale: "en".to_owned(),
            is_instance_owner: false,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
        }
    }

    /// Records a failed sign-in and returns the lock deadline, if any.
    pub fn register_failure(&mut self, now: OffsetDateTime) {
        self.failed_attempts = self.failed_attempts.saturating_add(1);
        if let Some(until) = lock_until(self.failed_attempts, now) {
            self.locked_until = Some(until);
        }
    }

    pub fn reset_failures(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
    }

    /// `Throttled` while a lock is running.
    pub fn check_not_locked(&self, now: OffsetDateTime) -> CoreResult<()> {
        match self.locked_until {
            Some(until) if until > now => Err(CoreError::Throttled {
                retry_after_secs: u64::try_from((until - now).whole_seconds().max(1)).unwrap_or(1),
            }),
            _ => Ok(()),
        }
    }
}

/// What the API shows of an administrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdministratorView {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub status: AdministratorStatus,
    pub second_factor_enrolled: bool,
    pub locale: String,
    pub is_instance_owner: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<&Administrator> for AdministratorView {
    fn from(a: &Administrator) -> Self {
        Self {
            id: a.id,
            email: a.email.clone(),
            display_name: a.display_name.clone(),
            status: a.status,
            second_factor_enrolled: a.totp_confirmed_at.is_some(),
            locale: a.locale.clone(),
            is_instance_owner: a.is_instance_owner,
            created_at: a.created_at,
        }
    }
}

/// The caller's own account and roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Me {
    #[serde(flatten)]
    pub administrator: AdministratorView,
    pub roles: BTreeMap<Uuid, Role>,
}

/// An administrator and their role in one organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Member {
    #[serde(flatten)]
    pub administrator: AdministratorView,
    pub role: Role,
}

/// Result of adding a member: an invitation link when the person was new.
#[derive(Debug)]
pub struct Added {
    pub member: Member,
    pub invitation: Option<IssuedLink>,
}

/// Trimmed, lowercased e-mail with a minimal shape check.
/// End of the lock after `failed_attempts` consecutive failures, if the account is locked:
/// none for the first free attempts, then 30 s doubling up to 15 minutes.
pub fn lock_until(failed_attempts: u32, now: OffsetDateTime) -> Option<OffsetDateTime> {
    (failed_attempts >= FREE_ATTEMPTS).then(|| {
        let doublings = (failed_attempts - FREE_ATTEMPTS).min(10);
        now + Duration::seconds((FIRST_DELAY_SECS << doublings).min(MAX_DELAY_SECS))
    })
}

pub fn normalize_email(email: &str) -> CoreResult<String> {
    let email = email.trim().to_lowercase();
    let invalid = |reason: &str| CoreError::Invalid {
        field: "email",
        reason: reason.to_owned(),
    };
    if email.is_empty() || email.len() > EMAIL_MAX_LEN {
        return Err(invalid("must have 1 to 254 characters"));
    }
    match email.split_once('@') {
        Some((local, domain))
            if !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !email.chars().any(|c| c.is_whitespace() || c.is_control()) => {}
        _ => return Err(invalid("is not an e-mail address")),
    }
    Ok(email)
}

/// Display names: 1 to 120 characters, no control characters.
pub fn normalize_display_name(name: &str) -> CoreResult<String> {
    let name = name.trim();
    let invalid = |reason: &str| CoreError::Invalid {
        field: "display_name",
        reason: reason.to_owned(),
    };
    if name.is_empty() || name.chars().count() > DISPLAY_NAME_MAX_CHARS {
        return Err(invalid("must have 1 to 120 characters"));
    }
    if name.chars().any(char::is_control) {
        return Err(invalid("cannot contain control characters"));
    }
    Ok(name.to_owned())
}

pub fn validate_locale(locale: &str) -> CoreResult<()> {
    if LOCALES.contains(&locale) {
        Ok(())
    } else {
        Err(CoreError::Invalid {
            field: "locale",
            reason: format!("must be one of {}", LOCALES.join(", ")),
        })
    }
}

/// Password rules (FR-004): 12 to 256 characters, not a common password, not one repeated
/// character, not the e-mail address.
pub fn check_password(password: &str, email: &str) -> CoreResult<()> {
    let invalid = |reason: &str| CoreError::Invalid {
        field: "password",
        reason: reason.to_owned(),
    };
    let chars = password.chars().count();
    if chars < PASSWORD_MIN_CHARS {
        return Err(invalid("must have at least 12 characters"));
    }
    if chars > PASSWORD_MAX_CHARS {
        return Err(invalid("must have at most 256 characters"));
    }
    let lowered = password.to_lowercase();
    if COMMON_PASSWORDS
        .lines()
        .any(|common| common.to_lowercase() == lowered)
    {
        return Err(invalid("is too common"));
    }
    let mut distinct: Vec<char> = password.chars().collect();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.len() < 4 {
        return Err(invalid("needs more distinct characters"));
    }
    if lowered.contains(&email.to_lowercase())
        || email
            .split('@')
            .next()
            .is_some_and(|l| l.len() >= 4 && lowered.contains(l))
    {
        return Err(invalid("cannot contain the e-mail address"));
    }
    Ok(())
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    pub(crate) async fn administrator_by_id(&self, id: Uuid) -> CoreResult<Administrator> {
        self.store
            .get_administrator(id)
            .await?
            .ok_or(CoreError::NotFound("administrator"))
    }

    /// Builds the principal of an active administrator.
    pub(crate) async fn principal_of(
        &self,
        administrator: &Administrator,
    ) -> CoreResult<Principal> {
        Ok(Principal {
            administrator: administrator.id,
            is_instance_owner: administrator.is_instance_owner,
            roles: self.store.roles_of(administrator.id).await?,
        })
    }

    /// The caller's own account and roles.
    pub async fn me(&self, context: &Context) -> CoreResult<Me> {
        let principal = context.principal()?;
        let administrator = self.administrator_by_id(principal.administrator).await?;
        Ok(Me {
            administrator: AdministratorView::from(&administrator),
            roles: principal.roles.clone(),
        })
    }

    /// Changes the caller's display name and language.
    pub async fn update_profile(
        &self,
        context: &Context,
        display_name: Option<&str>,
        locale: Option<&str>,
    ) -> CoreResult<Me> {
        let principal = context.principal()?;
        let mut administrator = self.administrator_by_id(principal.administrator).await?;
        if let Some(name) = display_name {
            administrator.display_name = normalize_display_name(name)?;
        }
        if let Some(locale) = locale {
            validate_locale(locale)?;
            administrator.locale = locale.to_owned();
        }
        let audit = self
            .event(context, "profile.update")
            .target("administrator", administrator.id);
        self.store
            .update_administrator(&administrator, Some(&audit))
            .await?;
        self.me(context).await
    }

    /// Changes the caller's password; needs the current password and a TOTP code.
    pub async fn change_password(
        &self,
        context: &Context,
        current: &str,
        new_password: &str,
        code: &str,
    ) -> CoreResult<()> {
        const ACTION: &str = "password.change";
        let principal = context.principal()?;
        let mut administrator = self.administrator_by_id(principal.administrator).await?;
        if let Err(error) = self
            .check_password_and_code(&mut administrator, current, code)
            .await
        {
            return Err(self.record_refusal(context, ACTION, None, error).await);
        }
        check_password(new_password, &administrator.email)?;
        administrator.password_hash = Some(self.crypto.hash(new_password)?);
        let audit = self
            .event(context, ACTION)
            .target("administrator", administrator.id);
        self.store
            .update_administrator(&administrator, Some(&audit))
            .await
    }

    /// Members of an organization with their roles.
    pub async fn members(&self, context: &Context, organization: Uuid) -> CoreResult<Vec<Member>> {
        context
            .principal()?
            .require(organization, OrgAction::Read)?;
        Ok(self
            .store
            .members_of(organization)
            .await?
            .iter()
            .map(|(administrator, role)| Member {
                administrator: AdministratorView::from(administrator),
                role: *role,
            })
            .collect())
    }

    /// Gives a person a role in an organization (owners). A new person is created as invited
    /// with a single-use invitation link valid 72 hours, shown once.
    pub async fn add_member(
        &self,
        context: &Context,
        organization: Uuid,
        email: &str,
        role: Role,
    ) -> CoreResult<Added> {
        const ACTION: &str = "member.add";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            Some(organization),
            principal.require(organization, OrgAction::ManageAdministrators),
        )
        .await?;
        self.ensure_organization_active(organization).await?;
        let email = normalize_email(email)?;
        let now = self.clock.now();

        if let Some(existing) = self.store.find_administrator_by_email(&email).await? {
            if self
                .store
                .roles_of(existing.id)
                .await?
                .contains_key(&organization)
            {
                return Err(CoreError::Conflict(format!(
                    "{email} already has a role in this organization"
                )));
            }
            let audit = self
                .event(context, ACTION)
                .organization(organization)
                .target("administrator", existing.id)
                .detail("role", role.as_str());
            self.store
                .set_role(
                    existing.id,
                    organization,
                    role,
                    principal.administrator,
                    &audit,
                )
                .await?;
            return Ok(Added {
                member: Member {
                    administrator: AdministratorView::from(&existing),
                    role,
                },
                invitation: None,
            });
        }

        let administrator = Administrator::invited(email, now);
        let issued = self.new_link(
            administrator.id,
            LinkPurpose::Invitation,
            Some(principal.administrator),
            now,
        );
        let audit = self
            .event(context, ACTION)
            .organization(organization)
            .target("administrator", administrator.id)
            .detail("role", role.as_str())
            .detail("invited", "true");
        self.store
            .insert_invited_administrator(
                &administrator,
                organization,
                role,
                principal.administrator,
                &issued.link,
                &audit,
            )
            .await?;
        Ok(Added {
            member: Member {
                administrator: AdministratorView::from(&administrator),
                role,
            },
            invitation: Some(issued),
        })
    }

    /// Changes a member's role (owners). The last owner cannot be demoted (T-10).
    pub async fn change_role(
        &self,
        context: &Context,
        organization: Uuid,
        administrator: Uuid,
        role: Role,
    ) -> CoreResult<Member> {
        const ACTION: &str = "member.change_role";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            Some(organization),
            principal.require(organization, OrgAction::ManageAdministrators),
        )
        .await?;
        self.ensure_organization_active(organization).await?;
        let current = self.current_role(organization, administrator).await?;
        if current == Role::Owner && role != Role::Owner {
            self.ensure_not_last_owner(organization).await?;
        }
        let audit = self
            .event(context, ACTION)
            .organization(organization)
            .target("administrator", administrator)
            .detail("from", current.as_str())
            .detail("to", role.as_str());
        self.store
            .set_role(
                administrator,
                organization,
                role,
                principal.administrator,
                &audit,
            )
            .await?;
        let target = self.administrator_by_id(administrator).await?;
        Ok(Member {
            administrator: AdministratorView::from(&target),
            role,
        })
    }

    /// Removes a member's role (owners). The last owner cannot be removed (T-10).
    pub async fn remove_member(
        &self,
        context: &Context,
        organization: Uuid,
        administrator: Uuid,
    ) -> CoreResult<()> {
        const ACTION: &str = "member.remove";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            Some(organization),
            principal.require(organization, OrgAction::ManageAdministrators),
        )
        .await?;
        self.ensure_organization_active(organization).await?;
        if self.current_role(organization, administrator).await? == Role::Owner {
            self.ensure_not_last_owner(organization).await?;
        }
        let audit = self
            .event(context, ACTION)
            .organization(organization)
            .target("administrator", administrator);
        self.store
            .remove_role(administrator, organization, &audit)
            .await
    }

    async fn current_role(&self, organization: Uuid, administrator: Uuid) -> CoreResult<Role> {
        self.store
            .roles_of(administrator)
            .await?
            .get(&organization)
            .copied()
            .ok_or(CoreError::NotFound("member"))
    }

    async fn ensure_not_last_owner(&self, organization: Uuid) -> CoreResult<()> {
        if self.store.count_owners(organization).await? <= 1 {
            return Err(CoreError::Conflict(
                "the last owner of an organization cannot be removed or demoted".to_owned(),
            ));
        }
        Ok(())
    }

    /// Actions on another administrator's whole account (suspend, reset the second factor,
    /// revoke sessions) affect every organization they belong to, so they need the instance
    /// owner, or an owner of every organization where the target has a role.
    async fn require_account_authority(
        &self,
        context: &Context,
        action: &'static str,
        target: &Administrator,
    ) -> CoreResult<()> {
        let principal = context.principal()?;
        let allowed = if principal.administrator == target.id {
            false
        } else if principal.is_instance_owner {
            true
        } else if target.is_instance_owner {
            false
        } else {
            let roles = self.store.roles_of(target.id).await?;
            !roles.is_empty()
                && roles
                    .keys()
                    .all(|org| principal.role_in(*org) == Some(Role::Owner))
        };
        let check = if allowed {
            Ok(())
        } else {
            Err(CoreError::Forbidden {
                permission: "administrators.manage",
            })
        };
        self.guard(context, action, None, check).await
    }

    /// Suspends an administrator: they cannot sign in, and their sessions and API tokens stop.
    pub async fn suspend_administrator(
        &self,
        context: &Context,
        id: Uuid,
    ) -> CoreResult<AdministratorView> {
        const ACTION: &str = "administrator.suspend";
        let mut target = self.administrator_by_id(id).await?;
        self.require_account_authority(context, ACTION, &target)
            .await?;
        if target.is_instance_owner && self.store.count_instance_owners().await? <= 1 {
            return Err(CoreError::Conflict(
                "the last instance owner cannot be suspended".to_owned(),
            ));
        }
        target.status = AdministratorStatus::Suspended;
        let audit = self.event(context, ACTION).target("administrator", id);
        self.store
            .update_administrator(&target, Some(&audit))
            .await?;
        let audit = self
            .event(context, "sessions.revoke_all")
            .target("administrator", id);
        self.store.revoke_all_sessions(id, &audit).await?;
        Ok(AdministratorView::from(&target))
    }

    /// Reactivates a suspended administrator who had completed enrollment.
    pub async fn reactivate_administrator(
        &self,
        context: &Context,
        id: Uuid,
    ) -> CoreResult<AdministratorView> {
        const ACTION: &str = "administrator.reactivate";
        let mut target = self.administrator_by_id(id).await?;
        self.require_account_authority(context, ACTION, &target)
            .await?;
        if target.status != AdministratorStatus::Suspended {
            return Err(CoreError::Conflict(
                "the administrator is not suspended".to_owned(),
            ));
        }
        target.status = if target.totp_confirmed_at.is_some() {
            AdministratorStatus::Active
        } else {
            AdministratorStatus::Invited
        };
        target.reset_failures();
        let audit = self.event(context, ACTION).target("administrator", id);
        self.store
            .update_administrator(&target, Some(&audit))
            .await?;
        Ok(AdministratorView::from(&target))
    }

    /// Resets another administrator's credentials: their sessions end and a single-use
    /// recovery link (72 hours, shown once) lets them set a new password and TOTP.
    pub async fn reset_second_factor(&self, context: &Context, id: Uuid) -> CoreResult<IssuedLink> {
        const ACTION: &str = "administrator.reset_second_factor";
        let target = self.administrator_by_id(id).await?;
        self.require_account_authority(context, ACTION, &target)
            .await?;
        if target.status == AdministratorStatus::Suspended {
            return Err(CoreError::Conflict(
                "the administrator is suspended".to_owned(),
            ));
        }
        let principal = context.principal()?;
        let issued = self.new_link(
            id,
            LinkPurpose::Recovery,
            Some(principal.administrator),
            self.clock.now(),
        );
        let audit = self.event(context, ACTION).target("administrator", id);
        self.store.insert_link(&issued.link, &audit).await?;
        let audit = self
            .event(context, "sessions.revoke_all")
            .target("administrator", id);
        self.store.revoke_all_sessions(id, &audit).await?;
        Ok(issued)
    }

    /// Ends every session of another administrator.
    pub async fn revoke_sessions_of(&self, context: &Context, id: Uuid) -> CoreResult<()> {
        const ACTION: &str = "sessions.revoke_all";
        let target = self.administrator_by_id(id).await?;
        self.require_account_authority(context, ACTION, &target)
            .await?;
        let audit = self.event(context, ACTION).target("administrator", id);
        self.store.revoke_all_sessions(id, &audit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_rules() {
        assert_eq!(
            normalize_email(" Ana@Example.COM ").ok().as_deref(),
            Some("ana@example.com")
        );
        for bad in [
            "",
            "ana",
            "ana@",
            "@example.com",
            "ana@example",
            "a na@example.com",
            "ana@.com",
        ] {
            assert!(normalize_email(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn password_rules() {
        let email = "ana.silva@example.com";
        assert!(check_password("correct horse battery", email).is_ok());
        assert!(check_password("short", email).is_err());
        assert!(check_password("Password2026!", email).is_err());
        assert!(check_password("aaaaaaaaaaaaaaa", email).is_err());
        assert!(check_password("ana.silva-is-the-best", email).is_err());
        assert!(check_password(&"x1".repeat(129), email).is_err());
    }

    #[test]
    fn lockout_grows_and_caps() {
        let now = time::macros::datetime!(2026-11-06 12:00 UTC);
        let mut admin = Administrator::invited("a@example.com".into(), now);
        for _ in 0..4 {
            admin.register_failure(now);
        }
        assert!(admin.check_not_locked(now).is_ok());
        admin.register_failure(now);
        assert!(matches!(
            admin.check_not_locked(now),
            Err(CoreError::Throttled {
                retry_after_secs: 30
            })
        ));
        admin.register_failure(now);
        assert!(matches!(
            admin.check_not_locked(now),
            Err(CoreError::Throttled {
                retry_after_secs: 60
            })
        ));
        for _ in 0..20 {
            admin.register_failure(now);
        }
        assert!(matches!(
            admin.check_not_locked(now),
            Err(CoreError::Throttled {
                retry_after_secs: 900
            })
        ));
        admin.reset_failures();
        assert!(admin.check_not_locked(now).is_ok());
    }
}
