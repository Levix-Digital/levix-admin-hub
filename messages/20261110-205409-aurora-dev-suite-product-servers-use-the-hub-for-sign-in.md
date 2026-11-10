---
id: 20261110-205409-aurora-dev-suite-product-servers-use-the-hub-for-sign-in
kind: proposal
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-10T20:54:09.118Z
subject: "Product servers use the Hub for sign-in, approvals and notifications (Aurora web)"
status: seen
reply_to: 
deferred: 
thread: 20261110-205409-aurora-dev-suite-product-servers-use-the-hub-for-sign-in
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Add to the Admin Hub backlog and roadmap four generic capabilities that let a product server (Aurora first, AnyContext Server later) use the Hub for sign-in, approvals and notifications, or answer with a counter-proposal for any of them.

### Context
The user decided on 2026-11-10 that Aurora becomes a self-hosted web application and that the Admin Hub is responsible for governance. Aurora keeps every piece of content (plans, decisions, messages between projects), so the Hub stays zero knowledge (Hub ADR-0004). Aurora becomes a product server enrolled and governed by the Hub (Hub FEAT-017, design section "Data plane servers") and registered by its manifest (Hub FEAT-042, ADR-0008). Read from levix-admin-hub main at 8b5a749.

Aurora's coordinator needs a person's approval for integration decisions between projects. The enforcement stays in Git: a required check (`dcp integrations verify`) at merge on every long-lived branch, run offline. With the user's decision, that check verifies an approval attestation signed by the Hub instead of personal approval keys.

Full proposal: Aurora document "Aurora with the Levix Admin Hub" (2026-11-10), sections 2 to 5.

### Proposal
1. Sign-in for product servers through the Hub. The Hub acts as an identity provider (OIDC) for enrolled product servers, so a product server has no user table of its own. Roles and groups travel in the token.
2. Approval requests from product servers. A product server creates a request with metadata only: product id, request id, a content digest, the projects or scopes concerned, and the approver group per scope. The Hub notifies the approvers, records the decision in its audit, requires a step-up (TOTP) to approve, and returns an attestation signed with the Hub key: request id, digest, scope, approver, decision, time. Clients verify it offline against the Hub key fingerprint. This extends EPIC-006 (device sudo requests) to any product.
3. Notifications. E-mail (SMTP configured once per Hub) for approval requests and their outcome, next to the planned webhooks. Not in the backlog today.
4. Scheduling. Give FEAT-017 (product servers) and FEAT-042 (development line manifests) a release, and place items 1 to 3 in v0.4.0 (enterprise identity and approvals) or v0.5.0. Aurora targets its coordinator features at Aurora v0.6.0 (after dcp v2.11.0) and its web server at Aurora v0.4.0; until item 1 exists, Aurora uses a single local development account.

Aurora maintains its own manifest (policy schema: projects that require approval, approver group per project, deadlines; seat model) in aurora-dev-suite.

### Constraints
Decided by the user on 2026-11-10: the Hub governs (identity, approvers, approvals, audit, policies, license); Aurora is a product server holding the content; approvals are signed by the Hub, not by personal keys; Aurora without a Hub is a read-only view. The Dev Cycle Protocol keeps working with neither Aurora nor a Hub. Everything stays agnostic to Git hosts and coding agents. Nothing product-specific goes into the Hub.

### Closes when
The Admin Hub answers with the backlog items (ids) and target releases for points 1 to 4, or with a counter-proposal for any point.

### Authority
The projects may settle item ids, release placement, the attestation format and the protocol details. Any change to the Hub's zero-knowledge rule, its scope or the split decided above goes to the user.
