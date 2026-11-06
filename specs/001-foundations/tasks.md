# Tasks: Foundations v0.1.0

**Input**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/permissions.md](contracts/permissions.md)

## Format: `[ID] [P?] [Story] Description`

`[P]` can run in parallel (different files, no dependency). `[USn]` is the user story of spec.md. Each phase is one milestone of the plan and one pull request into `main`. Tests are written with the code they cover and must pass before the phase ends.

## Phase 1: Threat model and security architecture (M1)

- [ ] T001 [P] [US5] Write `docs/security/threat-model.md`: assets, actors, trust boundaries of every release (browser, CLI, host, database and key file; device channel, IdP, ITSM, SIEM, mirror), STRIDE threats per boundary, each with a status (mitigated in v0.1.0, planned with a backlog id, accepted with a reason)
- [ ] T002 [P] [US5] Write `docs/security/security-architecture.md`: authentication, authorization matrix, sessions, secrets and keys, audit chain, transport, zero knowledge, supply chain
- [ ] T003 [P] Edit `docs/architecture/admin-hub-design.md`: seats per user by default, per device for OT; link the security documents
- [ ] T004 Capture as backlog items the threats planned for later releases that no item covers yet

## Phase 2: Service skeleton and organizations (M2)

- [ ] T005 Create the Cargo workspace (`Cargo.toml`, version 0.1.0) with the crates of plan.md, `rust-toolchain.toml`, `deny.toml`, `rustfmt.toml`
- [ ] T006 [P] Add `.github/workflows/ci.yml` from the protocol template (fmt, clippy, tests, `cargo-deny`) and set `test_commands` and `devops.version_files` in `.dev-cycle.json`
- [ ] T007 [P] Core: `CoreError`, clock, randomness and storage ports, `instance` and `organization` modules with use cases (initialize instance, create, rename, archive, list, get)
- [ ] T008 Storage: `hub-storage-sqlite` with migration 1 (instance, organization), schema version check, refusal on newer schema, integration tests
- [ ] T009 Crypto: `hub-crypto` file key custody (create on `init`, permissions 0600, refusal when missing)
- [ ] T010 API base: Axum router `/api/v1`, error object, unknown-field refusal, request id, JSON logs, `/healthz`, `/readyz`
- [ ] T011 [US1] TLS: customer certificate or generated self-signed certificate with printed fingerprint; `--insecure-behind-proxy`
- [ ] T012 [US1] Server binary `levix-hub`: `serve`, `init` (instance, key file, setup link), `migrate`; configuration by file and environment
- [ ] T013 [P] `deploy/Containerfile` (distroless, non-root, data volume) and a container smoke test in CI (ready in under 10 s)
- [ ] T014 Test: no outbound socket during the end-to-end run

## Phase 3: Administrators, roles and TOTP (M3)

- [ ] T015 [US2] Core: administrator, role assignment and permission matrix (table test over every action and role)
- [ ] T016 [US2] Core: credential links (setup, invitation, recovery) single use and expiry; last-owner protection
- [ ] T017 [US2] Crypto adapters: Argon2id hasher, TOTP (one step of drift, replay refused), recovery codes, XChaCha20-Poly1305 secret box
- [ ] T018 [US2] Core: sign-in with password and TOTP, lockout with increasing delays, sessions with inactivity and absolute expiry, revocation
- [ ] T019 [US4] Core: personal API tokens (create, list, revoke, expiry ≤ 90 days, creator's permissions at use time)
- [ ] T020 [US2] Core and storage: append-only hash-chained administrative audit written by every use case; `levix-hub audit verify`
- [ ] T021 Storage: migration 2 (administrator, recovery_code, role_assignment, credential_link, session, api_token, audit_record), integration tests including concurrent use of one link
- [ ] T022 [US2] API: auth endpoints, console cookie session with CSRF, bearer sessions and API tokens, administrators, roles, audit
- [ ] T023 [US1] `levix-hub recover-owner` on the host; audit of host actions
- [ ] T024 Test: logs of the end-to-end run contain no password, code, token or secret

## Phase 4: Public contract (M4)

- [ ] T025 [US5] Describe every endpoint with `utoipa`; generate `docs/contracts/api/openapi.json`; CI check that it matches
- [ ] T026 [P] [US5] CI breaking-change check with `oasdiff` against the last release tag
- [ ] T027 [P] [US5] Channel specification v1 in `docs/contracts/channel/v1/` (AsyncAPI 3.0, JSON Schemas for envelope, hello, challenge, authenticate, error; Hub signature format; version negotiation) with validated examples
- [ ] T028 [US5] `docs/contracts/CHANGELOG.md` (contract 1.0.0) and the permission matrix published from contracts/permissions.md

## Phase 5: Administration CLI (M5)

- [ ] T029 [US4] `levix-hub-cli`: configuration, `login` (password, TOTP, CA or `--fingerprint`), session file 0600, `logout`
- [ ] T030 [US4] Commands for every API operation: organizations, administrators, roles, invitations, sessions, tokens, audit, own profile and TOTP
- [ ] T031 [US4] `--output json`, stable error object and exit codes
- [ ] T032 [US4] End-to-end tests of stories 1 to 4 against a spawned server; SC-004 parity test (every API operation has a command)

## Phase 6: Web console (M6)

- [ ] T033 [US3] `hub-console` Leptos CSR app with `trunk`; embedded in `hub-api` with strict CSP; not-initialized page
- [ ] T034 [US3] Pages: setup link, sign-in with TOTP, TOTP enrollment and recovery codes, organizations, administrators and roles, invitations, own profile, sessions, API tokens, audit
- [ ] T035 [P] [US3] Translations `en`, `fr-CA`, `pt-BR` (Fluent), browser language detection, profile choice
- [ ] T036 [US3] Playwright flows and axe-core scan of every page in the three languages in CI
- [ ] T037 [US3] Inactivity sign-out and session expiry handling in the UI

## Phase 7: Release readiness (M7)

- [ ] T038 README: Installation, Quick Start, Requirements, Configuration, CLI, Troubleshooting, Documentation; `check_readme` zero errors
- [ ] T039 [P] Subsystem pages `service-and-storage.md`, `administration-and-access.md`, `console.md`, `cli.md`; `build_index`, `check_docs`
- [ ] T040 [P] `tests/MANUAL_TESTS.md` suite for this release; `manual_tests validate`
- [ ] T041 [P] `.github/workflows/release.yml` (images x86_64 and arm64, CLI binaries, CycloneDX SBOM, pre-release for `-rc`) and `promote.yml`; `check_ci_policy` zero errors
- [ ] T042 Update the threat model statuses to what was delivered; measure SC-002 and SC-003 and record them in the pull request

## Dependencies

Phase 1 has none. Phase 2 blocks every later phase. Phase 3 needs Phase 2. Phase 4 needs Phase 3 (all endpoints). Phases 5 and 6 need Phase 4 and can run in either order. Phase 7 needs all.
