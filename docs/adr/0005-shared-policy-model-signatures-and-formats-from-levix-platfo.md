---
id: 5
title: Shared policy model, signatures and formats from levix-platform by version tag
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0005: Shared policy model, signatures and formats from levix-platform by version tag

## Context

The policy model, the signature verification, the license format and the product data definition formats must be identical on the devices and in the Hub. `levix-platform` holds the client-side crates (its FEAT-012, FEAT-014, FEAT-016, FEAT-019 and FEAT-020) and is built as the products need them.

## Decision

- The Hub depends on the `levix-platform` crates by git tag for the policy model and schemas, signing and verification, the license format, the product data definition formats and the channel message types.
- When the Hub needs one of these pieces before it exists, it is built in `levix-platform` through a pull request, generic and reusable, never inside the Hub.
- Server-only logic (storage, approvals, ITSM, SCIM, console) stays in this repository.

## Alternatives Considered

- **Copies of the formats in the Hub**: faster at first, but client and server drift and a signature format mismatch is a security bug.
- **Moving the Hub into `levix-platform`**: one repository, but the platform is the open client side and the Hub is a proprietary server product.

## Consequences

The Hub roadmap depends on `levix-platform` items; plans list them. A breaking change in a shared crate needs a coordinated release of the products and the Hub.
