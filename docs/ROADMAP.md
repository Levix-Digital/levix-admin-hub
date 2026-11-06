# Roadmap

The releases of the product in delivery order, from the MVP onwards. Each release has a goal and the backlog items (`docs/BACKLOG.md`) it delivers; the roadmap is the only place that says which item goes into which release. Discovery starts from the open items of the current release (the first one not released), after the bugs that come first.

Each release: `## vX.Y.Z: <name>` with the fields `Status` (planned, in_progress, released), an optional `Target`, `Released` (the date, once released), `Goal` and `Items` (backlog ids separated by commas, `-` when empty). Edit this file with `plan-memory/roadmap.py` or by hand, and check it with `roadmap.py validate`.

## v0.1.0: Foundations
- **Status**: in_progress
- **Goal**: IT installs the Hub, creates its organization and administrators, and works through the console, the CLI and a published contract
- **Items**: FEAT-018, FEAT-001, FEAT-019, FEAT-020, FEAT-021, FEAT-040

## v0.2.0: Enrolled fleet
- **Status**: planned
- **Goal**: Devices enroll and pin the Hub, users activate their accounts, licenses and seats are enforced offline, and IT can kill-switch a device with every action audited
- **Items**: FEAT-003, FEAT-005, FEAT-006, FEAT-024, FEAT-025, FEAT-007, FEAT-026, FEAT-009, FEAT-043

## v0.3.0: Governed products
- **Status**: planned
- **Goal**: IT signs and distributes DocMapper and AnyContext policies and runs the Hub in either profile with backup, diagnostics, SIEM export and signed releases
- **Items**: FEAT-008, FEAT-027, FEAT-028, FEAT-002, FEAT-022, FEAT-036, FEAT-037, FEAT-034, FEAT-039

## v0.4.0: Enterprise identity and approvals
- **Status**: planned
- **Goal**: Users and groups come from the company's identity provider through SSO and SCIM, and access and sudo requests are approved in the Hub, through webhooks, ServiceNow or Jira
- **Items**: FEAT-004, FEAT-010, FEAT-023, FEAT-031, FEAT-032, FEAT-033

## v0.5.0: Air-gapped fleet
- **Status**: planned
- **Goal**: IT releases verified updates, models and packs to fleets without internet, and governs quotas, corporate endpoints, source telemetry and MCP usage
- **Items**: FEAT-016, FEAT-029, FEAT-030, FEAT-015, FEAT-035

## v1.0.0: Break-glass and assurance
- **Status**: planned
- **Goal**: The complete enterprise Hub: per-device break-glass passwords, the organization recovery key, migration to the scalable profile and a published disclosure policy, ready for the independent penetration test
- **Items**: FEAT-011, FEAT-012, FEAT-038, FEAT-041
