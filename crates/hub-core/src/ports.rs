//! Ports: what the core needs from the outside world. Adapters implement them.

use std::future::Future;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::instance::Instance;
use crate::organization::Organization;
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

/// Persistence of the instance record.
pub trait InstanceRepository: Send + Sync {
    fn get_instance(&self) -> impl Future<Output = CoreResult<Option<Instance>>> + Send;
    /// Inserts the single instance row; fails with `AlreadyInitialized` if it exists.
    fn create_instance(&self, instance: &Instance) -> impl Future<Output = CoreResult<()>> + Send;
}

/// Persistence of organizations.
pub trait OrganizationRepository: Send + Sync {
    /// Inserts a new organization; fails with `Conflict` when the slug is taken.
    fn insert_organization(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn update_organization(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = CoreResult<()>> + Send;
    fn get_organization(
        &self,
        id: Uuid,
    ) -> impl Future<Output = CoreResult<Option<Organization>>> + Send;
    /// Every organization, ordered by name.
    fn list_organizations(&self) -> impl Future<Output = CoreResult<Vec<Organization>>> + Send;
}
