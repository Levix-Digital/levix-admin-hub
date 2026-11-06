//! The Hub installation itself: initialization on the host and instance settings.

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::administrator::{normalize_email, Administrator, AdministratorStatus};
use crate::context::Context;
use crate::credential::{IssuedLink, LinkPurpose};
use crate::hub::Hub;
use crate::ports::{new_id, Clock, Crypto, Store};
use crate::{CoreError, CoreResult};

const INACTIVITY_MINUTES_RANGE: std::ops::RangeInclusive<u32> = 5..=60;
const ABSOLUTE_HOURS_RANGE: std::ops::RangeInclusive<u32> = 1..=24;

/// Instance-wide settings, changed by the instance owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceSettings {
    /// Sessions end after this many minutes without activity (5 to 60).
    pub session_inactivity_minutes: u32,
    /// Sessions end this many hours after sign-in (1 to 24).
    pub session_absolute_hours: u32,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            session_inactivity_minutes: 15,
            session_absolute_hours: 12,
        }
    }
}

impl InstanceSettings {
    pub fn validate(&self) -> CoreResult<()> {
        if !INACTIVITY_MINUTES_RANGE.contains(&self.session_inactivity_minutes) {
            return Err(CoreError::Invalid {
                field: "session_inactivity_minutes",
                reason: "must be between 5 and 60".to_owned(),
            });
        }
        if !ABSOLUTE_HOURS_RANGE.contains(&self.session_absolute_hours) {
            return Err(CoreError::Invalid {
                field: "session_absolute_hours",
                reason: "must be between 1 and 24".to_owned(),
            });
        }
        Ok(())
    }

    pub fn inactivity(&self) -> Duration {
        Duration::minutes(i64::from(self.session_inactivity_minutes))
    }

    pub fn absolute(&self) -> Duration {
        Duration::hours(i64::from(self.session_absolute_hours))
    }
}

/// The single instance record, created by `levix-hub init` on the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Instance {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub initialized_at: OffsetDateTime,
    pub settings: InstanceSettings,
}

/// What any caller, signed in or not, may learn about the instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InstanceStatus {
    pub initialized: bool,
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    /// Initializes the instance from the host and issues the instance owner's setup link.
    /// Runs once: a second call fails with `AlreadyInitialized` (threat model T-01).
    pub async fn initialize(
        &self,
        context: &Context,
        owner_email: &str,
    ) -> CoreResult<(Instance, IssuedLink)> {
        if self.store.get_instance().await?.is_some() {
            return Err(CoreError::AlreadyInitialized);
        }
        let now = self.clock.now();
        let instance = Instance {
            id: new_id(),
            initialized_at: now,
            settings: InstanceSettings::default(),
        };
        let owner = Administrator {
            is_instance_owner: true,
            ..Administrator::invited(normalize_email(owner_email)?, now)
        };
        let issued = self.new_link(owner.id, LinkPurpose::Setup, None, now);
        let audit = self
            .event(context, "instance.initialize")
            .target("administrator", owner.id)
            .detail("owner_email", &owner.email);
        self.store
            .create_instance(&instance, &owner, &issued.link, &audit)
            .await?;
        Ok((instance, issued))
    }

    /// Whether the instance has been initialized. Public.
    pub async fn instance_status(&self) -> CoreResult<InstanceStatus> {
        Ok(InstanceStatus {
            initialized: self.store.get_instance().await?.is_some(),
        })
    }

    /// The instance, or `NotInitialized`.
    pub async fn instance(&self) -> CoreResult<Instance> {
        self.store
            .get_instance()
            .await?
            .ok_or(CoreError::NotInitialized)
    }

    /// Changes the instance settings (instance owner only).
    pub async fn update_instance_settings(
        &self,
        context: &Context,
        settings: InstanceSettings,
    ) -> CoreResult<Instance> {
        const ACTION: &str = "instance.settings";
        let principal = context.principal()?;
        self.guard(
            context,
            ACTION,
            None,
            principal.require_instance_owner("instance.settings"),
        )
        .await?;
        settings.validate()?;
        let audit = self
            .event(context, ACTION)
            .detail(
                "session_inactivity_minutes",
                settings.session_inactivity_minutes,
            )
            .detail("session_absolute_hours", settings.session_absolute_hours);
        self.store.update_settings(&settings, &audit).await?;
        self.instance().await
    }

    /// Issues a recovery link for an instance owner who lost every factor. Host only
    /// (threat model T-15, T-23).
    pub async fn recover_owner(&self, context: &Context, email: &str) -> CoreResult<IssuedLink> {
        self.instance().await?;
        let administrator = self
            .store
            .find_administrator_by_email(&normalize_email(email)?)
            .await?
            .filter(|a| a.is_instance_owner)
            .ok_or(CoreError::NotFound("instance owner"))?;
        if administrator.status == AdministratorStatus::Suspended {
            return Err(CoreError::Conflict(
                "the instance owner is suspended".to_owned(),
            ));
        }
        let now = self.clock.now();
        let issued = self.new_link(administrator.id, LinkPurpose::Recovery, None, now);
        let audit = self
            .event(context, "instance.recover_owner")
            .target("administrator", administrator.id);
        self.store.insert_link(&issued.link, &audit).await?;
        Ok(issued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_limits() {
        assert!(InstanceSettings::default().validate().is_ok());
        let bad = InstanceSettings {
            session_inactivity_minutes: 4,
            ..InstanceSettings::default()
        };
        assert!(bad.validate().is_err());
        let bad = InstanceSettings {
            session_absolute_hours: 25,
            ..InstanceSettings::default()
        };
        assert!(bad.validate().is_err());
    }
}
