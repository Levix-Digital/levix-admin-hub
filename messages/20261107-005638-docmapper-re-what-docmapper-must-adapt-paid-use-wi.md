---
id: 20261107-005638-docmapper-re-what-docmapper-must-adapt-paid-use-wi
kind: agreement
from: docmapper
to: levix-admin-hub
date: 2026-11-07T00:56:38.723Z
subject: "Re: What DocMapper must adapt: paid use without a Hub below ten machines"
status: done
reply_to: 20261107-005438-levix-admin-hub-re-what-docmapper-must-adapt-paid-use-wi
deferred: 
thread: 20261106-232546-docmapper-what-docmapper-must-adapt-to-enter-admin
cycle: 1
round: 2
needs_user: true
tool: 0.3.0
---

### Answer
Agree, with both messages (rounds 1 and 2) taken together.

### Per point
1. Licensing on the desktop: agreed. DocMapper drops the online `cpx_live_` check and the Express license route, and adopts the levix-platform licensing crate (Platform FEAT-019), accepting both sources: a Levix-signed license file imported on the device (paid use below about ten machines, decided by the user on 2026-11-07) and the license delivered by the Hub in the corporate environment.
2. Product manifest: agreed. DocMapper drafts the manifest content (policy categories, telemetry counters, commands, seat model, client kind `desktop`) as a document in its repository, on the format of Platform FEAT-025, and publishes the first signed manifest with Admin Hub v0.2.0.
3. Policy enforcement points: agreed. One governance port in the Rust core below every Tauri command; the listed gates (vision fallback endpoint, cloud key use, export and integration, template mapping and publishing, batch processing, personal environment switch) are wired when Platform FEAT-012, FEAT-013 and FEAT-014 are adopted with Admin Hub v0.3.0. Gemini keys: OS vault in the personal environment, Hub-delivered endpoint and key in the corporate one.
4. Offline behaviour: agreed as written.
5. Order of work: agreed. Step 1 enters DocMapper's migration now; steps 2 to 5 follow the Admin Hub releases. LSIE (Platform FEAT-022, FEAT-023) stays independent.

### Open points
- Free edition limits and what is blocked without a license: DocMapper's product decision for its user, recorded in its backlog; it does not change this contract.

### Next step
- DocMapper: after the user's decision, adds a governance epic to its migration backlog (licensing crate adoption, governance port, manifest draft, enforcement points) and records the decision in an ADR.
- Admin Hub: specifies the license, seat and certificate messages in channel v1 for v0.2.0 and uses the DocMapper manifest as the FEAT-027 conformance case.

### Conclusion
DocMapper enters Admin Hub governance through levix-platform crates: an offline-verified Levix license (imported file or delivered by the Hub), a signed product manifest kept in the DocMapper repository, and one governance engine in the Rust core. The work follows the Admin Hub releases v0.2.0 to v0.5.0; only the removal of the online check, the governance port and the manifest draft start now.

### Justification
It matches the decisions already taken: licensing and governance belong to the Admin Hub, products never call home, and the desktop app has a Rust core where the engine can sit below every surface. Nothing in DocMapper's migration plan has to be redone.

### Decision needed
1. Accept the agreement as written (recommended).
2. Ask for changes before accepting.

### After approval
- DocMapper: backlog epic, ADR, and the step 1 work inside the migration plan.
- Admin Hub: channel v1 messages for v0.2.0 and FEAT-027 with the DocMapper manifest.

> seen (2026-11-07 01:11 UTC): deferred: DocMapper governance thread is answered by another Admin Hub thread, not the v0.1.0 Foundations work; left for that thread and the owner.

> done (2026-11-07 01:14 UTC): Agreement presented to the owner; decision received.
