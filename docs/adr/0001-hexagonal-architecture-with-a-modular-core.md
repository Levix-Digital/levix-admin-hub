---
id: 1
title: Hexagonal architecture with a modular core
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0001: Hexagonal architecture with a modular core

## Context

The Admin Hub serves the same rules through several interfaces: the REST API, the device WebSocket channel, the SCIM endpoints, the web console and the administration CLI. The compact and scalable profiles (docs/product/VISION.md, Scope 12) are the same code with a different database and message bus, and identity providers, ITSM tools and the update mirror storage are replaceable integrations. The sibling projects (AnyContext, Aurora, Gedo) use a hexagonal core.

## Decision

- Top-level style: hexagonal. `crates/hub-core` holds the domain and the use cases, with no framework, transport, database or driver code; it declares ports (traits) for storage, the message bus, the clock, key custody, identity providers, ITSM providers and the mirror store.
- Inside the core, code is grouped in modules by capability (organizations, identity, devices, licensing, policies, approvals, break-glass, audit, sharing, mirror, servers), each with a small public interface; a module never reaches into another module's internals or tables.
- Driving adapters: `hub-api` (REST, SCIM), `hub-channel` (device WebSocket), `hub-console` (web console), `hub-cli` (administration CLI). Driven adapters: storage, bus, identity, ITSM and mirror crates.
- One composition root per executable. `check_design` enforces the dependency rule through `architecture.forbidden_core_imports`.

## Alternatives Considered

- **Modular monolith alone**: clear module boundaries, but database and bus swaps between profiles would leak into every module.
- **Layered**: simpler, but gives no guarantee that the core stays free of the framework and driver types.

## Consequences

Every user action is one core use case shared by all surfaces, so the console, the CLI and the API cannot diverge. Each adapter can be tested in isolation and the profiles differ only in wiring. Ports are added only where there are two implementations or a test need.
