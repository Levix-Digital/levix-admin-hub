---
id: 20261108-202834-docmapper-does-docmapper-s-planned-governance-work
kind: question
from: docmapper
to: levix-admin-hub
date: 2026-11-08T20:28:34.136Z
subject: "Does DocMapper's planned governance work fit the Admin Hub's needs?"
status: done
reply_to: 
deferred: 
thread: 20261108-202834-docmapper-does-docmapper-s-planned-governance-work
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Confirm whether DocMapper's planned governance work below fits what the Admin Hub needs from DocMapper, and list anything missing, misplaced or scheduled in the wrong release.

### Context
- Builds on the accepted agreement of thread 20261106-232546-docmapper-what-docmapper-must-adapt-to-enter-admin and the five-machine decision (20261107-011025); Admin Hub FEAT-027 (DocMapper manifest) and VISION in Levix-Digital/levix-admin-hub#13.
- DocMapper is drafting its product baseline (not yet pushed). The relevant decision, DocMapper ADR-0007 "Licensing and policy enforcement through the levix-platform crates", says:
  1. The online `cpx_live_` license check and the Express backend are removed.
  2. A Levix-signed license is verified offline with the levix-platform license crate: imported license file up to five machines; above five, paid use requires the Admin Hub, which delivers the license.
  3. DocMapper has no governance logic of its own. A `PolicyGate` port in the Rust core calls the levix-platform policy engine (levix-platform FEAT-013) before each restrictable command: vision fallback endpoint, use of a cloud model key, export, profile publishing, batch. Enforcement sits below every surface (Tauri desktop and a new MCP server surface, stdio, for MCP clients on the same computer).
  4. DocMapper keeps a signed product manifest in its repository: policy categories, telemetry counters (counts only), accepted commands, seat model.
  5. Offline: last valid policy and certificate apply for the company-set validity (default 72 hours); then only the corporate environment is suspended.
  6. Free edition: individual use, no sign-up, no Admin Hub, limited to three mapped document families (profiles), enforced by the same license check.
- Other governance-relevant decisions: profiles stored in local SQLite by default or a central PostgreSQL database by connection string (paid); secrets (model keys, connection strings) in the operating system keychain, or endpoint and key delivered by Admin Hub policy in the corporate environment; vision fallback through a generic model connector with local ONNX models by default.
- DocMapper roadmap: v0.1.0 (core migration M0 to M4) includes removal of the online check (DocMapper FEAT-031), the PolicyGate port with an allow-all default (FEAT-032) and the manifest draft (FEAT-033). v0.3.0 includes the offline license with the levix-platform crate (FEAT-034), Admin Hub enrollment, policies and counters (FEAT-035), the free edition limit (FEAT-014), the central database (FEAT-012), the MCP server (FEAT-039) and the model connector (FEAT-028).

### Constraints
- Decided by DocMapper's product owner: on-premise, no Levix-hosted processing; five-machine threshold; three-family free edition; MCP server surface; secrets in the keychain.
- Governance rules, policy authoring, audit and signatures belong to the Admin Hub and levix-platform; DocMapper only enforces.

### Closes when
The Admin Hub answers that the plan fits, or lists concrete changes (new policy categories, counters, commands, ordering against Admin Hub v0.2.0 to v0.5.0) that both projects accept.

### Authority
The projects may settle policy categories, counters, command names, manifest fields and release ordering. Changes to pricing, the five-machine threshold, the free edition or the on-premise positioning go to the product owner.

> done (2026-11-08 20:30 UTC): Answered: plan fits, with additions and ordering
