---
id: 2
title: Rust service on Tokio and Axum with a Leptos web console
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0002: Rust service on Tokio and Axum with a Leptos web console

## Context

The products and `levix-platform` are Rust; the platform crates (policy model, signature verification, formats) must be identical on the client and the server. The initial specification planned Python and FastAPI, which the products removed. The Hub needs HTTP, WebSocket and a web console that IT opens in a browser inside the perimeter, served by the Hub itself with no external CDN.

## Decision

- Rust, stable toolchain, on Tokio. HTTP and WebSocket with Axum; serialization with Serde; database access with SQLx.
- Web console in Leptos compiled to WASM, built into the Hub binary and served by it, with no external assets (works in air-gapped networks). Same interface stack as Aurora.
- Administration CLI as a separate Rust binary that talks to the Hub through the public API.
- Dependencies permissive only (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0), checked with `cargo-deny`; SBOM produced at each release.

## Alternatives Considered

- **Python and FastAPI** (initial specification): fast to start, but the platform crates would be duplicated in another language and the product line is Rust only.
- **Console as a separate JavaScript application**: larger ecosystem, but a second language, a build chain with many dependencies to audit, and assets to host.

## Consequences

One language and one dependency audit across client and server. The console is shipped and versioned with the Hub. Leptos is younger than mainstream JavaScript frameworks; the console stays simple (forms, tables, approvals) to limit that risk.
