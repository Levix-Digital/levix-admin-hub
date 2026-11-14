---
id: 20261114-142637-aurora-dev-suite-nested-organizations-policy-ceilings-and
kind: proposal
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-14T14:26:37.963Z
subject: "Nested organizations, policy ceilings and seat types for Aurora's editions"
status: open
reply_to: 
deferred: 
thread: 20261114-142637-aurora-dev-suite-nested-organizations-policy-ceilings-and
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Say whether Hub organizations can be nested (or will be), and whether the Aurora policy schema can separate ceilings set by the organization from defaults chosen by a project.

### Context
On 2026-11-14 the user decided Aurora's positioning and editions (Aurora ADR-0018; aurora-dev-suite branch levix/aurora-suite-positioning at 2667530): Aurora names the whole suite, sold as Community, Team, Enterprise (Aurora with the Hub) and Sovereign (Enterprise in networks with no internet), with a license per seat. The analysis behind it found that large buyers (holdings, multinationals, consultancies with one environment per client project) expect one contract and identity above several operating boundaries, each with its own members, approvers, policies and model endpoints. Aurora also adds a policy view per project (Aurora FEAT-064) and per-team metrics by default (Aurora FEAT-060).

### Proposal
1. Nested organizations: one account (license, identity configuration, central audit) above organizations that can contain others (region, business unit, subsidiary, client project), each with its own administrators, groups, policies and product servers. If the Hub already covers this another way (several organizations in one Hub, groups), say how.
2. Ceilings and defaults in product policy schemas: a field marked as a ceiling is set at the organization and lower levels (a nested organization, a project) can only tighten it; other fields are defaults a project may choose inside the ceilings. Aurora's first ceilings: projects that require approval, allowed coding agents and model endpoints, and the most an agent may do in a project without a person.
3. Seat types in the Aurora manifest: contributor seats and approver or reader seats counted separately under the Hub license.

### Constraints
The Hub stays zero-knowledge and carries no product code (Hub ADR-0004, ADR-0008); everything above is metadata and manifest data. Aurora grants no access of its own.

### Closes when
The Admin Hub answers each point with what exists, a backlog item and release, or a counterproposal.

### Authority
The projects settle the contract shape. New Hub epics or changes to its roadmap order go to the user.
