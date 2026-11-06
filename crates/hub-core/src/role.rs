//! Roles and the permission matrix (contract: permissions). One matrix, enforced here for
//! every surface (threat model T-08).

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

/// Role of an administrator in one organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Role {
    Owner,
    Administrator,
    Approver,
    Auditor,
    ReadOnly,
}

impl Role {
    pub const ALL: [Role; 5] = [
        Role::Owner,
        Role::Administrator,
        Role::Approver,
        Role::Auditor,
        Role::ReadOnly,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Administrator => "administrator",
            Self::Approver => "approver",
            Self::Auditor => "auditor",
            Self::ReadOnly => "read_only",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        Self::ALL
            .into_iter()
            .find(|role| role.as_str() == value)
            .ok_or_else(|| CoreError::Invalid {
                field: "role",
                reason: format!("unknown role '{value}'"),
            })
    }
}

/// Actions on an organization governed by the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrgAction {
    /// Read the organization, its administrators and roles.
    Read,
    /// Rename or archive the organization.
    Manage,
    /// Invite administrators and change or remove their roles.
    ManageAdministrators,
    /// Read the administrative audit of the organization.
    ReadAudit,
}

impl OrgAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "organization.read",
            Self::Manage => "organization.manage",
            Self::ManageAdministrators => "administrators.manage",
            Self::ReadAudit => "audit.read",
        }
    }
}

/// Whether `role` may perform `action` in its organization.
pub fn allows(role: Role, action: OrgAction) -> bool {
    match action {
        OrgAction::Read => true,
        OrgAction::Manage | OrgAction::ManageAdministrators => role == Role::Owner,
        OrgAction::ReadAudit => matches!(role, Role::Owner | Role::Auditor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The matrix of contracts/permissions.md, row by row.
    #[test]
    fn matrix_matches_the_contract() {
        use OrgAction::*;
        use Role::*;
        let expected: &[(OrgAction, &[Role])] = &[
            (Read, &[Owner, Administrator, Approver, Auditor, ReadOnly]),
            (Manage, &[Owner]),
            (ManageAdministrators, &[Owner]),
            (ReadAudit, &[Owner, Auditor]),
        ];
        for (action, allowed) in expected {
            for role in Role::ALL {
                assert_eq!(
                    allows(role, *action),
                    allowed.contains(&role),
                    "{role:?} / {action:?}"
                );
            }
        }
    }

    #[test]
    fn roles_round_trip() {
        for role in Role::ALL {
            assert_eq!(Role::parse(role.as_str()), Ok(role));
        }
        assert!(Role::parse("root").is_err());
    }
}
