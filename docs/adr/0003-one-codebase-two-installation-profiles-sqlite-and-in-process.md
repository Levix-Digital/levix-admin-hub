---
id: 3
title: 'One codebase, two installation profiles: SQLite and in-process bus or PostgreSQL and Redis'
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0003: One codebase, two installation profiles: SQLite and in-process bus or PostgreSQL and Redis

## Context

The Hub has two installation profiles (docs/product/VISION.md, Scope 12): compact, one container for offices and clinics with no infrastructure team; scalable, replicas behind a load balancer for large companies. The target is that IT deploys the compact profile alone in one afternoon.

## Decision

- Compact profile: one container, SQLite as the embedded database, an in-process message bus, a single replica.
- Scalable profile: PostgreSQL, Redis Pub/Sub as the bus between replicas (each device connection lives on one replica; commands are routed through the bus), stateless replicas, Helm chart.
- Both profiles run the same binary; the profile is a configuration choice that selects the storage and bus adapters. Schema migrations are written once and run on both databases; the test suite runs against both.
- Commands for offline devices are queued in the database and delivered on reconnection in both profiles.

## Alternatives Considered

- **PostgreSQL only**: one database to test, but the compact profile would need a second container and database administration skills that small customers lack.
- **Two codebases or two editions**: each profile simpler, but double maintenance and drift.

## Consequences

Small customers get a single container; large ones get horizontal scaling, from one codebase. SQL is limited to what both databases support, and CI runs the suite twice. A migration path from compact to scalable (export and import) is needed.
