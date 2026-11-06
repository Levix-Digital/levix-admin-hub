---
id: 8
title: Products registered by signed manifests, no product code in the Hub
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0008: Products registered by signed manifests, no product code in the Hub

## Context

A customer adds to its Hub every product of its contract: AnyContext, DocMapper, Gedo, the Dev Cycle Protocol, Aurora, future Levix products and possibly third-party products. If the Hub knew each product in its code, every new product or schema change would need a Hub release in every customer, many of them air-gapped and updating quarterly.

## Decision

Levi decided on 2026-11-06 that the Hub governs any product the contract adds, current or future:

- A product is registered by importing a **product manifest** signed by Levix: identifier and name, client kinds (desktop, server, CLI, browser), policy schema, data definition formats, seat model, telemetry counters and commands it accepts.
- The Hub validates policies, data definitions, telemetry and commands against the manifest and has no product-specific code.
- The license of the contract lists the products (and their seats) the Hub governs; a manifest without a license is not active.
- The manifest format belongs to `levix-platform` (ADR-0005); each product maintains its own manifest in its repository, released with the product and imported through the update mirror or the console.

## Alternatives Considered

- **Product modules compiled into the Hub**: richer per-product screens, but a Hub release for every product change and no third-party products.
- **Unsigned manifests**: simpler, but anyone could register a product and weaken the governance.

## Consequences

The console renders policy editors and inventories generically from the schema. Each product team owns its manifest; the Hub tests a reference manifest set in CI. New products reach existing customers without a Hub upgrade.
