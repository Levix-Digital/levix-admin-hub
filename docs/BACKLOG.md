# Backlog

Work captured for later planning: bugs, features, improvements and adjustments. Capturing an item takes no discovery; when the user decides to work on items, they are promoted into a plan (`plan-memory/backlog.py promote`), where the discovery and the planning happen. Open bugs come before new work.

Each item: `### <ID>: <title>` with the fields `Severity` (bugs: critical, high, medium, low) or `Priority` (others: high, medium, low), `Status` (open, planned, in_progress), `Found`, `Plan` and `Description`. Closed items (done, dropped, duplicate) move to `Closed` with a `Closed` note and keep their ids, which are never reused. Edit this file with `plan-memory/backlog.py` or by hand, and check it with `backlog.py validate`.

## Bugs

## Features

### FEAT-001: [core] Rust service skeleton with organizations and the shared platform crates
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: in_progress
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Design: docs/architecture/admin-hub-design.md, Principles, and section Common core. The Admin Hub as a Rust service with organizations, sharing the policy model, signature verification and data definition formats with levix-platform, so client and server are identical.

### FEAT-002: [packaging] Container package with compact and scalable profiles
- **Priority**: high
- **Epic**: EPIC-012
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Packaging. Containers for a single server and Helm for Kubernetes; a compact profile (one container with an embedded database) and a scalable one (Postgres, Redis, replicas) from the same code. Always hosted by the customer, never a Levix SaaS.

### FEAT-003: [identity] Own directory of users and groups
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Users and groups. Users and groups managed in the Admin Hub for customers without an identity provider, per product (accounts never shared across products).

### FEAT-004: [identity] SSO with the company's identity provider and SCIM provisioning
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Users and groups. Corporate users and groups from the company's identity provider through SSO, provisioned and deprovisioned through SCIM.

### FEAT-005: [identity] Activation codes for directory users
- **Priority**: high
- **Epic**: EPIC-002
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Activation of directory users. The admin registers a user and the Admin Hub issues a single-use activation code with a short validity, delivered through a separate channel; the password and keys are created on the device and never go through the Admin Hub; a new code serves a forgotten password.

### FEAT-006: [devices] Enrollment with a pinned key, device identity, heartbeat and kill-switch
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Devices and enrollment. The enrollment code carries the Admin Hub key fingerprint for the device to pin; devices have an Ed25519 identity and keep an outbound-only WebSocket with heartbeat; the kill-switch blocks a corporate environment.

### FEAT-007: [licensing] Product licenses and seats verified offline
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, License and seats per product. A license signed by Levix per product (seats, validity), verified offline by the Admin Hub. Seats per device or per user is an open commercial decision; both are supported.

### FEAT-008: [policy] Policy schema registry, signing and distribution
- **Priority**: high
- **Epic**: EPIC-005
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Policy distribution and signing. Each product registers its policy schema; the Admin Hub edits, versions, signs and distributes policies to enrolled devices without knowing the product details, with a company-set offline validity.

### FEAT-009: [audit] Metadata-only audit
- **Priority**: high
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Audit. Audit records from devices (rule, when, device) and from the Admin Hub itself, never with content.

### FEAT-010: [itsm] sudo approval requests and ITSM tickets
- **Priority**: high
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, ITSM. Administrative requests from devices become approval requests in the Admin Hub and, optionally, ServiceNow or Jira tickets.

### FEAT-011: [identity] Break-glass root passwords per device (LAPS model)
- **Priority**: high
- **Epic**: EPIC-007
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Break-glass. A unique per-device root password generated and rotated by the Admin Hub, visible only to authorized administrators with access logging; each use is reported by the device and followed by rotation.

### FEAT-012: [identity] Organization recovery public key distribution
- **Priority**: medium
- **Epic**: EPIC-007
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Common core, Organization recovery key. When the policy turns it on, the Admin Hub distributes the organization public key; the private part stays offline under the company's control and each recovery is audited.

### FEAT-013: [sharing] Groups, curators and organization data units
- **Priority**: medium
- **Epic**: EPIC-009
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Product data definitions and sharing. Sharing with groups from SSO or the directory; organization data units (AnyContext organization workspaces) with the admin as owner and department curators as editors.

### FEAT-014: [sharing] Distribution of product data definitions
- **Priority**: medium
- **Epic**: EPIC-009
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Product data definitions and sharing. Product data definitions (AnyContext: workspace name, sources, filters, domain pack, prompt) distributed as metadata only, never content.

### FEAT-015: [telemetry] Source telemetry at the level set by the policy
- **Priority**: medium
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Product data definitions and sharing. Counts only; anonymized identifiers by default; full paths only when the company needs to audit and informs users.

### FEAT-016: [mirror] Update mirror for air-gapped fleets
- **Priority**: high
- **Epic**: EPIC-010
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Update mirror. Product updates, models, domain packs and bundled components downloaded once by IT, verified by the Levix signature, and released to the fleet when IT decides.

### FEAT-017: [devices] Enrollment and governance of product servers
- **Priority**: medium
- **Epic**: EPIC-011
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section Data plane servers. Product servers in the company VPC (AnyContext Server, Model Server) are enrolled and governed like devices.

### FEAT-018: [contract] Documented API and WebSocket protocol
- **Priority**: high
- **Epic**: EPIC-013
- **Status**: in_progress
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Design: docs/architecture/admin-hub-design.md, section Contract. A versioned, documented contract for everything that goes through the channel, so any product in any language can talk to the Admin Hub.

### FEAT-019: [admin] Administrator accounts, roles and second factor
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: in_progress
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Organization owner, administrator, approver, auditor and read-only roles per organization and product; SSO or local accounts with Argon2id and mandatory TOTP (TECH_SPEC, Security and Privacy).

### FEAT-020: [console] Web console served by the Hub
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: in_progress
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Leptos console embedded in the binary, no external assets, WCAG 2.1 AA, English, Canadian French and Brazilian Portuguese (ADR-0002).

### FEAT-021: [cli] Administration CLI
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: in_progress
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Rust CLI over the public API for every administrative action, scripts and automation.

### FEAT-022: [install] Guided installation
- **Priority**: high
- **Epic**: EPIC-001
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: First-run wizard: organization, Hub key, TLS certificate, first administrator and first enrollment code; target: IT deploys the compact profile alone in one afternoon.

### FEAT-023: [devices] Request-access enrollment queue for pre-installed devices
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Devices pre-installed by Intune, Jamf or SCCM start dormant and request access; requests wait for approval in the Hub, through ITSM or SCIM group membership.

### FEAT-024: [devices] Fleet inventory
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Status, product and version, OS, last heartbeat and enrollment history of every device, without content.

### FEAT-025: [devices] Short-lived device certificates and offline command queue
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Certificates renewed on each heartbeat with a company-set offline validity (default 72 h); commands for offline devices queued and delivered on reconnection.

### FEAT-026: [licensing] Seat enforcement with hardware replacement
- **Priority**: high
- **Epic**: EPIC-004
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Approvals blocked when seats are full; revoking an old device frees its seat; seats per user by default, per device for OT.

### FEAT-027: [products] DocMapper manifest as the first end-to-end case
- **Priority**: high
- **Epic**: EPIC-015
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: The DocMapper manifest, maintained by the DocMapper project, registered and governed end to end as the first conformance case; no DocMapper code in the Hub.

### FEAT-028: [products] AnyContext manifest end to end
- **Priority**: high
- **Epic**: EPIC-015
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: The AnyContext manifest (models and endpoints, web search and domains, MCP server, connectors and account domains, sources, quotas), maintained by the AnyContext project, registered and governed end to end; no AnyContext code in the Hub.

### FEAT-029: [policy] Quotas
- **Priority**: medium
- **Epic**: EPIC-005
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Usage quotas per device or user reported as counters in heartbeats, with the action the policy sets when reached.

### FEAT-030: [policy] Corporate model endpoint and key distribution
- **Priority**: medium
- **Epic**: EPIC-005
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Endpoints and keys set by IT delivered to devices encrypted to the device public key, never shown to users.

### FEAT-031: [itsm] Generic webhooks to approve and revoke
- **Priority**: high
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Outbound events (request created, device enrolled, revoked) and authenticated approve and revoke endpoints, HMAC-SHA256 signed, for any ITSM or script.

### FEAT-032: [itsm] ServiceNow adapter
- **Priority**: high
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Requests open ServiceNow items; decisions return through signed webhooks.

### FEAT-033: [itsm] Jira Service Management adapter
- **Priority**: high
- **Epic**: EPIC-006
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Requests open Jira Service Management issues; decisions return through signed webhooks.

### FEAT-034: [audit] SIEM export
- **Priority**: high
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Audit and security events exported as syslog (RFC 5424) or JSON over HTTPS, buffered while the SIEM is unavailable.

### FEAT-035: [audit] Governed MCP usage audit
- **Priority**: medium
- **Epic**: EPIC-008
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Metadata of MCP calls to product servers on devices (caller process, tool, volume, time) at the level the policy sets, never payloads by default.

### FEAT-036: [ops] Backup and restore
- **Priority**: high
- **Epic**: EPIC-012
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Consistent encrypted backup and restore from the CLI in both profiles; recovery time under 1 h for the compact profile.

### FEAT-037: [ops] Built-in diagnostics
- **Priority**: high
- **Epic**: EPIC-012
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: A diagnostics bundle with configuration, health and logs, without content or secrets, that IT can send to support.

### FEAT-038: [ops] Migration from the compact to the scalable profile
- **Priority**: medium
- **Epic**: EPIC-012
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Export from SQLite and import into PostgreSQL with no data loss.

### FEAT-039: [release] Signed images, SBOM and reproducible builds
- **Priority**: high
- **Epic**: EPIC-012
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Signed container images, a CycloneDX SBOM per release, reproducible build instructions and the source bundle for customers with an active agreement (ADR-0007).

### FEAT-040: [security] Threat model and security architecture document
- **Priority**: high
- **Epic**: EPIC-013
- **Status**: in_progress
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: levix_admin_hub_foundations_v010
- **Description**: Published threat model and security architecture of the Hub, kept current with each release.

### FEAT-041: [security] Vulnerability disclosure policy
- **Priority**: high
- **Epic**: EPIC-013
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Contact channel, response deadlines and a public record of fixes.

### FEAT-042: [products] Manifests of Gedo, the Dev Cycle Protocol and Aurora
- **Priority**: low
- **Epic**: EPIC-015
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: The development line products registered by their manifests, maintained by each project: gate policies and human approval audit governed by the Hub.

### FEAT-043: [products] Product registration by signed manifests
- **Priority**: high
- **Epic**: EPIC-015
- **Status**: open
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Plan**: -
- **Description**: Import and validate product manifests signed by Levix (identifier, client kinds, policy schema, data definition formats, seat model, telemetry counters, commands); the license lists the products a Hub governs; adding a product needs no Hub release. The manifest format lives in levix-platform (ADR-0005, ADR-0008).

### FEAT-044: [security] Hardware key custody (PKCS#11) for the Hub signing key and the secret key
- **Priority**: medium
- **Epic**: EPIC-013
- **Status**: open
- **Found**: 2026-11-06, threat model, v0.1.0 plan
- **Plan**: -
- **Description**: Threat model T-21 and T-33; TECH_SPEC open question. A key custody adapter for HSMs and cloud KMS through PKCS#11, so the keys never sit in a file on the data volume.

### FEAT-045: [devices] Hub signing key rotation and compromise recovery
- **Priority**: high
- **Epic**: EPIC-003
- **Status**: open
- **Found**: 2026-11-06, threat model, v0.1.0 plan
- **Plan**: -
- **Description**: Threat model T-33. Rotate the Hub signing key with a re-pinning message signed by the previous key; after a compromise, re-enroll devices through a recovery path the customer controls; audited.

### FEAT-046: [products] Levix signing key revocation for manifests and licenses
- **Priority**: high
- **Epic**: EPIC-015
- **Status**: open
- **Found**: 2026-11-06, threat model, v0.1.0 plan
- **Plan**: -
- **Description**: Threat model T-41. Offline key ceremony for the Levix keys that sign product manifests and licenses, and a signed revocation list delivered in Hub updates and through the mirror, so a compromised key stops being accepted.

## Improvements

## Adjustments

## Epics

### EPIC-001: Organizations and administration
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 1. Organizations, administrators and roles, the web console, the administration CLI and the guided installation.

### EPIC-002: Identity
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 2. Own directory, activation codes, SSO and SCIM; accounts never shared across products.

### EPIC-003: Devices and enrollment
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 3. Enrollment, request-access queue, device identity, channel, inventory, certificates and kill-switch.

### EPIC-004: Licensing and seats
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 4. Levix licenses verified offline, seats per user or device, seat enforcement.

### EPIC-005: Policies
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 5. Per-product policy schemas, signing and distribution, quotas, corporate endpoint and key distribution.

### EPIC-006: Approvals and ITSM
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 6. Approval requests, generic webhooks, ServiceNow and Jira adapters.

### EPIC-007: Break-glass and recovery
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 7. Per-device root password (LAPS model) and organization recovery public key.

### EPIC-008: Audit and telemetry
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 8. Metadata-only audit, MCP usage audit, source telemetry, SIEM export.

### EPIC-009: Sharing
- **Priority**: medium
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 9. Groups and curators, organization data units, product data definitions.

### EPIC-010: Update mirror
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 10. Updates, models and packs verified and released to the fleet by IT.

### EPIC-011: Product servers
- **Priority**: medium
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 11. Enrollment and governance of product servers in the company VPC.

### EPIC-012: Packaging and operations
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 12. Compact and scalable profiles, backup and restore, diagnostics, signed releases.

### EPIC-013: Contract and security assurance
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 13. Public versioned contract, threat model, disclosure policy, penetration test readiness.

### EPIC-015: Product registry
- **Priority**: high
- **Status**: open
- **Found**: 2026-11-06
- **Description**: Design: docs/product/VISION.md, Scope 1. Every product, current or future, Levix or third party, registered by a manifest signed by Levix; no product-specific code in the Hub (ADR-0008).

## Closed

### EPIC-014: Every Levix product under one Hub
- **Priority**: low
- **Status**: duplicate
- **Found**: 2026-11-06
- **Closed**: 2026-11-06, duplicate of EPIC-015
- **Description**: Design: docs/product/VISION.md, Scope 15. Current and future products governed through their manifests under the same Hub and contract.

### ADJ-001: [license] One license text shared by every proprietary Levix product
- **Priority**: medium
- **Status**: done
- **Found**: 2026-11-06, cross-project decision 20261106-161526
- **Plan**: -
- **Closed**: 2026-11-06, #5
- **Description**: Owner decision 2026-11-06 (cross-project thread with AnyContext): the LICENSE is byte-identical in every proprietary Levix product; section 2 allows a free edition only where Levix Digital publishes one (the Admin Hub publishes none), section 7 covers termination without an Agreement. A change is proposed by one product and applied by all.
