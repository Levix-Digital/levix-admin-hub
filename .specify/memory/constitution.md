<!-- dev-cycle-constitution
generated-by: dev-cycle-protocol/constitution.py (do not edit; change a source and regenerate)
sources-sha256: 0521b9870597b926365c226123ba227ca93406040ed1190b003897e88b5b6e8f
content-sha256: 7565bca3fcb4c34d82b618ca08e464049dd7a214f0b8cfad23ab607c654c8db1
-->
# levix-admin-hub Constitution

The non-negotiable rules every specification, plan and task of this project follows. It is generated from the project's sources of truth by `dev-cycle-protocol/constitution.py`; change a source and regenerate it, never edit it.

## Core Principles

### I. Design

*Source: software-design*

#### Modularity and separation of responsibilities
- One module, type or function has **one reason to change**. When a description needs "and", split it.
- Group code by what it is about (a domain concept or a feature), not by what kind of file it is, unless the architecture style says otherwise.
- Business rules never live in the user interface, in transport code (HTTP handlers, CLI argument parsing) or in persistence code.
- Dependencies point toward stable code: details (frameworks, databases, network, UI) depend on the core, never the other way around.

#### SOLID, in practice
| Principle | What it means when writing code | Smell that it is violated |
|---|---|---|
| **S**ingle responsibility | Each unit does one job and can be named without "and". | A 600-line file mixing parsing, rules and output. |
| **O**pen/closed | Add behavior by adding a new implementation, not by growing an `if/else` chain in stable code. | Every new provider edits the same `match` in five places. |
| **L**iskov substitution | Any implementation of an interface can replace another without surprises. | An implementation that panics or ignores half the contract. |
| **I**nterface segregation | Small interfaces shaped by what the caller needs. | Callers implementing methods they never use. |
| **D**ependency inversion | High-level code depends on abstractions it owns; adapters implement them. | Core code constructing an HTTP client or a database driver. |

#### Clean Code
- **Names** say what something is or does; no abbreviations a newcomer cannot decode; no `data`, `info`, `manager`, `util` as whole names.
- **Small functions** that do one thing at one level of abstraction; early returns instead of deep nesting.
- **No magic values**: named constants with units in the name (`TIMEOUT_MS`).
- **Comments explain why**, never what; delete commented-out code and dead code.
- **Consistency over preference**: follow the conventions already in the codebase (naming, error types, layout) unless the change is an agreed refactor.
- **Duplication**: remove it when the same rule exists twice; tolerate it when two pieces only look alike today.

#### Explicit error handling
- Never swallow an error (`except: pass`, `let _ =` on a meaningful result, an empty `catch`, a blind `.unwrap()` on input that can fail).
- Errors carry context (what was being done, with which input) and use the project's canonical error types.
- Decide at each boundary: recover, translate into the caller's error type, or propagate. Report to the user once, at the edge.
- Fallbacks are allowed only when they are explicit, logged and visible in status output; a silent fallback that changes behavior is a defect.

#### Testability
- Pure logic is separated from I/O so it can be unit-tested without mocks.
- External services sit behind an interface the tests can replace.
- A structure that is hard to test is a design problem, not a testing problem.

### II. Simplicity First

*Source: software-design*

Principles applied mechanically produce over-engineering. The rule: **the simplest solution that keeps responsibilities separate.**

- Introduce an abstraction (trait, interface, base class, factory) only when at least one of these is true:
  1. there is a **second implementation** now (not "maybe later");
  2. it creates a **test seam** you need today;
  3. it isolates an **external dependency** (framework, service, database, OS API).
- No speculative configuration, plugin systems or generic frameworks for a single use case.
- Prefer composition over inheritance; prefer plain data and functions over class hierarchies.
- Three similar lines are better than a premature helper with five parameters.

#### Design patterns: only when they solve a real problem
| Problem actually present | Pattern that fits |
|---|---|
| Several interchangeable algorithms or providers chosen at runtime | Strategy |
| A third-party or legacy interface that does not match yours | Adapter |
| Construction is complex or depends on configuration | Factory / Builder |
| Behavior changes with an explicit lifecycle state | State machine |
| Actions must be queued, logged, undone or dispatched uniformly | Command |
| Persistence must be swappable or mocked | Repository |
| Several parts react to an event without knowing each other | Observer / event channel |

If you cannot point to the problem in the left column, do not use the pattern. Name the pattern in code or docs only when it helps the reader.

### III. Architecture Style: hexagonal

*Source: software-design/styles/hexagonal.md*

##### Structure

- **Core (domain + application services)**: entities, business rules, use cases and the **ports** they need (interfaces/traits for storage, providers, clocks, file systems). No framework, UI, transport or driver code.
- **Driving adapters** (input): CLI, TUI, HTTP handlers, RPC, MCP servers. They translate requests into core calls and core results into output.
- **Driven adapters** (output): database repositories, HTTP clients, model providers, file system access. They implement the core's ports.
- **Composition root**: one place per executable that builds the adapters and wires them into the core.

##### Dependency rule

Adapters depend on the core; the core depends on nothing outside itself except plain libraries (serialization, math, date handling). List the adapter and framework modules in `architecture.forbidden_core_imports` so `check_design` enforces the rule.

##### Where things go

| Piece | Location |
|---|---|
| A business rule or validation | Core service or entity |
| A new user command | Core command/use case, then one thin handler per surface |
| Database schema or query | Driven adapter implementing a core repository port |
| A new LLM, payment or storage provider | Driven adapter implementing an existing port |
| Output formatting, colors, layout | Driving adapter |

##### Common mistakes

- Business logic in a CLI handler or TUI event loop "because it was quicker".
- The core constructing its own HTTP client, database connection or UI widget.
- Ports shaped like the first adapter's API (leaking driver types into the core).
- A port for something with one implementation and no test need (see "Simplicity First").

### IV. Development Cycle

*Source: dev-cycle-protocol*

1. **Design**: modular code with clear responsibilities, SOLID, Clean Code, explicit errors, patterns only when needed, in the project's architecture style (`software-design`).
2. **Evidence**: every change is proven by automated tests; manual tests cover only what cannot be automated, in one curated suite (`quality-assurance`).
3. **Documentation describes today**: technical docs and README are edited in place; history lives in Git and release notes (`architecture-docs`, `user-docs`).
4. **Plans are shared and complete**: approved plans are versioned files in the project (`plan-memory`).
5. **Safe delivery**: short-lived branches on the project's branching model, CI on every pull request, release candidates, the user's release gate (`devops-release`).
6. **Security**: no secrets in Git, backward-compatible data migrations, a rollback path for every release.
7. **Bugs first**: open `critical`, `high` and `medium` bugs are planned and fixed before new features, improvements and adjustments; a `low` bug goes into the next release at the latest; no known bug stays untriaged (a severity, and a release for the ones that do not block). Only the user can explicitly accept the risk. Nothing found is lost, it is fixed now or captured in the backlog.
8. **Follow-through**: the agent follows CI and releases to the end and fixes what fails; the user's decisions (plan approval, candidate acceptance, the release gate) are never taken by the agent.
9. **No AI attribution, anywhere**: no artifact names an AI, a model or a coding agent as author, co-author, reviewer or source: commits and their trailers, tags, branches, pull and merge requests, reviews and review comments, issues, plans, documentation, release notes and code comments. Everything is authored by the repository's Git identity (`devops-release`). Configuration a project needs for its tools (an agent's settings folder, an integration name) is technical, not attribution. Text posted through any tool other than `pr_sync` (a platform tool, a connector, the web page) is checked first with `"<skills_root>/dev-cycle-protocol/bin/dev-cycle" dev-cycle-protocol _no_ai --file <text>`, and read back after posting: a footer, link or marker the tool appended ("Generated by ...", an agent's session or thread link) is edited out at once. Gate 8 also refuses commits whose author or committer is an AI identity.

## Product Requirements and Constraints

*Source: docs/product/TECH_SPEC.md*

### Non-Functional Requirements

#### Performance

- **Compact profile:** governs 500 devices with the default heartbeat on 2 vCPU and 4 GB of RAM;
  the container starts in under 10 s.
- **Scalable profile:** governs 50,000 connected devices across replicas, adding capacity by
  adding replicas.
- **Policy propagation:** a published policy reaches 95% of connected devices in under 10 s.
- **Kill-switch:** a revocation reaches a connected device in under 5 s (p95).
- **Console and API:** p95 response time under 300 ms for interactive operations at the profile's
  rated fleet size.
- **Heartbeat:** default interval 30 s, configurable by policy.

#### Security and Privacy

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

#### Availability and Recovery

- Devices keep working offline within the policy's offline validity, so a Hub outage never
  stops users immediately.
- **Compact:** one instance; `backup` and `restore` commands of the CLI produce and restore a
  consistent encrypted archive; default recovery point 24 h, recovery time under 1 h.
- **Scalable:** stateless replicas behind a load balancer; availability target 99.9% given a
  highly available PostgreSQL and Redis provided by the customer.
- Upgrades apply additive migrations; a version can be rolled back to the previous minor
  release.

#### Accessibility and Localization

- Console conforms to WCAG 2.1 level AA.
- Console languages: English, Canadian French and Brazilian Portuguese; dates and times in the
  administrator's time zone; API and contract in English.

### Constraints

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

## Architecture Decisions

Accepted decisions; a plan that contradicts one needs a new ADR that supersedes it.

- ADR-0001: Hexagonal architecture with a modular core (`docs/adr/0001-hexagonal-architecture-with-a-modular-core.md`)
- ADR-0007: Proprietary license with source access for customers (`docs/adr/0007-proprietary-license-with-source-access-for-customers.md`)

## Governance

- This constitution supersedes other practices for specifications, plans and tasks. A plan passes the Constitution Check or records each justified violation in its Complexity Tracking table; a lasting exception becomes an ADR.
- Amendments change the source, not this file: an ADR for decisions, the technical specification for requirements and constraints, `.dev-cycle.json` for the architecture style, the skills for principles. Then run `constitution.py generate` and commit the result with the change.
- Gate 3 of the development cycle blocks a plan while this file is stale or edited by hand.

**Version**: 1.0.0 | **Ratified**: 2026-11-06 | **Last Amended**: 2026-11-06
