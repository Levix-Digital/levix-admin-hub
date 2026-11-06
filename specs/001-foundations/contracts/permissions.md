# Permission Matrix: Foundations v0.1.0

Fixed matrix enforced in `hub-core` and published in the contract. `✓` allowed, `–` refused. The instance owner also holds every right of an organization owner in every organization.

| Action | Instance owner | Org owner | Administrator | Approver | Auditor | Read-only |
|---|---|---|---|---|---|---|
| Change instance settings | ✓ | – | – | – | – | – |
| Create an organization | ✓ | – | – | – | – | – |
| Rename or archive an organization | ✓ | ✓ | – | – | – | – |
| Read an organization | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Invite an administrator, change roles | ✓ | ✓ | – | – | – | – |
| Suspend an administrator, reset TOTP, revoke sessions | ✓ | ✓ | – | – | – | – |
| List administrators and roles | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Read the administrative audit | ✓ | ✓ | – | – | ✓ | – |
| Verify the audit chain | ✓ | ✓ | – | – | ✓ | – |
| Manage own profile, password, TOTP, sessions, API tokens | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

The administrator and approver roles have no exclusive action yet: they receive the device, policy and approval actions in v0.2.0 to v0.4.0. They exist now so role assignments do not change shape later.
