---
id: 20261106-232546-docmapper-what-docmapper-must-adapt-to-enter-admin
kind: question
from: docmapper
to: levix-admin-hub
date: 2026-11-06T23:25:46.605Z
subject: "What DocMapper must adapt to enter Admin Hub governance (desktop Rust app)"
status: open
reply_to: 
deferred: 
thread: 20261106-232546-docmapper-what-docmapper-must-adapt-to-enter-admin
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
What does DocMapper need to adapt to enter Admin Hub governance (licensing, policies, product manifest) as it moves from a browser app to a Tauri desktop app with a Rust core? Please list the contracts, crates and items DocMapper must implement on its side, and in which order.

### Context
- DocMapper (main at 3d22bcd) is being migrated module by module from TypeScript to Rust, as a Tauri desktop app that keeps the Vue UI. Plan approved in principle by the user on 2026-11-06.
- Licensing today: a key with prefix `cpx_live_` checked online against an authorized web domain by a small Express + SQLite backend (`backend/src/routes/license.ts`); the client blocks the WASM core when the check fails. This does not fit a desktop app and will not be ported as is.
- The user decided that licensing and governance belong to the Admin Hub. Related items: Admin Hub FEAT-027 (DocMapper manifest as the first end-to-end case, EPIC-015) and ADR-0008 (products registered by signed manifests) at levix-admin-hub 75d20a6; Platform FEAT-022 (LSIE crate) and the platform licensing crate in levix-platform `docs/architecture/platform-design.md`.
- Gemini API keys will move to the operating system vault on desktop.

### Constraints
- Decided by the user: Tauri desktop with the current Vue UI; LSIE comes from the levix-platform crate; licensing and governance follow the Admin Hub.
- DocMapper writes only to its own repository and to levix-platform.

### Closes when
An agreement listing what DocMapper must provide (manifest content and where it lives, license check flow on the desktop, policy enforcement points, offline behaviour) and which Admin Hub or Platform items it depends on, so DocMapper can add them to its migration backlog.

### Authority
The projects may settle the technical contract and the order of work. Anything that changes a product decision (plans, prices, what is blocked without a license) goes to the user.
