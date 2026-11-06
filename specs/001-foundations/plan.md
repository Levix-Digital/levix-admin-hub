# Implementation Plan: Foundations v0.1.0

**Branch**: `feat/001-foundations` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-foundations/spec.md`

## Summary

The first working Admin Hub: a Rust service in a container that IT claims from the host, with organizations, administrators with roles and mandatory TOTP, sessions and personal API tokens, a hash-chained administrative audit, a web console embedded in the binary, an administration CLI over the public API, the published contract (OpenAPI for the API, AsyncAPI for the device channel) and the threat model. Decisions and rejected alternatives are in [research.md](research.md); entities in [data-model.md](data-model.md); permissions in [contracts/permissions.md](contracts/permissions.md).

## Technical Context

**Language/Version**: Rust stable (1.97 at planning time), edition 2021; console compiled to `wasm32-unknown-unknown`.

**Primary Dependencies**: Tokio, Axum, axum-server with rustls, SQLx (SQLite), utoipa, argon2, totp-rs, chacha20poly1305, sha2, tracing, clap, reqwest with rustls (CLI), Leptos (CSR), fluent-bundle, rust-embed.

**Storage**: SQLite 3 file per Hub (compact profile); secret key file beside it.

**Testing**: `cargo test` (unit tests in the core, integration tests of the storage adapter and of the HTTP API against a temporary database), contract tests (generated OpenAPI equals the committed file; `oasdiff` breaking check), CLI end-to-end tests against a spawned server, Playwright with axe-core for console flows and accessibility.

**Target Platform**: Linux container images (x86_64 and arm64); CLI binaries for Linux, macOS and Windows.

**Project Type**: web service with an embedded web console and a CLI (Cargo workspace).

**Performance Goals**: ready in under 10 s; p95 under 300 ms for interactive operations (TECH_SPEC).

**Constraints**: no outbound connections, no external assets, no content, permissive licenses only, TLS by default.

**Scale/Scope**: tens of administrators and organizations per Hub in v0.1.0.

## Constitution Check

| Rule (constitution) | How this plan complies |
|---|---|
| Hexagonal style, framework-free core (`crates/hub-core`, forbidden imports) | Domain and use cases in `hub-core`; Axum, SQLx, Leptos, reqwest only in adapters; `check_design` in CI. |
| Simplicity first | One storage adapter now (SQLite); ports only where a second implementation or a test seam exists today (storage, clock, randomness, password hashing, secret box, key custody). |
| Explicit errors | One `CoreError` enum per module translated once at each adapter edge into the contract's error object. |
| Evidence: automated tests | Every use case unit-tested; adapters integration-tested; manual tests only for real authenticator apps, screen readers and the air-gapped check. |
| Zero knowledge, no call home (ADR-0004) | Unknown fields refused; no HTTP client in the server; a test asserts no outbound socket during the end-to-end run. |
| Public versioned contract (ADR-0006) | Contract committed under `docs/contracts/`, checked in CI. |
| Proprietary license, permissive dependencies (ADR-0007, TECH_SPEC) | `cargo-deny` in CI. |
| No product code in the Hub (ADR-0008) | No product appears in this release. |
| No AI attribution | `_no_ai` on every pull request text. |

No violation; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/001-foundations/
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/permissions.md
├── plan.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml                      # workspace, version 0.1.0
crates/
├── hub-core/                   # domain, use cases, ports (no framework)
│   └── src/{instance,organization,administrator,auth,session,token,audit,permission}/
├── hub-storage-sqlite/         # storage port adapter, migrations/
├── hub-crypto/                 # Argon2id, TOTP, XChaCha20-Poly1305, file key custody
├── hub-api/                    # surface "api": Axum REST /api/v1, sessions, CSRF, OpenAPI, serves the console
├── hub-console/                # surface "console": Leptos CSR, Fluent translations (en, fr-CA, pt-BR)
├── hub-cli/                    # surface "cli": binary `levix-hub-cli`
└── hub-server/                 # composition root: binary `levix-hub` (serve, init, recover-owner, migrate, audit verify)
docs/contracts/
├── api/openapi.json            # generated, committed
├── channel/v1/                 # AsyncAPI 3.0 + JSON Schemas (surface "channel", served from v0.2.0)
└── CHANGELOG.md                # contract versions
docs/security/
├── threat-model.md
└── security-architecture.md
tests/MANUAL_TESTS.md
deploy/Containerfile
.github/workflows/{ci,release,promote}.yml
deny.toml
```

**Structure Decision**: one crate per hexagon part and per surface, matching `architecture.core_path` and `architecture.surfaces` in `.dev-cycle.json`; the surface `channel` has only its specification in this release.

## Complexity Tracking

None.

## Dev Cycle Protocol

### Backlog items and release

FEAT-018, FEAT-001, FEAT-019, FEAT-020, FEAT-021, FEAT-040; release v0.1.0 Foundations (docs/ROADMAP.md). Also corrected here: `docs/architecture/admin-hub-design.md` still calls seats per user or per device an open decision; the strategy and the VISION set per user by default and per device for OT.

### Tests (quality-assurance)

- Unit: permission matrix (every action × role, generated table test), password rules, TOTP window and replay refusal, recovery codes single use, link expiry and single use, lockout delays, last-owner protection, audit hash chain and its verifier.
- Integration: storage adapter on a temporary SQLite file (migrations from empty, constraints, concurrent single-use links); HTTP API for every endpoint including CSRF, cookie flags, unknown-field refusal, session expiry; startup refusal on read-only volume and on a newer schema.
- Contract: generated OpenAPI equals `docs/contracts/api/openapi.json`; `oasdiff breaking` against the last release; AsyncAPI document and JSON Schemas validate; example messages validate against the schemas.
- End to end: CLI scripts covering stories 1 to 4 against a spawned server; a check that the server opens no outbound socket.
- Console: Playwright flows for claim, sign-in with TOTP, invitation, role change; axe-core scan of every page in the three languages.
- Manual (`tests/MANUAL_TESTS.md`, new suite): install the container and claim it in a real browser verifying the fingerprint; enroll TOTP with two real authenticator apps; recovery code and host recovery; keyboard-only and screen reader pass (NVDA on Windows, VoiceOver on macOS); the three languages read by a person; air-gapped run with the network cut.
- Acceptance criteria: the spec's acceptance scenarios and success criteria SC-001 to SC-006.

### Documentation (architecture-docs, user-docs)

- New subsystem pages in `docs/architecture/`: `service-and-storage.md` (workspace, ports, profiles), `administration-and-access.md` (accounts, roles, sessions, tokens, audit), `console.md`, `cli.md`; `admin-hub-design.md` edited in place (seats, links to the new pages); `build_index` updates `TECDOC.md`.
- `docs/security/threat-model.md` (STRIDE per trust boundary: browser to Hub, CLI to Hub, host to Hub, Hub to database and key file, and the later boundaries device to Hub, IdP, ITSM, SIEM, mirror, each threat with a status) and `docs/security/security-architecture.md`.
- No new ADR: the decisions of research.md apply accepted ADRs. R5 and R6 change only the scope of backlog items, recorded in the backlog and the roadmap.
- README: Installation (container, `init`, claim), Quick Start, Requirements, Configuration, CLI, Troubleshooting, Documentation; `check_readme` with zero errors.

### CI, release and observability (devops-release)

- New workflows from the protocol templates for the trunk model: `ci.yml` on pull requests into `main` (fmt, clippy `-D warnings`, tests, console build, contract checks, `cargo-deny`, Playwright), `release.yml` (container images for both architectures, CLI binaries, SBOM, pre-release for `-rc` tags) and `promote.yml` (Levi promotes the accepted candidate; needs a `release` environment with Levi as required reviewer, set by Levi in the repository settings). `check_ci_policy` with zero errors.
- `.dev-cycle.json`: `devops.version_files` set to `Cargo.toml`; `test_commands` set to the cargo commands.
- Version: 0.1.0, first release (minor).
- Observability: JSON logs through `tracing` (request id, administrator id, action, result, latency; never passwords, codes, tokens or secrets, enforced by a test that scans logs of the end-to-end run); `/healthz` (liveness) and `/readyz` (database and key file); `levix-hub audit verify` for the chain.

### Security, migration and rollback

- Secrets: none in Git; the secret key file and database live on the data volume; tokens, links and recovery codes stored hashed; TOTP secrets encrypted; constant-time comparisons.
- Migrations: first schema (version 1); additive from here on; the Hub refuses to start on a newer schema.
- Rollback: before any customer installs, rollback is redeploying the previous image with its data volume; from v0.1.0 on, a schema change keeps the previous minor able to read the data (TECH_SPEC).

## Milestones

1. **M1 Threat model and security architecture** (FEAT-040, docs only; also the seats correction in the design page).
2. **M2 Service skeleton and organizations** (FEAT-001): workspace, CI, container, server with TLS, health, logs, SQLite, `init`, organizations.
3. **M3 Administrators, roles and TOTP** (FEAT-019): accounts, invitations, TOTP and recovery codes, sessions, API tokens, permission matrix, administrative audit, `recover-owner`.
4. **M4 Public contract** (FEAT-018): complete `/api/v1`, OpenAPI committed and checked, channel specification v1, contract changelog.
5. **M5 Administration CLI** (FEAT-021).
6. **M6 Web console** (FEAT-020).
7. **M7 Release readiness**: README, subsystem pages, manual test suite, release workflows; then Phase 8 (release candidate v0.1.0-rc.1).
