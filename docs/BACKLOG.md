# Backlog

Work captured for later planning: bugs, features, improvements and adjustments. Capturing an item takes no discovery; when the user decides to work on items, they are promoted into a plan (`plan-memory/backlog.py promote`), where the discovery and the planning happen. Open bugs come before new work.

Each item: `### <ID>: <title>` with the fields `Severity` (bugs: critical, high, medium, low) or `Priority` (others: high, medium, low), `Status` (open, planned, in_progress), `Found`, `Plan` and `Description`. Closed items (done, dropped, duplicate) move to `Closed` with a `Closed` note and keep their ids, which are never reused. Edit this file with `plan-memory/backlog.py` or by hand, and check it with `backlog.py validate`.

## Bugs

## Features

### FEAT-001: [core] Rust service skeleton with organizations and the shared platform crates
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, Principles, and section 1. The Admin Hub as a Rust service with organizations, sharing the policy model, signature verification and data definition formats with levix-platform, so client and server are identical.

### FEAT-002: [packaging] Container package with compact and scalable profiles
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 5. Containers for a single server and Helm for Kubernetes; a compact profile (one container with an embedded database) and a scalable one (Postgres, Redis, replicas) from the same code. Always hosted by the customer, never a Levix SaaS.

### FEAT-003: [identity] Own directory of users and groups
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Users and groups. Users and groups managed in the Admin Hub for customers without an identity provider, per product (accounts never shared across products).

### FEAT-004: [identity] SSO with the company's identity provider and SCIM provisioning
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Users and groups. Corporate users and groups from the company's identity provider through SSO, provisioned and deprovisioned through SCIM.

### FEAT-005: [identity] Activation codes for directory users
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Activation of directory users. The admin registers a user and the Admin Hub issues a single-use activation code with a short validity, delivered through a separate channel; the password and keys are created on the device and never go through the Admin Hub; a new code serves a forgotten password.

### FEAT-006: [devices] Enrollment with a pinned key, device identity, heartbeat and kill-switch
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Devices and enrollment. The enrollment code carries the Admin Hub key fingerprint for the device to pin; devices have an Ed25519 identity and keep an outbound-only WebSocket with heartbeat; the kill-switch blocks a corporate environment.

### FEAT-007: [licensing] Product licenses and seats verified offline
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, License and seats per product. A license signed by Levix per product (seats, validity), verified offline by the Admin Hub. Seats per device or per user is an open commercial decision; both are supported.

### FEAT-008: [policy] Policy schema registry, signing and distribution
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Policy distribution and signing. Each product registers its policy schema; the Admin Hub edits, versions, signs and distributes policies to enrolled devices without knowing the product details, with a company-set offline validity.

### FEAT-009: [audit] Metadata-only audit
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Audit. Audit records from devices (rule, when, device) and from the Admin Hub itself, never with content.

### FEAT-010: [itsm] sudo approval requests and ITSM tickets
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, ITSM. Administrative requests from devices become approval requests in the Admin Hub and, optionally, ServiceNow or Jira tickets.

### FEAT-011: [identity] Break-glass root passwords per device (LAPS model)
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Break-glass. A unique per-device root password generated and rotated by the Admin Hub, visible only to authorized administrators with access logging; each use is reported by the device and followed by rotation.

### FEAT-012: [identity] Organization recovery public key distribution
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 1, Organization recovery key. When the policy turns it on, the Admin Hub distributes the organization public key; the private part stays offline under the company's control and each recovery is audited.

### FEAT-013: [sharing] Groups, curators and organization data units
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 2. Sharing with groups from SSO or the directory; organization data units (AnyContext organization workspaces) with the admin as owner and department curators as editors.

### FEAT-014: [sharing] Distribution of product data definitions
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 2. Product data definitions (AnyContext: workspace name, sources, filters, domain pack, prompt) distributed as metadata only, never content.

### FEAT-015: [telemetry] Source telemetry at the level set by the policy
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 2. Counts only; anonymized identifiers by default; full paths only when the company needs to audit and informs users.

### FEAT-016: [mirror] Update mirror for air-gapped fleets
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 3. Product updates, models, domain packs and bundled components downloaded once by IT, verified by the Levix signature, and released to the fleet when IT decides.

### FEAT-017: [devices] Enrollment and governance of product servers
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 4. Product servers in the company VPC (AnyContext Server, Model Server) are enrolled and governed like devices.

### FEAT-018: [contract] Documented API and WebSocket protocol
- **Priority**: medium
- **Status**: open
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Plan**: -
- **Description**: Design: docs/architecture/admin-hub-design.md, section 6. A versioned, documented contract for everything that goes through the channel, so any product in any language can talk to the Admin Hub.

## Improvements

## Adjustments

## Closed
