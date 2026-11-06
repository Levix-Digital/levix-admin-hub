# Admin Hub security architecture

How the Admin Hub protects what it governs: who can do what, how they prove it, where secrets live, how actions are recorded and how the Hub is built and shipped. The threats these controls answer, and the ones still open, are in [threat-model.md](threat-model.md). This page describes the Hub as it is today and is edited in place.

## Principles

- **Customer-hosted, zero knowledge, no call home** ([ADR-0004](../adr/0004-always-hosted-by-the-customer-zero-knowledge-no-call-home.md)): the Hub runs in the customer's perimeter, never receives product content and opens no outbound connection except to systems the customer configures.
- **One core, many surfaces** ([ADR-0001](../adr/0001-hexagonal-architecture-with-a-modular-core.md)): the console, the CLI, the API and the device channel call the same use cases, so authorization and audit are enforced once, in the core, for every surface.
- **No default trust**: no default account, no secret in the binary, no first-come claim.
- **Public contract** ([ADR-0006](../adr/0006-public-versioned-contract-for-the-api-and-the-device-channel.md)): auditors read exactly what the Hub accepts and returns.

## Claiming a new Hub

1. The container starts uninitialized; the console only explains how to initialize it.
2. On the host, `levix-hub init --owner-email <e-mail>` creates the instance and a single-use setup link valid for 24 hours, and prints the link with the TLS certificate fingerprint. The link carries its token in the URL fragment, which browsers never send to a server.
3. Whoever opens the link sets a password, enrolls TOTP and becomes the instance owner. Using the link consumes it.

Holding the host is the trust anchor: the same person who can run the container can claim it. `levix-hub recover-owner` uses the same anchor when the instance owner loses every factor.

## Authentication

| Element | Control |
|---|---|
| Passwords | Argon2id (19 MiB, 2 iterations, parallelism 1, tunable); at least 12 characters; refused when in the bundled list of common passwords. |
| Second factor | TOTP (RFC 6238, SHA-1, 6 digits, 30 s), mandatory before any action; one step of clock drift accepted; each step accepted once. |
| Recovery | 10 single-use recovery codes stored as hashes; owners reset other administrators' TOTP; host recovery for the instance owner. |
| Brute force | After 5 failures, a delay per account starting at 30 s and doubling up to 15 minutes; 20 requests per minute per source address on the unauthenticated endpoints (sign-in, links); unknown e-mails cost the same hashing time as known ones. |
| Invitations | Single-use links valid 72 hours, shown once to the inviting owner, delivered by them through a separate channel (the Hub sends no e-mail). |
| Console sessions | Random token in the `__Host-lvx_session` cookie (`HttpOnly`, `Secure`, `SameSite=Strict`) with a per-session CSRF token required in `X-CSRF-Token` on every write; 15 minutes of inactivity, 12 hours absolute (the instance owner tunes both within 5–60 minutes and 1–24 hours); stored server-side as hashes and revocable. |
| CLI sessions | Bearer token stored in a file readable only by the user; same expiry rules. |
| API tokens | Personal, named, prefixed `lvxh_`, 256 random bits, stored as SHA-256, expiry at most 90 days, revocable; they act with the creator's current permissions. |
| Single sign-on | OIDC and SAML 2.0 for administrators arrive with FEAT-004; the local instance owner remains as a break-glass administrator. |

## Authorization

- Roles per organization: owner, administrator, approver, auditor, read-only; the instance owner manages instance settings and creates organizations.
- One fixed permission matrix, published in the contract and enforced in the core; every request is checked against the caller's role in the organization it targets.
- Owners alone change roles, suspend administrators, reset second factors and revoke sessions; the last owner of an organization cannot be removed or demoted.
- Product-scoped roles arrive with product manifests (FEAT-043).

## Secrets and keys

| Secret | Where it lives |
|---|---|
| Secret key (256-bit) | File `keys/secret.key` on the data volume, owner-only permissions, created at the first start (or by `init`), never in the database; behind the key custody port, which a hardware adapter will implement (FEAT-044). |
| TOTP secrets | Encrypted with XChaCha20-Poly1305 under the secret key. |
| Passwords, tokens, links, recovery codes | Hashes only (Argon2id for passwords, SHA-256 for random tokens); compared in constant time. |
| Hub signing key (Ed25519) | Created with enrollment (FEAT-006) on the key custody port; its fingerprint travels in enrollment codes. |

## Audit

- Every administrative action and authentication event (success, denial, failure) becomes one record: sequence, time, actor, organization, action, target, result, source address and metadata.
- Records are append-only and chained: each record's hash covers the previous hash and the record itself. `levix-hub audit verify` checks the chain.
- Owners and auditors read the audit in the console and the CLI. Device events, retention, search and SIEM export extend the same store (FEAT-009, FEAT-034).

## Transport and network

- HTTPS only, TLS 1.2 or later through rustls, with the customer's certificate or a generated one whose fingerprint `init` and `serve` print.
- Plain HTTP only with an explicit setting for deployments behind a TLS-terminating proxy, logged as a warning at every start.
- The Hub process has no HTTP client and makes no outbound connection; a test of the end-to-end run asserts it.
- Devices and product servers connect out to the Hub; the Hub never connects to them (from v0.2.0).

## Data protection

- Personal data limited to administrators' and users' names, e-mails, groups and device names and addresses; no product content.
- The contract refuses unknown fields, so content cannot be stored by accident.
- Logs are structured, carry identifiers only and never passwords, codes, tokens or secrets.

## Supply chain

- Permissive dependencies only, checked with `cargo-deny`, which also fails on known advisories.
- A CycloneDX SBOM with every release.
- Signed container images, reproducible builds and the source bundle for customers arrive with FEAT-039 ([ADR-0007](../adr/0007-proprietary-license-with-source-access-for-customers.md)).

## Assurance

- The threat model is reviewed with every release.
- Independent penetration test before the first large customer and yearly; vulnerability disclosure policy (FEAT-041).
