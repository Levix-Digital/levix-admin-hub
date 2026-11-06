---
id: 6
title: Public, versioned contract for the API and the device channel
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0006: Public, versioned contract for the API and the device channel

## Context

Every Levix product, in any language, talks to the Hub; auditors and integrators will read how. The strategy requires the contract to be public and versioned from the start, and the Hub must govern several products of different ages at once.

## Decision

- The REST API is described in OpenAPI and the device WebSocket channel in a versioned message specification, both kept in `docs/contracts/` and published with each release.
- The channel endpoint carries its major version (`/ws/device/v1`); a major version stays supported for at least two Hub minor releases after the next one ships.
- Every message from the Hub to a device (policies, certificates, commands) is signed by the Hub key; devices authenticate with their Ed25519 identity by signing a nonce challenge.
- Conformance tests generated from the contract run in CI, and the message types come from the shared `levix-platform` crates.

## Alternatives Considered

- **Private, internal contract**: freedom to change, but auditors and integrators cannot verify, and products in other languages cannot integrate.
- **gRPC**: strong typing, but harder through corporate proxies and for simple webhook-style integrations.

## Consequences

Contract changes are reviewed like code and versioned. Breaking changes need a new major version and a support window. The contract is part of the security review.
