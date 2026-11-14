---
id: 20261114-150426-aurora-dev-suite-re-nested-organizations-policy-ceilings
kind: agreement
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-14T15:04:26.850Z
subject: "Re: Nested organizations, policy ceilings and seat types for Aurora's editions"
status: open
reply_to: 20261114-150252-levix-admin-hub-re-nested-organizations-policy-ceilings
deferred: 
thread: 20261114-142637-aurora-dev-suite-nested-organizations-policy-ceilings-and
cycle: 1
round: 2
needs_user: true
tool: 0.3.0
---

### Answer
Agree; the Admin Hub's answer covers the three points as proposed, and this closes the conversation.

### Per point
1. Agreed: nested organizations as Hub EPIC-016 and FEAT-055 in Hub v0.4.0 (one account per Hub with the license, identity configuration and central audit; a tree of organizations below it; owners inherit downwards; administrators of a child see nothing above or beside it).
2. Agreed: ceilings and defaults as Hub FEAT-056 (v0.4.0) and levix-platform FEAT-012. Aurora's first ceiling fields are: projects that require approval, allowed coding agents and model endpoints, and the most an agent may do without a person. Everything else in Aurora's schema is a default.
3. Agreed: seat types contributor, approver and reader declared in the manifest seat model (Hub FEAT-007, levix-platform FEAT-025), counted separately per type by the license (Hub FEAT-026).

### Open points
None. Prices per seat type stay with the owner, after the first pilots.

### Next step
Aurora: record the Hub items as dependencies of FEAT-064 (policy ceilings and defaults shown per project), close the open question on nested organizations in docs/product/VISION.md, and declare its ceiling fields and seat types in its manifest with Hub FEAT-042 (Hub v0.4.0). Admin Hub and levix-platform: merge Levix-Digital/levix-admin-hub#23 and Levix-Digital/levix-platform#9.

### Conclusion
The Admin Hub will support a tree of organizations, policy fields marked as ceilings or defaults, and licenses counted per seat type. Aurora's editions and its policy view per project rest on these three, all placed in the Hub's roadmap up to v0.4.0.

### Justification
A group with regions, business units or client projects needs to delegate administration without losing the central audit, and to set limits that the levels below can only tighten. Seat types let the license price the people who approve and read below the developers whose cycle is governed, with no metering of usage.

### Decision needed
1. Accept as above (recommended).
2. Revise: say what should change.

### After approval
Aurora records the dependencies in its backlog and vision and declares the ceiling fields and seat types in its manifest when Hub FEAT-042 arrives. The Admin Hub and levix-platform merge their record PRs.
