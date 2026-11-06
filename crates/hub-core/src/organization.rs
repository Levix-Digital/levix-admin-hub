//! Organizations governed by the Hub. Every organization-scoped record carries one.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::ports::{new_id, Clock, OrganizationRepository};
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

/// Creates an active organization.
pub async fn create<R: OrganizationRepository, C: Clock>(
    repository: &R,
    clock: &C,
    input: NewOrganization,
) -> CoreResult<Organization> {
    validate_slug(&input.slug)?;
    let organization = Organization {
        id: new_id(),
        slug: input.slug,
        name: normalize_name(&input.name)?,
        status: OrganizationStatus::Active,
        created_at: clock.now(),
    };
    repository.insert_organization(&organization).await?;
    Ok(organization)
}

/// Returns one organization.
pub async fn get<R: OrganizationRepository>(repository: &R, id: Uuid) -> CoreResult<Organization> {
    repository
        .get_organization(id)
        .await?
        .ok_or(CoreError::NotFound("organization"))
}

/// Every organization, ordered by name.
pub async fn list<R: OrganizationRepository>(repository: &R) -> CoreResult<Vec<Organization>> {
    repository.list_organizations().await
}

/// Renames an active organization.
pub async fn rename<R: OrganizationRepository>(
    repository: &R,
    id: Uuid,
    name: &str,
) -> CoreResult<Organization> {
    let mut organization = get(repository, id).await?;
    ensure_active(&organization)?;
    organization.name = normalize_name(name)?;
    repository.update_organization(&organization).await?;
    Ok(organization)
}

/// Archives an organization; it becomes read-only. Archiving twice is refused.
pub async fn archive<R: OrganizationRepository>(
    repository: &R,
    id: Uuid,
) -> CoreResult<Organization> {
    let mut organization = get(repository, id).await?;
    ensure_active(&organization)?;
    organization.status = OrganizationStatus::Archived;
    repository.update_organization(&organization).await?;
    Ok(organization)
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
    use crate::testing::{FixedClock, MemoryStore};

    fn new_org(slug: &str, name: &str) -> NewOrganization {
        NewOrganization {
            slug: slug.to_owned(),
            name: name.to_owned(),
        }
    }

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

    #[tokio::test]
    async fn create_rename_archive() {
        let store = MemoryStore::default();
        let clock = FixedClock::default();
        let org = create(&store, &clock, new_org("acme", "Acme"))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(org.status, OrganizationStatus::Active);

        let renamed = rename(&store, org.id, "Acme Canada")
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(renamed.name, "Acme Canada");

        let archived = archive(&store, org.id)
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(archived.status, OrganizationStatus::Archived);

        assert!(matches!(
            rename(&store, org.id, "Other").await,
            Err(CoreError::Conflict(_))
        ));
        assert!(matches!(
            archive(&store, org.id).await,
            Err(CoreError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn duplicate_slug_conflicts() {
        let store = MemoryStore::default();
        let clock = FixedClock::default();
        assert!(create(&store, &clock, new_org("acme", "Acme"))
            .await
            .is_ok());
        assert!(matches!(
            create(&store, &clock, new_org("acme", "Acme 2")).await,
            Err(CoreError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn unknown_id_is_not_found() {
        let store = MemoryStore::default();
        assert_eq!(
            get(&store, new_id()).await,
            Err(CoreError::NotFound("organization"))
        );
    }
}
