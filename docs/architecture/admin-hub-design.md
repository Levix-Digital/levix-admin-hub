# Levix Admin Hub — design

The control plane of the Levix products (AnyContext, DocMapper and future ones). This is the
Admin Hub's slice of the design decided in the AnyContext v2 redesign (October 2026). The
client side of the channel, the policy model and signature verification live in
`levix-platform`; the products keep their own content and data.

## Principles

- **Always hosted by the customer** (VPC or on-premises) — never a Levix SaaS. (The initial
  Admin Hub specifications listed "Levix SaaS" as a hosting option; that is corrected.)
- **Zero knowledge:** the Admin Hub never sees content (documents, chunks, vectors,
  prompts, conversations). It governs installations, not data. Content never leaves the
  company perimeter.
- **Several products, one Admin Hub:** a company with AnyContext and DocMapper installs a
  single Admin Hub.
- **Accounts are never shared across products:** the Admin Hub manages each product's
  accounts with the same tools, but users of different products never mix.
- **Rust**, sharing crates with the clients (`levix-platform`): the policy model, signature
  verification and the product data definition formats are identical on both sides. (The
  initial specification planned Python/FastAPI; the products removed Python.)

## Common core

- **Organizations.**
- **Users and groups:** the Admin Hub's own directory (for customers without an identity
  provider: offices, clinics, smaller municipalities), **SSO** with the company's identity
  provider, and **SCIM** provisioning.
- **Activation of directory users:** the admin registers a user (name, login, roles); the
  Admin Hub generates a **single-use activation code** with a short validity (e.g. 72 h),
  delivered through a separate channel. The password and the keys are created on the
  device; **the password never goes through the Admin Hub**. A forgotten password needs a
  new activation code.
- **Devices and enrollment:** the enrollment code carries the Admin Hub key fingerprint,
  which the device pins; each device has an Ed25519 identity. Outbound-only WebSocket from
  the device, heartbeat and **kill-switch**.
- **License and seats per product:** a license signed by Levix (seats, validity), verified
  **offline** by the Admin Hub. Seats per device or per user is an open commercial decision;
  the architecture supports both.
- **Policy distribution and signing:** each product registers its **policy schema** (for
  AnyContext: models and trust levels, web search, MCP server, connectors and account
  domains, sources, quotas…); the core signs, distributes and audits without knowing the
  details. The offline validity of a policy is set by the company.
- **Audit:** metadata only (rule, when, device), never content.
- **ITSM:** `sudo` requests from devices become approval requests and, optionally,
  ServiceNow or Jira tickets.
- **Break-glass:** a unique per-device `root` password, generated and rotated by the Admin
  Hub (LAPS model), visible only to authorized administrators with access logging; every
  use is reported and followed by rotation.
- **Organization recovery key** (policy option): the Admin Hub only distributes the
  organization public key; the private part stays offline under the company's control.

## Product data definitions and sharing

- **Groups and curators:** sharing with groups from SSO or the directory; in organization
  data units (in AnyContext, organization workspaces) the admin is the owner and appoints
  curators from a department as editors.
- **Definitions, never content:** the Admin Hub distributes product data definitions (in
  AnyContext: workspace name, sources, filters, domain pack, prompt) as metadata only.
- **Source telemetry** at the level the policy sets: counts only; anonymized identifiers (a
  hash that says "same source" without the name) as the default; or full paths when the
  company needs to audit and informs users.

## Update mirror

The Admin Hub distributes product updates, models, domain packs and bundled components
(e.g. Chromium) to devices without internet: IT downloads once, verified by the Levix
signature, and controls when each version reaches the fleet.

## Data plane servers

Product servers in the company VPC (e.g. AnyContext Server, the shared index of deployment
mode 3, and the Model Server of mode 2) are enrolled and governed by the Admin Hub like a
device.

## Packaging

- **Installable package:** containers for a single server; Helm for Kubernetes.
- **Two installation profiles, same code:** **compact** (one container with an embedded
  database — offices, clinics) and **scalable** (Postgres, Redis, replicas — large
  companies).

## Contract

A documented API and WebSocket protocol: any product, in any language, talks to the Admin
Hub through it. What goes through the channel: identities (SSO, directory, SCIM) and
activation; signed policies; product data definitions and organization data units; groups
and curators; break-glass passwords; `sudo` requests and ITSM tickets; metadata-only audit;
quotas; the update mirror; server enrollment.
