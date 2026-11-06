//! The Hub installation itself: whether it has been initialized on its host.

use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::ports::{new_id, Clock, InstanceRepository};
use crate::{CoreError, CoreResult};

/// The single instance record, created by `levix-hub init` on the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Instance {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub initialized_at: OffsetDateTime,
}

/// What any caller, signed in or not, may learn about the instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InstanceStatus {
    pub initialized: bool,
}

/// Creates the instance. Runs once: a second call fails with `AlreadyInitialized`.
pub async fn initialize<R: InstanceRepository, C: Clock>(
    repository: &R,
    clock: &C,
) -> CoreResult<Instance> {
    if repository.get_instance().await?.is_some() {
        return Err(CoreError::AlreadyInitialized);
    }
    let instance = Instance {
        id: new_id(),
        initialized_at: clock.now(),
    };
    repository.create_instance(&instance).await?;
    Ok(instance)
}

/// Whether the instance has been initialized.
pub async fn status<R: InstanceRepository>(repository: &R) -> CoreResult<InstanceStatus> {
    Ok(InstanceStatus {
        initialized: repository.get_instance().await?.is_some(),
    })
}

/// Fails with `NotInitialized` until the host has run `init`.
pub async fn require_initialized<R: InstanceRepository>(repository: &R) -> CoreResult<Instance> {
    repository
        .get_instance()
        .await?
        .ok_or(CoreError::NotInitialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FixedClock, MemoryStore};

    #[tokio::test]
    async fn initializes_once() {
        let store = MemoryStore::default();
        let clock = FixedClock::default();
        assert!(
            !status(&store)
                .await
                .unwrap_or_else(|e| panic!("{e}"))
                .initialized
        );

        let instance = initialize(&store, &clock).await;
        assert!(instance.is_ok());
        assert!(
            status(&store)
                .await
                .unwrap_or_else(|e| panic!("{e}"))
                .initialized
        );

        assert_eq!(
            initialize(&store, &clock).await,
            Err(CoreError::AlreadyInitialized)
        );
    }

    #[tokio::test]
    async fn require_initialized_refuses_before_init() {
        let store = MemoryStore::default();
        assert_eq!(
            require_initialized(&store).await,
            Err(CoreError::NotInitialized)
        );
    }
}
