//! Organizations governed by the Hub. Every organization-scoped record carries one.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::context::Context;
use crate::hub::Hub;
use crate::ports::{new_id, Clock, Crypto, Store};
use crate::role::OrgAction;
use crate::{CoreError, CoreResult};

const SLUG_MIN_LEN: usize = 2;
const SLUG_MAX_LEN: usize = 63;
const NAME_MAX_CHARS: usize = 120;

/// Lifecycle of an organization. Archived organizations are read-only and never deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationStatus {
    Active,
    Archived,
}

impl OrganizationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            other => Err(CoreError::Storage(format!(
                "unknown organization status '{other}'"
            ))),
        }
    }
}

/// A customer organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Organization {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub status: OrganizationStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Input of `create`.
#[derive(Debug, Clone)]
pub struct NewOrganization {
    pub slug: String,
    pub name: String,
}

/// A slug is 2 to 63 lowercase ASCII letters, digits and hyphens, not starting or ending
/// with a hyphen, so it is safe in URLs and command lines.
pub fn validate_slug(slug: &str) -> CoreResult<()> {
    let invalid = |reason: &str| CoreError::Invalid {
        field: "slug",
        reason: reason.to_owned(),
    };
    if !(SLUG_MIN_LEN..=SLUG_MAX_LEN).contains(&slug.len()) {
        return Err(invalid("must have 2 to 63 characters"));
    }
    if !slug
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(invalid("only lowercase letters, digits and hyphens"));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(invalid("cannot start or end with a hyphen"));
    }
    Ok(())
}

/// A name is 1 to 120 characters after trimming, without control characters.
pub fn normalize_name(name: &str) -> CoreResult<String> {
    let trimmed = name.trim();
    let invalid = |reason: &str| CoreError::Invalid {
        field: "name",
        reason: reason.to_owned(),
    };
    if trimmed.is_empty() {
        return Err(invalid("cannot be empty"));
    }
    if trimmed.chars().count() > NAME_MAX_CHARS {
        return Err(invalid("must have at most 120 characters"));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(invalid("cannot contain control characters"));
    }
    Ok(trimmed.to_owned())
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    /// Creates an active organization (instance owner only); the creator becomes its owner.
    pub async fn create_organization(
        &self,
        context: &Context,
        input: NewOrganization,
    ) -> CoreResult<Organization> {
        const ACTION: &str = "organization.create";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            None,
            principal.require_instance_owner("organization.create"),
        )
        .await?;
        validate_slug(&input.slug)?;
        let organization = Organization {
            id: new_id(),
            slug: input.slug,
            name: normalize_name(&input.name)?,
            status: OrganizationStatus::Active,
            created_at: self.clock.now(),
        };
        let audit = self
            .event(context, ACTION)
            .organization(organization.id)
            .target("organization", organization.id)
            .detail("slug", &organization.slug);
        self.store
            .insert_organization(&organization, principal.administrator, &audit)
            .await?;
        Ok(organization)
    }

    /// One organization visible to the caller.
    pub async fn organization(&self, context: &Context, id: Uuid) -> CoreResult<Organization> {
        context.principal()?.require(id, OrgAction::Read)?;
        self.store
            .get_organization(id)
            .await?
            .ok_or(CoreError::NotFound("organization"))
    }

    /// The organizations visible to the caller, ordered by name.
    pub async fn organizations(&self, context: &Context) -> CoreResult<Vec<Organization>> {
        let principal = context.principal()?;
        let all = self.store.list_organizations().await?;
        Ok(all
            .into_iter()
            .filter(|o| principal.role_in(o.id).is_some())
            .collect())
    }

    /// Renames an active organization (owners).
    pub async fn rename_organization(
        &self,
        context: &Context,
        id: Uuid,
        name: &str,
    ) -> CoreResult<Organization> {
        const ACTION: &str = "organization.rename";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            Some(id),
            principal.require(id, OrgAction::Manage),
        )
        .await?;
        let mut organization = self.organization(context, id).await?;
        ensure_active(&organization)?;
        organization.name = normalize_name(name)?;
        let audit = self
            .event(context, ACTION)
            .organization(id)
            .target("organization", id)
            .detail("name", &organization.name);
        self.store
            .update_organization(&organization, &audit)
            .await?;
        Ok(organization)
    }

    /// Archives an organization (owners); it becomes read-only. Archiving twice is refused.
    pub async fn archive_organization(
        &self,
        context: &Context,
        id: Uuid,
    ) -> CoreResult<Organization> {
        const ACTION: &str = "organization.archive";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            Some(id),
            principal.require(id, OrgAction::Manage),
        )
        .await?;
        let mut organization = self.organization(context, id).await?;
        ensure_active(&organization)?;
        organization.status = OrganizationStatus::Archived;
        let audit = self
            .event(context, ACTION)
            .organization(id)
            .target("organization", id);
        self.store
            .update_organization(&organization, &audit)
            .await?;
        Ok(organization)
    }

    /// Fails with `Conflict` when the organization is archived.
    pub(crate) async fn ensure_organization_active(&self, id: Uuid) -> CoreResult<()> {
        let organization = self
            .store
            .get_organization(id)
            .await?
            .ok_or(CoreError::NotFound("organization"))?;
        ensure_active(&organization)
    }
}

fn ensure_active(organization: &Organization) -> CoreResult<()> {
    match organization.status {
        OrganizationStatus::Active => Ok(()),
        OrganizationStatus::Archived => Err(CoreError::Conflict(format!(
            "organization '{}' is archived and read-only",
            organization.slug
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_rules() {
        assert!(validate_slug("acme").is_ok());
        assert!(validate_slug("acme-2").is_ok());
        for bad in ["a", "Acme", "acme_co", "-acme", "acme-", "a cme", ""] {
            assert!(validate_slug(bad).is_err(), "{bad} should be refused");
        }
        assert!(validate_slug(&"a".repeat(63)).is_ok());
        assert!(validate_slug(&"a".repeat(64)).is_err());
    }

    #[test]
    fn name_rules() {
        assert_eq!(
            normalize_name("  Acme Inc. ").ok().as_deref(),
            Some("Acme Inc.")
        );
        assert!(normalize_name("   ").is_err());
        assert!(normalize_name("bad\u{0007}name").is_err());
        assert!(normalize_name(&"é".repeat(120)).is_ok());
        assert!(normalize_name(&"é".repeat(121)).is_err());
    }
}
