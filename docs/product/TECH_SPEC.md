# Levix Admin Hub: Technical Specification

The Admin Hub is a Rust service, shipped as a Linux container image (and a Helm chart for
Kubernetes), that the customer runs in its own VPC or on-premises. It exposes a REST API, SCIM
endpoints, inbound webhooks, an outbound-only WebSocket channel for devices and product
servers, a web console served by the Hub itself, and an administration CLI. The architecture
style is hexagonal (`architecture.style` in `.dev-cycle.json`): a framework-free core with
modules per capability and ports for storage, message bus, key custody, identity providers,
ITSM providers and the mirror store. How each part works is described in
[TECDOC.md](../../TECDOC.md).

## Platforms and Stack

- **Hub runtime:** Linux container images for x86_64 and arm64; Docker or Podman for the compact
  profile, Kubernetes 1.28 or later with Helm for the scalable profile.
- **Language:** Rust stable, edition 2021, the same toolchain as the products, so the shared
  `levix-platform` crates compile on both sides
  ([ADR-0002](../adr/0002-rust-service-on-tokio-and-axum-with-a-leptos-web-console.md)).
- **Async runtime and HTTP:** Tokio and Axum, for HTTP, WebSocket and SCIM in one server.
- **Database access:** SQLx with SQLite 3 (compact) and PostgreSQL 15 or later (scalable), one
  set of migrations for both
  ([ADR-0003](../adr/0003-one-codebase-two-installation-profiles-sqlite-and-in-process.md)).
- **Message bus:** in-process channels (compact) or Redis 7 Pub/Sub (scalable).
- **Web console:** Leptos compiled to WASM, embedded in the Hub binary, no external assets.
- **Cryptography and formats:** the `levix-platform` crates (policy model and schemas, Ed25519
  signing and verification, license format, product data definitions, channel messages)
  pinned by git tag
  ([ADR-0005](../adr/0005-shared-policy-model-signatures-and-formats-from-levix-platfo.md)).
- **Browsers for the console:** the current and previous major versions of Chrome, Edge and
  Firefox.
- **Supply chain:** `cargo-deny` (permissive licenses, advisories), SBOM (CycloneDX) per
  release, signed container images, reproducible builds with published instructions
  ([ADR-0007](../adr/0007-proprietary-license-with-source-access-for-customers.md)).

## Architecture Overview

```
 Console (browser) ─┐                                   ┌─► Storage: SQLite | PostgreSQL
 CLI ───────────────┤                                   ├─► Bus: in-process | Redis
 SCIM (IdP) ────────┼─► Driving adapters ─► hub-core ───┼─► Key custody (Hub key, secrets)
 Webhooks (ITSM) ───┤   (api, channel,     (use cases,  ├─► IdP (OIDC, SAML)
 Devices, servers ──┘    console, cli)      modules)    ├─► ITSM (ServiceNow, Jira)
   outbound WSS                                         └─► Mirror store, SIEM export
```

- The Hub has no product-specific code: each product (current, future or third party) is
  registered by importing a product manifest signed by Levix that declares its identifier,
  client kinds, policy schema, data definition formats, seat model, telemetry counters and
  commands; the Hub validates policies, telemetry and commands against it. The license lists
  which products a Hub governs.
- Every action is one core use case shared by all surfaces
  ([ADR-0001](../adr/0001-hexagonal-architecture-with-a-modular-core.md)).
- Devices and product servers open the channel to the Hub; the Hub never connects to them.
  Every Hub-to-device message is signed by the Hub key pinned at enrollment; devices
  authenticate by signing a nonce with their Ed25519 identity.
- Commands for offline devices are queued and delivered on reconnection.
- The contract (OpenAPI and the channel message specification) is public and versioned
  ([ADR-0006](../adr/0006-public-versioned-contract-for-the-api-and-the-device-channel.md)).

## Non-Functional Requirements

### Performance

- **Compact profile:** governs 500 devices with the default heartbeat on 2 vCPU and 4 GB of RAM;
  the container starts in under 10 s.
- **Scalable profile:** governs 50,000 connected devices across replicas, adding capacity by
  adding replicas.
- **Policy propagation:** a published policy reaches 95% of connected devices in under 10 s.
- **Kill-switch:** a revocation reaches a connected device in under 5 s (p95).
- **Console and API:** p95 response time under 300 ms for interactive operations at the profile's
  rated fleet size.
- **Heartbeat:** default interval 30 s, configurable by policy.

### Security and Privacy

- **Zero knowledge:** no content is ever accepted, stored or logged; payload schemas reject
  unknown content fields
  ([ADR-0004](../adr/0004-always-hosted-by-the-customer-zero-knowledge-no-call-home.md)).
- **No call home:** the Hub makes no outbound connection except to systems the customer
  configures (identity provider, ITSM, SIEM, mirror source).
- **Administrator authentication:** SSO (OIDC or SAML 2.0) or local administrator accounts with
  Argon2id passwords and a mandatory TOTP second factor; sessions expire after inactivity.
- **Authorization:** role-based (organization owner, administrator, approver, auditor,
  read-only), per organization and per product; every administrative action is audited.
- **Keys and secrets:** the Hub signing key and the secrets it holds (ITSM credentials,
  corporate model endpoint keys, break-glass passwords) are encrypted at rest with a key
  separate from the database, through the key custody port; corporate keys are delivered to
  devices encrypted to the device public key.
- **Break-glass:** revealing a device root password requires the dedicated role and a reason,
  optionally a second approver; every reveal and every reported use is audited and triggers
  rotation.
- **Device certificates:** short-lived, renewed on each online heartbeat; the offline validity is
  set by the company (default 72 h); once expired, the corporate environment is suspended until
  it reconnects.
- **Transport:** TLS 1.2 or later on every endpoint, with the customer's certificate or a
  generated one whose fingerprint travels in the enrollment code; webhooks verified with
  HMAC-SHA256.
- **Audit:** append-only, hash-chained records; exportable to the customer's SIEM.
- **Personal data:** names, e-mails, groups, device names and addresses only; retention set by
  the customer; supports the obligations of PIPEDA, Quebec Law 25, LGPD and GDPR as processor
  data stays in the customer's perimeter.
- **Assurance:** independent penetration test before the first large customer and yearly;
  published vulnerability disclosure policy; threat model kept in the repository.

### Availability and Recovery

- Devices keep working offline within the policy's offline validity, so a Hub outage never
  stops users immediately.
- **Compact:** one instance; `backup` and `restore` commands of the CLI produce and restore a
  consistent encrypted archive; default recovery point 24 h, recovery time under 1 h.
- **Scalable:** stateless replicas behind a load balancer; availability target 99.9% given a
  highly available PostgreSQL and Redis provided by the customer.
- Upgrades apply additive migrations; a version can be rolled back to the previous minor
  release.

### Accessibility and Localization

- Console conforms to WCAG 2.1 level AA.
- Console languages: English, Canadian French and Brazilian Portuguese; dates and times in the
  administrator's time zone; API and contract in English.

## Integrations

| System | Direction | Protocol | When unavailable |
|---|---|---|---|
| Levix products and servers | Device to Hub (outbound channel) | WSS, signed messages | Devices apply the last valid policy until the offline validity ends |
| Identity provider | Hub to IdP (SSO), IdP to Hub (SCIM) | OIDC, SAML 2.0, SCIM 2.0 | SSO sign-in fails; local break-glass administrator remains; SCIM changes queue at the IdP |
| ITSM (ServiceNow, Jira Service Management) | Both | REST out, HMAC webhooks in | Requests stay pending in the Hub and can be decided in the console |
| Generic webhooks | Both | HTTPS JSON, HMAC | Outbound events are retried with backoff |
| SIEM | Hub to SIEM | Syslog (RFC 5424) or JSON over HTTPS | Events buffer in the Hub and are sent on recovery |
| Mirror source | IT to Hub | Signed bundle import (offline) or download by an IT machine | Fleet stays on the released versions |
| `levix-platform` | Build-time dependency | Rust crates by git tag | Not a runtime dependency |

## Data

- **Main entities:** organization; product manifest (signed by Levix: identifier, client
  kinds, policy schema, data definition formats, seat model, telemetry counters, commands); administrator and role; user, group and membership (directory, SSO or SCIM origin,
  per product); activation code; device and product server (identity public key, status,
  certificate, inventory, last heartbeat); enrollment request; license and seat assignment;
  policy and policy version (signed); command queue; approval request and ITSM ticket link;
  break-glass secret and reveal log; audit record; telemetry counters; mirror artifact and
  release ring; data unit, curator and data definition; webhook subscription.
- **Where it lives:** one database per Hub (SQLite file or PostgreSQL); mirror artifacts on a
  volume or object storage behind the mirror store port.
- **Retention:** audit and telemetry retention set by the customer (default audit 2 years,
  telemetry 90 days); deleted users and devices keep pseudonymous audit references.
- **Migrations:** additive or backward compatible only, tested on both databases; destructive
  changes need a plan section and a backup step.

## Constraints

- Always hosted by the customer; no Levix-hosted service and no call home.
- Never receives or stores content; passwords and user keys never go through the Hub.
- Rust only, sharing the `levix-platform` crates; anything shared is built there through a pull
  request, generic and reusable.
- Permissive dependencies only (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0).
- Proprietary license with source access for customers; reproducible builds.
- Must run in air-gapped networks: no external assets, offline license verification, offline
  mirror import.
- Small team: one person with development tooling until the second hire; scope is sequenced by
  the roadmap, not cut.

## Open Questions

- Hardware security module support (PKCS#11) for the Hub signing key: which release.
- Scale targets (500 devices compact, 50,000 scalable) to validate with load tests.
