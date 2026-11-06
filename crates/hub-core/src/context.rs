//! Who is acting and from where. Every use case receives a context.

use std::collections::BTreeMap;

use uuid::Uuid;

use crate::role::{allows, OrgAction, Role};
use crate::{CoreError, CoreResult};

/// The caller of a use case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// An administrator signed in with a session.
    Administrator { administrator: Uuid, session: Uuid },
    /// An administrator's personal API token.
    ApiToken { administrator: Uuid, token: Uuid },
    /// A command run on the Hub host (`init`, `recover-owner`).
    Host,
    /// Nobody authenticated yet (sign-in, credential links).
    Anonymous,
}

/// An authenticated administrator with their current roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub administrator: Uuid,
    pub is_instance_owner: bool,
    pub roles: BTreeMap<Uuid, Role>,
}

impl Principal {
    /// Role in an organization; the instance owner acts as owner everywhere.
    pub fn role_in(&self, organization: Uuid) -> Option<Role> {
        if self.is_instance_owner {
            return Some(Role::Owner);
        }
        self.roles.get(&organization).copied()
    }

    /// Fails with `Forbidden` (or `NotFound` when the organization is invisible).
    pub fn require(&self, organization: Uuid, action: OrgAction) -> CoreResult<Role> {
        let role = self
            .role_in(organization)
            .ok_or(CoreError::NotFound("organization"))?;
        if allows(role, action) {
            Ok(role)
        } else {
            Err(CoreError::Forbidden {
                permission: action.as_str(),
            })
        }
    }

    pub fn require_instance_owner(&self, permission: &'static str) -> CoreResult<()> {
        if self.is_instance_owner {
            Ok(())
        } else {
            Err(CoreError::Forbidden { permission })
        }
    }
}

/// Actor, principal (once authenticated) and network source of a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub actor: Actor,
    pub principal: Option<Principal>,
    pub source_address: Option<String>,
}

impl Context {
    pub fn host() -> Self {
        Self {
            actor: Actor::Host,
            principal: None,
            source_address: None,
        }
    }

    pub fn anonymous(source_address: Option<String>) -> Self {
        Self {
            actor: Actor::Anonymous,
            principal: None,
            source_address,
        }
    }

    /// The authenticated principal, or `Unauthenticated`.
    pub fn principal(&self) -> CoreResult<&Principal> {
        self.principal.as_ref().ok_or(CoreError::Unauthenticated)
    }
}
