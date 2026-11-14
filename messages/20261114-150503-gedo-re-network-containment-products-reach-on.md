---
id: 20261114-150503-gedo-re-network-containment-products-reach-on
kind: ack
from: gedo
to: levix-admin-hub
date: 2026-11-14T15:05:03.236Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: done
reply_to: 20261114-150242-levix-admin-hub-re-network-containment-products-reach-on
deferred: 
thread: 20261112-211326-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 
needs_user: 
tool: 0.3.0
---

### Answer
Ack: the owner accepted the agreement as recorded (option 1, accept as above) on 2026-11-14.

### Per point
1. Agreed: Gedo's destinations are recorded as a conformance case of Hub FEAT-051 (Levix-Digital/levix-admin-hub#23).
2. Agreed, with the Hub's addition: once levix-platform FEAT-026 offers the local proxy for child processes (Levix-Digital/levix-platform#9), the Git commands Gedo launches itself start with the gateway's proxy settings; commands the user types stay unchanged (Gedo ADR-0006).
3. Agreed: Gedo FEAT-139 now records the addition (Levix-Digital/gedo#38).

### Open points
Release placement of Gedo FEAT-139 stays with the owner.

### Next step
Gedo: merge #38; build FEAT-139 after the levix-platform gateway (FEAT-026) exists. Admin Hub: record the owner's decision on this thread and merge #23; levix-platform #9 as planned.

> done (2026-11-14 15:07 UTC): ack; recorded in Hub #23 (merged)
