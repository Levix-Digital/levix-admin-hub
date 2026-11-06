# Feature Specification: Foundations v0.1.0

**Feature Branch**: `feat/001-foundations`

**Created**: 2026-11-06

**Status**: Draft

**Input**: User description: "Foundations v0.1.0: public versioned contract, Rust service skeleton with organizations, administrator accounts with roles and TOTP, web console, administration CLI, threat model" (backlog FEAT-018, FEAT-001, FEAT-019, FEAT-020, FEAT-021, FEAT-040; roadmap release v0.1.0 Foundations).

Release goal (docs/ROADMAP.md): IT installs the Hub, creates its organization and administrators, and works through the console, the CLI and a published contract.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install the Hub and claim it safely (Priority: P1)

An IT administrator starts the Hub container on a server of the company. The Hub has no default account. The administrator runs the initialization command on the server, which creates the instance and prints a single-use setup link. Opening it in a browser, the administrator becomes the instance owner: sets a password, enrolls an authenticator app (TOTP), saves the recovery codes and creates the first organization.

**Why this priority**: nothing else in the Hub can be used before it is installed and claimed, and a claim that anyone on the network could make would be the first finding of any audit.

**Independent Test**: start the container, run the initialization command, complete the setup link, sign in again with password and TOTP; the setup link no longer works.

**Acceptance Scenarios**:

1. **Given** a freshly started Hub, **When** someone opens the console, **Then** it shows only that the Hub is not initialized and how to initialize it on the server; no account can be created from the network.
2. **Given** the initialization command was run, **When** the administrator opens the setup link within its validity, **Then** they set a password, confirm a TOTP code, receive recovery codes and create the first organization.
3. **Given** the setup link was used or expired, **When** it is opened again, **Then** it is refused and the refusal is recorded in the administrative audit.
4. **Given** the Hub runs without a certificate configured, **When** it starts, **Then** it serves TLS with a generated certificate and prints its fingerprint so the administrator can verify it in the browser.

---

### User Story 2 - Manage organizations and administrators with roles and a mandatory second factor (Priority: P1)

The instance owner and organization owners invite administrators and give each one a role per organization: owner, administrator, approver, auditor or read-only. An invited administrator accepts a single-use invitation, sets a password and must enroll TOTP before doing anything else. Owners can suspend an administrator, change roles, reset a lost second factor and revoke sessions.

**Why this priority**: every later capability (devices, policies, licenses) is an administrative action that must be attributed to an authenticated person with the right role.

**Independent Test**: invite an administrator with the auditor role, accept the invitation, verify that the auditor can read but every write is refused; reset their TOTP and check that the old one no longer works.

**Acceptance Scenarios**:

1. **Given** an organization owner, **When** they invite a person with a role, **Then** the Hub issues a single-use invitation valid for 72 hours, shown once, to be delivered through a separate channel.
2. **Given** an accepted invitation, **When** the new administrator signs in for the first time, **Then** no action is allowed until a TOTP code is confirmed.
3. **Given** an administrator with the read-only or auditor role, **When** they attempt any change, **Then** it is refused with a message naming the missing permission.
4. **Given** the last owner of an organization, **When** someone tries to remove or demote them, **Then** it is refused.
5. **Given** repeated wrong passwords or TOTP codes, **When** the limit is reached, **Then** sign-in for that account is delayed with increasing waits and the attempts are audited.
6. **Given** an administrator who lost their authenticator, **When** they use a recovery code, **Then** they sign in once and must enroll a new TOTP; the code cannot be reused.
7. **Given** the instance owner lost every factor, **When** an operator with shell access to the Hub host runs the local recovery command, **Then** a new single-use setup link for that account is printed and the event is audited.

---

### User Story 3 - Work from the web console (Priority: P1)

Administrators sign in to the console served by the Hub itself, in English, Canadian French or Brazilian Portuguese, and manage organizations, administrators, their own profile, second factor, sessions and API tokens, and read the administrative audit.

**Why this priority**: the console is how most IT teams will run the Hub; the success metric of the product (IT deploys alone in an afternoon) depends on it.

**Independent Test**: complete stories 1 and 2 using only the console in each language, with the keyboard only, in an air-gapped network.

**Acceptance Scenarios**:

1. **Given** a network without internet, **When** the console loads, **Then** every asset comes from the Hub binary; no external request is made.
2. **Given** an administrator idle beyond the inactivity limit, **When** they act again, **Then** they must sign in again.
3. **Given** the browser language is French (Canada) or Portuguese (Brazil), **When** the console opens, **Then** it is shown in that language, and the administrator can switch language in their profile.
4. **Given** an automated accessibility scan of every page, **When** it runs in CI, **Then** it reports no WCAG 2.1 AA violation.

---

### User Story 4 - Automate with the CLI and the public API (Priority: P2)

An administrator installs the CLI on their workstation, signs in with password and TOTP, and runs every administrative action of the console from scripts. For unattended automation they create a personal API token with an expiry, limited to their own permissions.

**Why this priority**: IT teams automate onboarding; the CLI also proves that the public API covers every action, which the contract promises.

**Independent Test**: repeat stories 1 to 3 from the CLI only (except the browser-based setup link, which the CLI can also complete), with `--output json` parsed by a script.

**Acceptance Scenarios**:

1. **Given** the CLI and the Hub fingerprint, **When** the administrator signs in, **Then** the CLI verifies the certificate (CA or pinned fingerprint), asks for password and TOTP and stores the session in the user's protected configuration.
2. **Given** a personal API token, **When** a script uses it, **Then** it acts with at most the creator's permissions, until its expiry or revocation.
3. **Given** any console action, **When** looked up in the CLI help, **Then** an equivalent command exists.
4. **Given** a command fails, **When** run with `--output json`, **Then** the error is a stable machine-readable object and the exit code is non-zero.

---

### User Story 5 - Read the published contract and threat model (Priority: P2)

An integrator or an auditor of a customer reads the public contract (REST API and device channel specification) and the threat model, both versioned with the Hub release.

**Why this priority**: the strategy requires a public versioned contract and a threat model from the first release; regulated buyers ask for them before any pilot.

**Independent Test**: from the release artifacts only, generate a client from the OpenAPI document and call the API; read the threat model and find, for each trust boundary, its threats and mitigations.

**Acceptance Scenarios**:

1. **Given** the Hub source, **When** CI runs, **Then** the published OpenAPI document matches what the service serves and an incompatible change without a major version is refused.
2. **Given** the channel specification, **When** a product team reads it, **Then** it defines the versioning, the message envelope, the device authentication handshake and the signature format, so products can start their client before enrollment ships in v0.2.0.
3. **Given** the threat model, **When** an auditor reads it, **Then** every threat has a status (mitigated in this release, planned with a backlog id, or accepted with a reason).

### Edge Cases

- Two setup or invitation links used at the same time for the same account: only the first succeeds.
- Clock drift between the administrator's phone and the Hub: TOTP accepts one step before and after; larger drift is refused with a hint.
- The SQLite file is on a read-only or full volume: the Hub refuses to start, or refuses writes, with an explicit error; it never runs half-working.
- An administrator belongs to several organizations with different roles: each action is checked against the role in the organization it targets.
- Deleting an organization that still has administrators or (later) devices: v0.1.0 only allows archiving it; archived organizations are read-only.
- A request with unknown fields: refused (contract rule of zero knowledge, ADR-0004).
- Upgrading from a later database schema (downgrade): the Hub refuses to start and names the schema version.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Hub MUST start with no default credentials and accept the first account only through a single-use setup link created by a command run on the Hub host (valid 24 hours).
- **FR-002**: The Hub MUST support one or more organizations per instance; every organization-scoped record carries its organization, and an administrator sees only the organizations where they have a role.
- **FR-003**: The Hub MUST provide the roles organization owner, administrator, approver, auditor and read-only per organization, plus the instance owner for instance-level settings and creating organizations; permissions are a fixed matrix documented in the contract.
- **FR-004**: Local administrator accounts MUST store passwords only as Argon2id hashes, require a minimum length of 12 characters and reject passwords from a bundled list of common passwords.
- **FR-005**: Every administrator MUST enroll TOTP (RFC 6238, SHA-1, 6 digits, 30 s) before any action; the Hub MUST issue 10 single-use recovery codes, stored hashed.
- **FR-006**: Sessions MUST expire after 15 minutes of inactivity and 12 hours absolute (both configurable within limits); owners can revoke an administrator's sessions.
- **FR-007**: Administrators MUST be able to create personal API tokens with a name and an expiry of at most 90 days, shown once and stored hashed, acting with at most the creator's current permissions.
- **FR-008**: Every administrative action and every authentication event MUST be written to an append-only, hash-chained administrative audit with actor, organization, action, target, time, source address and result; owners and auditors can read it. Device events, retention, search and SIEM export come with FEAT-009 and FEAT-034.
- **FR-009**: The console MUST be served by the Hub with every asset embedded, work without internet, conform to WCAG 2.1 AA and be available in English, Canadian French and Brazilian Portuguese.
- **FR-010**: The CLI MUST offer every administrative action of the console over the public API, with human and JSON output and stable exit codes.
- **FR-011**: The REST API MUST be described by an OpenAPI 3.1 document versioned with the release and checked in CI against the running service; it MUST reject unknown fields.
- **FR-012**: The device channel MUST be described by a versioned message specification (envelope, versioning, handshake, signatures); the endpoint itself ships with enrollment in v0.2.0.
- **FR-013**: The Hub MUST serve HTTPS (TLS 1.2 or later) with the customer's certificate or a generated one whose fingerprint it prints; plain HTTP only when explicitly configured behind a TLS-terminating proxy.
- **FR-014**: The Hub MUST make no outbound network connection.
- **FR-015**: The repository MUST contain a threat model and security architecture document covering every trust boundary of the product (including those of later releases), with the status of each threat.
- **FR-016**: The Hub MUST expose liveness and readiness endpoints and structured JSON logs without secrets or personal data beyond administrator identifiers.

### Key Entities

- **Instance**: the Hub installation; its initialization state, instance settings and the instance owner.
- **Organization**: a customer organization governed by the Hub; name, slug, status (active, archived).
- **Administrator**: a person who administers the Hub; name, e-mail, status (invited, active, suspended), password hash, TOTP secret (encrypted), recovery codes (hashed).
- **Role assignment**: an administrator's role in one organization.
- **Invitation / setup link**: single-use, expiring token that lets a person set their credentials.
- **Session**: an authenticated console or CLI session with inactivity and absolute expiry.
- **API token**: a personal, expiring credential for automation.
- **Audit record**: one administrative or authentication event, chained to the previous record by hash.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An IT administrator who never used the Hub starts it, claims it, creates an organization and invites a second administrator in under 30 minutes, following the README only.
- **SC-002**: The container starts and is ready in under 10 seconds on 2 vCPU and 4 GB of RAM.
- **SC-003**: Interactive console and API operations answer in under 300 ms at the 95th percentile on that machine.
- **SC-004**: Every action of the console has an equivalent CLI command and API operation (checked by a test that walks the permission matrix).
- **SC-005**: Zero WCAG 2.1 AA violations reported by the automated scan on every console page, in the three languages.
- **SC-006**: Every threat in the threat model has a status; none is left without one.

## Assumptions

- SSO for administrators (OIDC, SAML) arrives with FEAT-004 in v0.4.0; v0.1.0 has local administrator accounts only.
- Product-scoped roles arrive with product manifests (FEAT-043, v0.2.0); v0.1.0 roles are per organization.
- The compact profile is the only one delivered in v0.1.0 (SQLite); the PostgreSQL adapter and the scalable profile come with FEAT-002 in v0.3.0, and the SQL is kept portable from the start.
- No `levix-platform` crate is needed yet: the first shared crates (channel messages, signatures, policy model) are pinned when enrollment and policies are built (v0.2.0, v0.3.0).
- E-mail delivery of invitations is out of scope (no outbound connections); links are shown once to the inviting owner.
