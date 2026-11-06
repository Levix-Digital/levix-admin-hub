# Permission matrix

Contract 1.0.0. The Hub enforces this fixed matrix in its core, for the console, the CLI and the API alike (`crates/hub-core/src/role.rs`, checked by a table test). `✓` allowed, `–` refused with `403 forbidden`. An organization the caller holds no role in answers `404 not_found`, so its existence is not revealed.

Roles are held per organization. The instance owner also holds every right of an organization owner in every organization.

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

Rules around the matrix:

- Account actions (suspend, reactivate, reset the second factor, revoke sessions) apply to another administrator only when the caller owns every organization that administrator belongs to; only the instance owner acts on an instance owner. Nobody uses them on their own account.
- The last active owner of an organization cannot be demoted or removed; the last active instance owner cannot be suspended.
- A personal API token acts with its creator's permissions at the time of each request, and cannot create tokens.
- Every refusal is written to the audit with the result `denied`.

The administrator and approver roles have no exclusive action yet: they receive the device, policy and approval actions in later contract versions. They exist now so role assignments do not change shape later.
