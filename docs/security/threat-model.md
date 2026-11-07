# Admin Hub threat model

The threats to the Admin Hub across every trust boundary of the product, including the boundaries that later releases open, and what answers each one today. It is edited in place with every release: a threat moves from planned to mitigated when the release that answers it ships. How the controls work is described in [security-architecture.md](security-architecture.md).

Method: STRIDE per trust boundary (Spoofing, Tampering, Repudiation, Information disclosure, Denial of service, Elevation of privilege). Each threat has one status:

- **Mitigated (vX.Y.Z)**: answered by the release named.
- **Planned (ID)**: answered by the backlog item named, in the release the roadmap gives it.
- **Accepted**: not answered, with the reason.

## Scope and assumptions

- The Hub runs in the customer's perimeter (VPC or on-premises), hosted and operated by the customer's IT ([ADR-0004](../adr/0004-always-hosted-by-the-customer-zero-knowledge-no-call-home.md)). Levix has no access to it.
- The host (container runtime, operating system, volumes) is administered by the customer. Anyone with root on the host can read the database and the key files; the Hub does not defend against a hostile host administrator, it makes their actions on the Hub visible where it can.
- The Hub holds no content of the products: documents, prompts, conversations and their derivatives never reach it.
- Devices, product servers and administrators reach the Hub over networks the customer does not fully trust.

## Assets

| Asset | Why it matters |
|---|---|
| Administrator credentials (passwords, TOTP secrets, recovery codes, sessions, API tokens) | Whoever holds them governs the fleet. |
| Hub signing key (from v0.2.0) | Devices accept policies, certificates and commands signed by it. |
| Secret key file | Decrypts TOTP secrets and, later, every secret the Hub holds. |
| Break-glass passwords (from v1.0.0) | Root access to devices. |
| Corporate secrets delivered to devices (ITSM credentials, corporate endpoint keys, from v0.4.0) | Access to the customer's systems. |
| Directory data (names, e-mails, groups, devices) | Personal data under PIPEDA, Quebec Law 25, LGPD and GDPR. |
| Administrative audit | Evidence for the customer's auditors; must be complete and untampered. |
| Policies and licenses | Decide what devices may do and which products run. |
| Availability of the Hub | Kill-switch and policy changes depend on it; devices keep working offline within the policy's validity. |

## Actors

- **External attacker** on the network between clients and the Hub.
- **Malicious or compromised administrator**, with any role.
- **Compromised device or product server**, holding a valid device identity.
- **Insider on the host** with shell access to the container or volumes.
- **Supply chain attacker** targeting a dependency, the build, a release artifact or a Levix signing key.
- **Curious integrator** with a valid API token.

## Trust boundaries

| ID | Boundary | Opened in |
|---|---|---|
| TB1 | Administrator browser ↔ Hub console and API | v0.1.0 |
| TB2 | CLI and API clients ↔ Hub API | v0.1.0 |
| TB3 | Hub host operator ↔ Hub (initialization, recovery, files) | v0.1.0 |
| TB4 | Hub process ↔ database, key files and backups | v0.1.0 |
| TB5 | Devices and product servers ↔ Hub channel | v0.2.0 |
| TB6 | Levix (manifests, licenses, release artifacts) ↔ Hub | v0.2.0 |
| TB7 | Identity provider ↔ Hub (SSO, SCIM) | v0.4.0 |
| TB8 | ITSM and webhooks ↔ Hub | v0.4.0 |
| TB9 | Hub ↔ SIEM | v0.3.0 |
| TB10 | Mirror bundle import ↔ Hub | v0.5.0 |
| TB11 | Break-glass reveal ↔ administrator | v1.0.0 |

## Threats

### TB1 and TB2: browser, CLI and API clients

| ID | STRIDE | Threat | Answer | Status |
|---|---|---|---|---|
| T-01 | S | Claiming a fresh Hub from the network before its owner | No account creation over the network; the first account comes only from a single-use setup link created on the host by `levix-hub init`. | Mitigated (v0.1.0) |
| T-02 | S | Password guessing or credential stuffing against administrators | Argon2id hashes, minimum length and common-password list, mandatory TOTP, increasing delays per account and a request limit per source address on the authentication endpoints, all audited. | Mitigated (v0.1.0) |
| T-03 | S | Phishing of password and TOTP code (real-time relay) | TOTP limits the window but does not stop a live relay. Phishing-resistant factors (WebAuthn) and SSO with the customer's IdP policies answer it. | Planned (FEAT-004); WebAuthn: Accepted for v0.1.0, revisited at the penetration test (FEAT-041) |
| T-04 | S | Session theft through the browser (cross-site scripting, cookie leakage) | `HttpOnly`, `Secure`, `SameSite=Strict` cookies; strict Content Security Policy with no external origin and no inline script other than the console loader, allowed by its SHA-256 hash; output encoding by the console framework. | Mitigated (v0.1.0) |
| T-05 | T | Cross-site request forgery against the console | `SameSite=Strict` plus a per-session CSRF token on every state-changing request. | Mitigated (v0.1.0) |
| T-06 | S | Replay of a used TOTP code or of a setup, invitation or recovery link | Each TOTP time step accepted once per account; links are single use, expire (24 h setup, 72 h invitation) and are stored hashed. | Mitigated (v0.1.0) |
| T-07 | I | Interception on the network | TLS 1.2 or later on every endpoint; the CLI verifies the CA chain or the pinned SHA-256 fingerprint; plain HTTP only when explicitly configured behind a TLS-terminating proxy. | Mitigated (v0.1.0) |
| T-08 | E | An administrator acts outside their role or organization | One permission matrix enforced in the core for every surface; every request checked against the role in the organization it targets; table tests over every action and role. | Mitigated (v0.1.0) |
| T-09 | E | An API token outlives or exceeds its creator's rights | Tokens act with the creator's current permissions at use time, expire within 90 days, are revocable and stored hashed; suspending the creator stops its tokens. | Mitigated (v0.1.0) |
| T-10 | E | Removing the last owner locks the organization out, or an owner is removed by a lower role | Last-owner protection; only owners change roles. | Mitigated (v0.1.0) |
| T-11 | R | An administrator denies an action | Every administrative and authentication event recorded with actor, organization, target, result and source address in the hash-chained audit. | Mitigated (v0.1.0) |
| T-12 | I | Excess data in responses or errors (other organizations, secrets, stack traces) | Responses built from contract types only; errors are a stable object without internals; organization filter in every query. | Mitigated (v0.1.0) |
| T-13 | I | Content sent to the Hub by mistake or by design | The contract refuses unknown fields; the Hub has no field for content (zero knowledge). | Mitigated (v0.1.0) |
| T-14 | D | Flooding the API or the authentication endpoints | Request limits per source address on unauthenticated endpoints; body size limits; timeouts. Large-scale denial of service is left to the customer's network controls. | Mitigated (v0.1.0) for the application; network flooding Accepted (customer perimeter) |
| T-15 | S | Lost authenticator locks an administrator out, or recovery is abused | Ten single-use recovery codes; an owner resets another administrator's TOTP; the instance owner is recovered only from the host (`recover-owner`); all audited. | Mitigated (v0.1.0) |

### TB3 and TB4: host, database, key files and backups

| ID | STRIDE | Threat | Answer | Status |
|---|---|---|---|---|
| T-20 | I | Theft of the database file reveals credentials | Passwords as Argon2id, tokens, links and recovery codes as SHA-256 hashes, TOTP secrets encrypted with XChaCha20-Poly1305 under a key kept in a separate file. | Mitigated (v0.1.0) |
| T-21 | I | Theft of the database and the key file together | The key file is created with owner-only permissions on the data volume; protecting both is the host's job. Hardware key custody (PKCS#11) narrows it. | Accepted for v0.1.0 (host trust); Planned (FEAT-044) |
| T-22 | T | Editing or deleting audit records directly in the database | Hash chain over every record; `levix-hub audit verify` reports the first broken link. A host root user can rewrite the whole chain; export to the customer's SIEM keeps an outside copy. | Mitigated (v0.1.0) for detection of partial edits; full rewrite Planned (FEAT-034) |
| T-23 | E | Host operator misuses `init` or `recover-owner` | Only possible with shell access to the Hub host; each use is audited and visible to every owner. | Mitigated (v0.1.0), within the host-trust assumption |
| T-24 | I | Secrets in logs | Structured logs exclude passwords, codes, tokens and secrets; a test scans the logs of the end-to-end run. | Mitigated (v0.1.0) |
| T-25 | T | Downgrade to an older Hub on a newer database, or a failed migration | The Hub refuses to start on a newer schema; migrations are additive and tested. | Mitigated (v0.1.0) |
| T-26 | I | Backups leak the database | Encrypted backup archives. | Planned (FEAT-036) |
| T-27 | D | Disk full or read-only volume leaves the Hub half-working | Readiness check of the database and key file; writes fail with an explicit error; startup refuses an unusable volume. | Mitigated (v0.1.0) |

### TB5: devices and product servers

| ID | STRIDE | Threat | Answer | Status |
|---|---|---|---|---|
| T-30 | S | A device impersonates another | Ed25519 device identity kept in the OS vault; nonce challenge signed on every connection. | Planned (FEAT-006) |
| T-31 | S | A rogue Hub impersonates the customer's Hub to devices | The enrollment code carries the Hub key fingerprint; devices accept only messages signed by the pinned key. | Planned (FEAT-006, FEAT-008) |
| T-32 | T | Tampering with policies, certificates or commands in transit | Every Hub-to-device message signed by the Hub key. | Planned (FEAT-008, FEAT-025) |
| T-33 | E | Compromise of the Hub signing key | Key custody separate from the database; rotation with re-pinning signed by the previous key; hardware custody. | Planned (FEAT-044, FEAT-045) |
| T-34 | I | A device sends content in telemetry or audit | Channel schemas refuse unknown fields; metadata-only audit. | Planned (FEAT-009, FEAT-015) |
| T-35 | D | A lost or stolen device keeps working | Kill-switch; short-lived device certificates (default 72 h offline). | Planned (FEAT-006, FEAT-025) |
| T-36 | E | A cloned golden image enrolls many devices with one code | Single-use enrollment codes; request-access queue for pre-installed images. | Planned (FEAT-006, FEAT-023) |
| T-37 | D | Connection floods from many devices | Per-identity limits; scalable profile with replicas. | Planned (FEAT-006, FEAT-002) |

### TB6: Levix artifacts

| ID | STRIDE | Threat | Answer | Status |
|---|---|---|---|---|
| T-40 | T | A forged product manifest or license | Manifests and licenses signed by Levix and verified offline against keys shipped with the Hub. | Planned (FEAT-043, FEAT-007) |
| T-41 | E | Compromise of a Levix signing key | Offline key ceremony, revocation list delivered in Hub updates and the mirror. | Planned (FEAT-046) |
| T-42 | T | A malicious dependency or a tampered build | `cargo-deny` advisories and licenses and a CycloneDX SBOM per release; signed images and reproducible builds. | Mitigated (v0.1.0) for advisories and SBOM; Planned (FEAT-039) for signatures and reproducibility |

### TB7 to TB11: integrations opened by later releases

| ID | STRIDE | Threat | Answer | Status |
|---|---|---|---|---|
| T-50 | S | Forged SSO assertions or SCIM requests | OIDC and SAML signature validation, audience and time checks; SCIM bearer token per IdP. | Planned (FEAT-004) |
| T-51 | S | Forged ITSM callbacks or webhook deliveries | HMAC-SHA256 with timestamps and replay window. | Planned (FEAT-031, FEAT-032, FEAT-033) |
| T-52 | I | ITSM and endpoint credentials leak | Encrypted at rest through key custody; delivered to devices encrypted to the device public key. | Planned (FEAT-010, FEAT-030) |
| T-53 | I | Audit events disclosed in transit to the SIEM | TLS to the SIEM; metadata only. | Planned (FEAT-034) |
| T-54 | T | A tampered update, model or pack imported into the mirror | Levix signature verification of every bundle before it is offered to devices. | Planned (FEAT-016) |
| T-55 | E | Abuse of break-glass passwords | Dedicated role, reason, optional second approver, audit of every reveal and use, rotation after use. | Planned (FEAT-011) |
| T-56 | I | Organization recovery private key exposed | Only the public key ever reaches the Hub. | Planned (FEAT-012) |

## Assurance

- Independent penetration test before the first large customer and yearly; published vulnerability disclosure policy (FEAT-041).
- This document is reviewed with every release; a new boundary or capability adds its threats before it is built.
