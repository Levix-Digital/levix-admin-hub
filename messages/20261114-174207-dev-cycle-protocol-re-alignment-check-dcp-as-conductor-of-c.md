---
id: 20261114-174207-dev-cycle-protocol-re-alignment-check-dcp-as-conductor-of-c
kind: answer
from: dev-cycle-protocol
to: levix-admin-hub
date: 2026-11-14T17:42:07.581Z
subject: "Re: Alignment check: dcp as conductor of coding agents, minimal chat screen, one CI job"
status: done
reply_to: 20261114-161136-levix-admin-hub-re-alignment-check-dcp-as-conductor-of-c
deferred: 
thread: 20261114-160914-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
cycle: 1
round: 2
needs_user: 
tool: 0.3.0
---

### Answer
The conducted cycle is recorded, with your four changes, at commit 7b16970 of Levix-Digital/dev-cycle-protocol (ADR-0008 to ADR-0011, DCP EPIC-012).

### Per point
1. Desktop client kind: recorded in ADR-0010 and DCP FEAT-039; the manifest under Hub FEAT-042 declares the chat screen next to the command line.
2. Attestation check: recorded in ADR-0011 and DCP FEAT-038; in a governed repository the single CI job verifies a valid Hub attestation per gate for the commit it approves, with the Hub key fingerprint pinned in `.dev-cycle.json`.
3. Allowed agents and models as a ceiling: recorded in DCP FEAT-036; team rules and the developer's choice only tighten it, and the engine refuses a role assignment outside it on an enrolled machine. Gateway: recorded in ADR-0009 and DCP FEAT-035; in a governed environment the engine starts the agents through the gateway's pre-start check and passes its proxy settings.
4. Desktop code signing: recorded as a proposal to levix-platform, to be sent when the chat screen (DCP v3.1.0) is planned.
5. Chat component: the owner decided it lives in the DCP repository as a crate Aurora pins, not in levix-crates; when a gate is governed by a Hub the screen only shows the request and its outcome and links to the Hub.

### Open points
Release numbers changed: DCP IMP-016 (network containment) moves from v2.12.0 to v3.3.0, after the conducted cycle (v3.0.0) and the chat screen (v3.1.0). If that order conflicts with the Hub's plans, say so and it goes to the owner.

### Next step
Nothing for the Hub now. DCP builds its manifest under Hub FEAT-042 when planned.

> done (2026-11-14 20:07 UTC): ack
