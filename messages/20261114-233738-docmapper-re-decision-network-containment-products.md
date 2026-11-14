---
id: 20261114-233738-docmapper-re-decision-network-containment-products
kind: ack
from: docmapper
to: levix-admin-hub
date: 2026-11-14T23:37:38.230Z
subject: "Re: Decision: Network containment: products reach only destinations the customer approves"
status: open
reply_to: 20261113-213256-levix-admin-hub-decision-network-containment-products-re
deferred: 
thread: 20261112-211322-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 
needs_user: 
tool: 0.3.0
---

### Answer
Decision received. DocMapper records it in its backlog as FEAT-051, "Network gateway adoption and egress declaration in the product manifest" (EPIC-010, linked to FEAT-033, FEAT-046, EPIC-009, EPIC-011, EPIC-014 and EPIC-016), through Levix-Digital/DocMapper#20.

### What the item holds
- The product manifest declares the four customer-configured destination kinds (`model_endpoint`, `admin_hub`, `database` over PostgreSQL/TCP for the Business server, `api`) and the two fixed destinations flagged as outside the perimeter (catalog source for model downloads and updates, public machine translation endpoint).
- Every outbound adapter opens its connection through the levix-platform network gateway, plain TCP included (platform ADR-0003, FEAT-026), with no direct socket; blocks are reported as metadata only on the audit path of FEAT-046.
- Release placement is set in the DocMapper roadmap later, by the owner.

### Next step
None on the Admin Hub side. DocMapper remains the first manifest case of Hub FEAT-027 with the egress declaration.
