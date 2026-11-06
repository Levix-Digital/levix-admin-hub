//! In-memory implementations of the ports for the core's unit tests.

use std::sync::Mutex;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::instance::Instance;
use crate::organization::Organization;
use crate::ports::{Clock, InstanceRepository, OrganizationRepository};
use crate::{CoreError, CoreResult};

pub struct FixedClock(pub OffsetDateTime);

impl Default for FixedClock {
    fn default() -> Self {
        Self(time::macros::datetime!(2026-11-06 12:00 UTC))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.0
    }
}

#[derive(Default)]
pub struct MemoryStore {
    instance: Mutex<Option<Instance>>,
    organizations: Mutex<Vec<Organization>>,
}

fn poisoned() -> CoreError {
    CoreError::Storage("memory store lock poisoned".to_owned())
}

impl InstanceRepository for MemoryStore {
    async fn get_instance(&self) -> CoreResult<Option<Instance>> {
        Ok(self.instance.lock().map_err(|_| poisoned())?.clone())
    }

    async fn create_instance(&self, instance: &Instance) -> CoreResult<()> {
        let mut slot = self.instance.lock().map_err(|_| poisoned())?;
        if slot.is_some() {
            return Err(CoreError::AlreadyInitialized);
        }
        *slot = Some(instance.clone());
        Ok(())
    }
}

impl OrganizationRepository for MemoryStore {
    async fn insert_organization(&self, organization: &Organization) -> CoreResult<()> {
        let mut all = self.organizations.lock().map_err(|_| poisoned())?;
        if all.iter().any(|o| o.slug == organization.slug) {
            return Err(CoreError::Conflict(format!(
                "slug '{}' is taken",
                organization.slug
            )));
        }
        all.push(organization.clone());
        Ok(())
    }

    async fn update_organization(&self, organization: &Organization) -> CoreResult<()> {
        let mut all = self.organizations.lock().map_err(|_| poisoned())?;
        let slot = all
            .iter_mut()
            .find(|o| o.id == organization.id)
            .ok_or(CoreError::NotFound("organization"))?;
        *slot = organization.clone();
        Ok(())
    }

    async fn get_organization(&self, id: Uuid) -> CoreResult<Option<Organization>> {
        let all = self.organizations.lock().map_err(|_| poisoned())?;
        Ok(all.iter().find(|o| o.id == id).cloned())
    }

    async fn list_organizations(&self) -> CoreResult<Vec<Organization>> {
        let mut all = self.organizations.lock().map_err(|_| poisoned())?.clone();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(all)
    }
}
