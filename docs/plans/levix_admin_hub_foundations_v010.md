---
plan_id: levix_admin_hub_foundations_v010
project: levix-admin-hub
title: Foundations v0.1.0
kind: feature
version: v0.1.0
status: active
author: LeviGuilherme
created_at: '2026-11-06T16:20:08'
updated_at: '2026-11-06T16:24:16'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-018
- FEAT-001
- FEAT-019
- FEAT-020
- FEAT-021
- FEAT-040
spec_dir: specs/001-foundations
milestones:
- name: M1 Threat model and security architecture
  done: false
- name: M2 Service skeleton and organizations
  done: false
- name: M3 Administrators roles and TOTP
  done: false
- name: M4 Public contract
  done: false
- name: M5 Administration CLI
  done: false
- name: M6 Web console
  done: false
- name: M7 Release readiness
  done: false
status_log:
- at: '2026-11-06T16:20:08'
  by: LeviGuilherme
  status: draft
  note: Plan created
- at: '2026-11-06T16:24:09'
  by: LeviGuilherme
  status: planned
  note: Plan updated
- at: '2026-11-06T16:24:16'
  by: LeviGuilherme
  status: active
  note: Plan updated
---

---
plan_id: levix_admin_hub_foundations_v010
project: levix-admin-hub
title: Foundations v0.1.0
kind: feature
version: v0.1.0
status: draft
author: LeviGuilherme
created_at: '2026-11-06T16:20:08'
updated_at: '2026-11-06T16:20:08'
session_id: null
tags: []
supersedes: null
superseded_by: null
parent_plan_id: null
backlog_items:
- FEAT-018
- FEAT-001
- FEAT-019
- FEAT-020
- FEAT-021
- FEAT-040
milestones: []
status_log:
- at: '2026-11-06T16:20:08'
  by: LeviGuilherme
  status: draft
  note: Plan created
---

# Foundations v0.1.0

Envelope of the Spec Kit feature [`specs/001-foundations`](../../specs/001-foundations/): the detail (specification, research, data model, technical plan with the protocol sections, tasks) lives there and is not copied here.

## Backlog items

### FEAT-018: [contract] Documented API and WebSocket protocol
- **Priority**: high
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Description**: Design: docs/architecture/admin-hub-design.md, section Contract. A versioned, documented contract for everything that goes through the channel, so any product in any language can talk to the Admin Hub.

### FEAT-001: [core] Rust service skeleton with organizations and the shared platform crates
- **Priority**: high
- **Found**: 2026-10-23, initial backlog, from the AnyContext v2 rebuild
- **Description**: Design: docs/architecture/admin-hub-design.md, Principles, and section Common core. The Admin Hub as a Rust service with organizations, sharing the policy model, signature verification and data definition formats with levix-platform, so client and server are identical.

### FEAT-019: [admin] Administrator accounts, roles and second factor
- **Priority**: high
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Description**: Organization owner, administrator, approver, auditor and read-only roles per organization and product; SSO or local accounts with Argon2id and mandatory TOTP (TECH_SPEC, Security and Privacy).

### FEAT-020: [console] Web console served by the Hub
- **Priority**: high
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Description**: Leptos console embedded in the binary, no external assets, WCAG 2.1 AA, English, Canadian French and Brazilian Portuguese (ADR-0002).

### FEAT-021: [cli] Administration CLI
- **Priority**: high
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Description**: Rust CLI over the public API for every administrative action, scripts and automation.

### FEAT-040: [security] Threat model and security architecture document
- **Priority**: high
- **Found**: 2026-11-06, Phase 0, 2026-11-06
- **Description**: Published threat model and security architecture of the Hub, kept current with each release.

## 1. Goal and decisions

Deliver release v0.1.0 Foundations (docs/ROADMAP.md): IT installs the Hub, claims it from the host, creates its organizations and administrators with roles and a mandatory second factor, and works through the web console, the CLI and a published contract, with a threat model from the first release.

Decisions (detail and rejected alternatives in [research.md](../../specs/001-foundations/research.md)):

- One plan for the release; one pull request into `main` per milestone (R1).
- No default credentials: the Hub is claimed with a single-use setup link created by `levix-hub init` on the host (R2).
- Several organizations per Hub, with an instance owner (R3).
- Local administrator accounts with Argon2id and mandatory TOTP; SSO in v0.4.0, product-scoped roles in v0.2.0 (R4).
- Owner decision (accepted 2026-11-06): a hash-chained administrative audit already in v0.1.0; FEAT-009 extends it in v0.2.0 (R5).
- Owner decision (accepted 2026-11-06): SQLite only in v0.1.0, PostgreSQL with FEAT-002 in v0.3.0, portable SQL from the start (R6).
- Contract: OpenAPI generated from code and checked in CI; device channel specified first in AsyncAPI, served and typed in `levix-platform` from v0.2.0; contract version 1.0.0 (R7).
- Console sessions by cookie with CSRF, CLI sessions and personal API tokens as bearer tokens (R8); secrets encrypted under a key file outside the database (R9); Leptos CSR console embedded in the binary (R10); TLS by default (R11); `cargo-deny` and an SBOM from v0.1.0 (R12).
- No new ADR: these apply accepted ADRs 0001 to 0008.

## 2. Design (`software-design`)

See [plan.md](../../specs/001-foundations/plan.md), Summary, Technical Context, Constitution Check and Project Structure.

## 3. Affected surfaces and files

Surfaces `api`, `console` and `cli` are created; `channel` gets its specification only. New crates under `crates/`, contract under `docs/contracts/`, security documents under `docs/security/`, workflows under `.github/workflows/` (plan.md, Project Structure).

## 4. Tests (`quality-assurance`)

plan.md, Dev Cycle Protocol, Tests.

## 5. Documentation (`architecture-docs`, `user-docs`)

plan.md, Dev Cycle Protocol, Documentation.

## 6. CI, release and observability (`devops-release`)

plan.md, Dev Cycle Protocol, CI, release and observability. Levi creates the `release` environment for `promote.yml` in the repository settings before the release gate.

## 7. Security, migration and rollback

plan.md, Dev Cycle Protocol, Security, migration and rollback.

## 8. Execution steps and milestones

The phases of [tasks.md](../../specs/001-foundations/tasks.md), one pull request each:

1. M1 Threat model and security architecture (FEAT-040)
2. M2 Service skeleton and organizations (FEAT-001)
3. M3 Administrators, roles and TOTP (FEAT-019)
4. M4 Public contract (FEAT-018)
5. M5 Administration CLI (FEAT-021)
6. M6 Web console (FEAT-020)
7. M7 Release readiness, then release candidate v0.1.0-rc.1
