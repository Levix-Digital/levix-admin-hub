# Research: Foundations v0.1.0

Each decision lists what was chosen, why, and what was rejected. Decisions that change an accepted ADR or the scope of a backlog item are marked **owner decision**.

## R1. One plan for the release, one pull request per milestone

- **Decision**: one Spec Kit feature (`specs/001-foundations`) covers the six items of v0.1.0; each milestone is merged into `main` through its own pull request from `feat/001-foundations`, brought up to date with `main` after each merge.
- **Rationale**: the items depend on each other (the console and the CLI only call the API; the API needs the core); one approval gate for the release instead of six; small reviewable pull requests.
- **Alternatives**: one plan per item (six gates for one release goal); one large pull request (unreviewable).

## R2. Claiming a new Hub: setup link from the host

- **Decision**: `levix-hub init` on the Hub host creates the instance, the secret key file and a single-use setup link (24 h) for the instance owner. The network never offers account creation. `levix-hub recover-owner` (host only) issues a new link if the owner loses every factor.
- **Rationale**: proves control of the host, the same trust anchor as installing the container; no default password, no first-come claim from the network.
- **Alternatives**: default `admin/admin` (audit finding by definition); first browser to arrive becomes owner (race on a shared network); claim code in the container log only (logs are often shipped to shared systems).

## R3. Several organizations per Hub, with an instance owner

- **Decision**: an instance hosts one or more organizations; the instance owner (created at claim) manages instance settings and creates organizations; organization owners manage their own organization only.
- **Rationale**: groups with subsidiaries and IT providers that run one Hub for several entities; every record already carries its organization, so later isolation tests are meaningful.
- **Alternatives**: one organization per Hub (simpler now, a breaking data change later).

## R4. Local administrator accounts only; product-scoped roles later

- **Decision**: v0.1.0 has local accounts with Argon2id (memory 19 MiB, 2 iterations, parallelism 1, the OWASP baseline, tunable) and mandatory TOTP; SSO for administrators comes with FEAT-004 (v0.4.0), product-scoped roles with product manifests (FEAT-043, v0.2.0).
- **Rationale**: matches the roadmap; product roles cannot be designed before products are registered.

## R5. Administrative audit from the first release (owner decision: accepted 2026-11-06)

- **Decision**: v0.1.0 writes every administrative and authentication event to an append-only, hash-chained table, readable by owners and auditors in the console and CLI. FEAT-009 (v0.2.0) adds device events, retention, search and export on the same store.
- **Rationale**: the TECH_SPEC says every administrative action is audited; a release that manages administrators without an audit would be the first finding of the threat model.
- **Alternatives**: structured logs only until v0.2.0 (logs are not tamper-evident and not readable by auditors in the product).

## R6. Compact profile only; PostgreSQL with FEAT-002 (owner decision: accepted 2026-11-06)

- **Decision**: v0.1.0 ships the SQLite adapter only. Migrations use portable SQL (no SQLite-only types or functions) and the storage port hides the database; the PostgreSQL adapter and its CI job arrive with FEAT-002 (v0.3.0). ADR-0003 is unchanged: both profiles exist from v0.3.0, before any customer runs the scalable one.
- **Rationale**: the first deployments are compact; testing migrations on PostgreSQL without an adapter proves nothing.
- **Alternatives**: both adapters now (about a week more, no user before v0.3.0).

## R7. Contract: OpenAPI generated from code, channel specification written first

- **Decision**: REST endpoints under `/api/v1`, described with `utoipa`; the generated `docs/contracts/api/openapi.json` is committed, and CI fails when it differs from the code or when `oasdiff` finds a breaking change without a new major version. The device channel is a hand-written AsyncAPI 3.0 document with JSON Schemas in `docs/contracts/channel/v1/` (endpoint `/ws/device/v1`, envelope, version negotiation, Ed25519 nonce handshake, Hub signature format); it is served from v0.2.0, and its Rust types are created then in `levix-platform` (ADR-0006, FEAT-016 there).
- **Contract versioning**: the contract has its own semantic version, starting at 1.0.0 with this release; breaking changes need a new major version and the support window of ADR-0006.
- **Rationale**: generated docs cannot drift from the server; the channel has no server yet, so it is specified first, which lets products start their client.
- **Alternatives**: hand-written OpenAPI (drifts); gRPC (rejected in ADR-0006).

## R8. Sessions and tokens

- **Decision**: console sessions in an `HttpOnly`, `Secure`, `SameSite=Strict` cookie with a per-session CSRF token; CLI sessions as bearer tokens stored in a file readable only by the user (`0600`); personal API tokens prefixed `lvxh_`, 256 bits of randomness, stored as SHA-256, expiry at most 90 days. Inactivity 15 min, absolute 12 h. Sessions are rows in the database (revocable).
- **Alternatives**: JWT (not revocable without a list, larger attack surface); OS keyring for the CLI (heavy platform dependencies; can be added later).

## R9. Secrets at rest

- **Decision**: TOTP secrets are encrypted with XChaCha20-Poly1305 under a 256-bit key kept in a file outside the database (`keys/secret.key`, created at the first start), behind the key custody port. The Hub signing key arrives with enrollment (v0.2.0) on the same port. PKCS#11 stays an open question of the TECH_SPEC.

## R10. Console

- **Decision**: Leptos in client-side rendering, compiled to WASM with `trunk`, the build output embedded in the server binary; strict Content Security Policy (`default-src 'self'`, WASM allowed); translations in Fluent files (`en`, `fr`, `es`); accessibility checked by axe-core through Playwright in CI on every page and language.
- **Alternatives**: server-side rendering with hydration (more moving parts for an admin console); a JavaScript framework (second language and toolchain, against ADR-0002).

## R11. TLS and network

- **Decision**: `rustls` through `axum-server`; the customer's certificate and key, or a generated self-signed certificate whose SHA-256 fingerprint is printed by `init` and `serve` and accepted by the CLI with `--fingerprint`. Plain HTTP only with `--insecure-behind-proxy` and a warning in every log line of startup. No outbound connection: there is no HTTP client in the server.

## R12. Supply chain in v0.1.0

- **Decision**: `cargo-deny` (licenses and advisories) and a CycloneDX SBOM (`cargo-cyclonedx`) attached to every release; signed images and reproducible builds remain FEAT-039 (v0.3.0).
- **Rationale**: Levi's Phase 0 answer put the SBOM in v0.1; the cheap part lands now.

## R13. Dependencies (all permissive)

`tokio`, `axum`, `axum-server` with `rustls`, `tower-http`, `sqlx` (SQLite), `serde`, `utoipa`, `argon2`, `hmac` and `sha1` (TOTP), `chacha20poly1305`, `sha2`, `rand`, `tracing`, `clap`, `reqwest` with `rustls` (CLI only), `leptos`, `fluent-bundle`, `rust-embed`. `cargo-deny` enforces the license list of the TECH_SPEC.
