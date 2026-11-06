//! The use cases, as methods of `Hub`, which holds the ports. Every surface builds one `Hub`
//! and calls the same methods (surface parity, ADR-0001).

use std::sync::Arc;

use crate::audit::{AuditEvent, AuditResult};
use crate::context::Context;
use crate::ports::{Clock, Crypto, Store};
use crate::{CoreError, CoreResult};

pub struct Hub<S, K, C> {
    pub store: S,
    pub crypto: Arc<K>,
    pub clock: Arc<C>,
}

impl<S: Clone, K, C> Clone for Hub<S, K, C> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            crypto: Arc::clone(&self.crypto),
            clock: Arc::clone(&self.clock),
        }
    }
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    pub fn new(store: S, crypto: Arc<K>, clock: Arc<C>) -> Self {
        Self {
            store,
            crypto,
            clock,
        }
    }

    pub(crate) fn event(&self, context: &Context, action: &str) -> AuditEvent {
        AuditEvent::new(context, self.clock.now(), action, AuditResult::Success)
    }

    /// Records a refused or failed attempt and returns the error unchanged.
    pub(crate) async fn record_refusal(
        &self,
        context: &Context,
        action: &str,
        organization: Option<uuid::Uuid>,
        error: CoreError,
    ) -> CoreError {
        let result = match error {
            CoreError::Forbidden { .. }
            | CoreError::Unauthenticated
            | CoreError::Throttled { .. } => AuditResult::Denied,
            _ => AuditResult::Failed,
        };
        let mut event = AuditEvent::new(context, self.clock.now(), action, result)
            .detail("reason", error_code(&error));
        event.organization_id = organization;
        if let Err(audit_error) = self.store.append(&event).await {
            return audit_error;
        }
        error
    }

    /// Runs `check`; on refusal, records it.
    pub(crate) async fn guard<T>(
        &self,
        context: &Context,
        action: &str,
        organization: Option<uuid::Uuid>,
        check: CoreResult<T>,
    ) -> CoreResult<T> {
        match check {
            Ok(value) => Ok(value),
            Err(error @ CoreError::Forbidden { .. }) => Err(self
                .record_refusal(context, action, organization, error)
                .await),
            Err(error) => Err(error),
        }
    }
}

/// Short code of an error for audit details.
pub fn error_code(error: &CoreError) -> &'static str {
    match error {
        CoreError::Invalid { .. } => "invalid",
        CoreError::NotFound(_) => "not_found",
        CoreError::Conflict(_) => "conflict",
        CoreError::NotInitialized => "not_initialized",
        CoreError::AlreadyInitialized => "already_initialized",
        CoreError::Unauthenticated => "unauthenticated",
        CoreError::Forbidden { .. } => "forbidden",
        CoreError::Throttled { .. } => "throttled",
        CoreError::Storage(_) => "internal",
    }
}
