# Levix Admin Hub

The control plane that a company's IT hosts inside its own perimeter to govern every Levix
product installation (users, devices, licenses, policies, updates and audit) without ever
seeing content. It is what turns separate product installations into one governed account:
one Hub, one contract, one renewal, one person in IT.

## Problem

Organizations whose data cannot leave the perimeter (legal and accounting offices, clinics
and health networks, financial institutions, public sector, defense, critical
infrastructure, logistics with fiscal data) want to use the Levix products, but their IT and
security teams cannot accept unmanaged installations:

- **No central control.** Each installation is configured by hand on each machine; IT cannot
  say who uses what, on which device, under which rules, or turn an installation off when a
  laptop is lost or an employee leaves.
- **Cloud consoles are not an option.** The usual answer of vendors is a SaaS admin console,
  which requires the installation to call home and the company to trust the vendor with
  identities, policies and telemetry. For these customers that is exactly what the security
  policy forbids.
- **Air-gapped fleets cannot update.** Machines without internet need updates, models and
  domain packs that IT downloads once, verifies and releases when it decides, on a quarterly
  rhythm after homologation, not weekly.
- **Compliance needs evidence.** Auditors ask who approved which device, which policy was in
  force and what was blocked, and they want it without any content being collected.

Today these companies either forbid the products or accept manual, unaudited installations.

## Target Users

- **Primary: the IT or security administrator** of a customer organization. Buyer and user are
  the same person. They do not want features, they want to sleep well: reduced risk, clear
  evidence, and a tool they can install and run alone. Profiles range from the single
  administrator of an office or clinic with 5 to 50 machines (compact profile, no
  infrastructure team) to the platform team of a large company or public body with thousands
  of machines (scalable profile, Kubernetes).
- **Approvers and auditors:** managers and security officers who approve access requests
  (directly or through the company's ITSM), and auditors who read the metadata-only audit
  trail.
- **End users of the products** never use the Hub directly: they activate their account on an
  enrolled device, see the policy in force read-only, and request access or administrative
  actions that the Hub turns into approvals.
- **Levix and its implementation partners** install and support it, but never operate it for
  the customer and never see its data.

## Value Proposition

"Govern every Levix product in your perimeter, without Levix seeing anything." The five
arguments, in the order IT asks:

1. **Zero knowledge:** the Hub never sees documents, chunks, vectors, prompts or
   conversations. It governs installations, not data.
2. **Hosted by you:** VPC or on-premises, never a Levix SaaS. No call home.
3. **Verified offline:** licenses signed by Levix and policies signed by the Hub are verified
   without network access; devices pin the Hub key at enrollment.
4. **Auditable:** metadata of everything, content of nothing; logs exportable to the
   company's SIEM; a public, versioned contract that auditors and integrators can read.
5. **Contained:** in corporate use, customer data and the products stay strictly inside the
   customer's perimeter, enforced by mechanisms, not by convention: only destinations inside
   the perimeter or explicitly approved are reachable, with evidence per device (ADR-0009).

Plus the enterprise features IT expects from day one: SSO and SCIM with the company's
identity provider (or the Hub's own directory when there is none), approvals through
ServiceNow, Jira or any tool via webhooks, a per-device break-glass password with rotation,
and an update mirror for air-gapped fleets.

## Business Model

- **Never free.** Whoever needs the Hub is an organization; the free editions of the products
  work without it.
- **Annual license per Hub instance**, signed by Levix and verified offline, which carries the
  licenses of each governed product. One account, one invoice, one renewal.
- **Hub required above five machines:** paid use on up to five machines works without a Hub, through a Levix-signed license file imported on the device and verified offline; above five, the Hub carries the licenses.
- **Product licenses per seat** (named user) by default; **per device** as an option for shop
  floors and OT where there is no named user. Never per usage, token or document.
- **Price reference, to validate with pilots:** the Hub below the yearly cost of a junior
  analyst; each extra product in an account is the cheapest sale there is.
- **Services:** implementation, training and customer-specific integrations, priced as
  projects, with the result returning to the product when it fits.
- **Main costs:** security of the Hub itself (independent penetration test before the first
  large customer and yearly after, disclosure process, certification later) and support of
  heterogeneous on-premises environments, reduced by the compact profile and built-in
  diagnostics.

## Success Metrics

- **Time to deploy without Levix help:** IT installs the compact profile and enrolls the first
  device alone in one afternoon (target: under 4 hours, following the published guide).
- **Products per Hub:** average number of Levix products governed per installed Hub (target:
  2 or more within a year of installation).
- **Seats governed:** devices and users under an active Hub license (growth per quarter).
- **Security posture:** zero critical or high findings open from the independent penetration
  test at each release; vulnerability reports answered within the published deadline.
- **Renewal:** share of Hub licenses renewed at the end of the year (target: 90% or more).

## Scope

The complete product, as capabilities:

1. **Product registry:** any product, current or future, Levix or third party, is added to a
   Hub as data, through a product manifest signed by Levix (identifier, client kinds, policy
   schema, data definition formats, seat model, telemetry counters, commands); the license of
   the contract lists the products the Hub governs. Adding a product never needs a new Hub
   release. AnyContext, DocMapper, Gedo, the Dev Cycle Protocol and Aurora (gate policies and
   approval audit) are the first manifests.
2. **Organizations and administration:** organizations, administrators and roles, a web
   console served by the Hub, an administration CLI and a guided installation.
3. **Identity:** the Hub's own directory of users and groups, single-use activation codes,
   SSO with the company's identity provider and SCIM provisioning; accounts never shared
   across products.
4. **Devices and enrollment:** enrollment by a code that pins the Hub key, a request-access
   queue for devices pre-installed by the company's device management tools, device identity,
   outbound-only channel with heartbeat, fleet inventory, short-lived device certificates and
   kill-switch.
5. **Licensing and seats:** Levix licenses per product verified offline, seats per user or per
   device, seat enforcement with a hardware replacement flow.
6. **Policies:** per-product policy schemas declared in the product manifests; editing, versioning,
   signing and distribution, with a company-set offline validity; quotas; secure
   distribution of corporate model endpoints and keys to devices; network containment: products
   reach only destinations inside the customer's perimeter or explicitly approved, with
   compliance evidence per device and an allowlist export for the customer's network (ADR-0009).
7. **Approvals and ITSM:** `sudo` and access requests become approvals in the Hub; generic
   webhooks to approve and revoke; ServiceNow and Jira adapters.
8. **Break-glass and recovery:** per-device root password generated, rotated and revealed
   with access logging (LAPS model); distribution of the organization recovery public key.
9. **Audit and telemetry:** metadata-only audit of the Hub and of devices, governed MCP usage
   audit, source telemetry at the level the policy sets, export to the company's SIEM.
10. **Sharing:** groups and curators, organization data units, distribution of product data
    definitions as metadata only.
11. **Update mirror:** product updates, models, domain packs and bundled components downloaded
    once, verified by the Levix signature and released to the fleet when IT decides.
12. **Product servers:** enrollment and governance of product servers in the company VPC
    (AnyContext Server, Model Server) like devices.
13. **Packaging and operations:** compact profile (one container, embedded database) and
    scalable profile (Postgres, Redis, replicas, Helm) from the same code; backup and
    restore; built-in diagnostics.
14. **Contract:** a public, versioned API and WebSocket protocol through which any product, in
    any language, talks to the Hub.

## Non-Goals

- **No Levix-hosted SaaS** and no call home: the Hub always runs in the customer's perimeter.
- **No content:** the Hub never receives, stores or processes documents, chunks, vectors,
  prompts, conversations or file contents.
- **No passwords or user keys:** passwords and user keys are created on the device and never
  go through the Hub; the organization recovery private key never lives in the Hub.
- **No general device management:** the Hub governs Levix product installations, not the
  operating system (it works alongside Intune, Jamf or SCCM, it does not replace them).
- **No product features:** what a product does (extraction, search, chat) stays in the product;
  the Hub only knows each product through its signed manifest; no product-specific code lives
  in the Hub.
- **No free edition.**

## Risks and Assumptions

| Risk or assumption | Why it matters | How it is reduced |
|---|---|---|
| A bug in identity, keys or break-glass becomes an incident at the customer | It is the most dangerous piece to build and it carries the security brand | Small, audited core; break-glass built last on a mature key and audit core; independent penetration test before the first large customer; published disclosure process; public, reviewed contract |
| Building SSO/SCIM, ITSM and break-glass before the first sale delays the first sellable version | Strategy assumed a smaller first scope and one person with tooling | Clear build order in the roadmap; generic webhooks before specific ITSM adapters; product-neutral contract so each piece is built once |
| On-premises support is expensive | Heterogeneous environments consume a small team | Standardized compact profile, built-in diagnostics and exportable logs, deployment guide IT follows alone, partners for first-level support |
| Assumption: products adopt the shared crates of `levix-platform` | Client and server must share the policy model, signatures and formats | Crates built in `levix-platform` as the products need them; the Hub depends on them by version |
| Assumption: the first paid pilots convert into contracts that need the Hub | The Hub has no entry door of its own | Hub required above five machines in every contract, with a Levix-signed license file on the device below that (decided 2026-11-07); DocMapper and AnyContext as entry products |
| Customers want to audit the code | Regulated buyers do not trust what they cannot read | Proprietary license with source access for customers: under NDA, each customer with an active contract receives the exact source of the versions it runs to read, build and analyze, never to redistribute, modify for production or compete; reproducible builds to check the delivered binary; optional third-party source escrow for large and public-sector customers; signed builds and SBOM |

## Open Questions

- Exact price points: after the first pilots.
