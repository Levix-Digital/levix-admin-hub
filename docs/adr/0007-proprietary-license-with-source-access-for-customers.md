---
id: 7
title: Proprietary license with source access for customers
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0007: Proprietary license with source access for customers

## Context

Regulated customers want to audit the code that runs in their perimeter, but Levix does not want the code of its proprietary products public. The distribution strategy first suggested a public source-available license (BSL or FSL); both make the source readable by anyone, and FSL also allows free internal production use, against the rule that the Hub is never free. An earlier decision (2026-10-27, Aurora ADR-0005) made every product proprietary and binary-only, with no source access at all.

## Decision

Levi decided on 2026-11-06 that the proprietary products (Admin Hub, AnyContext, DocMapper, Aurora) use a proprietary license with source access for customers only, recorded in the strategy document v2 and in `LICENSE`:

- Use in object form only under an active agreement and a license signed by Levix Digital.
- A customer with an active agreement may request, under confidentiality, the source code of the exact versions it runs, to read, build, analyze and compare with the delivered binaries, and share it with its auditors under equivalent confidentiality.
- No redistribution, no modification for production or production builds from source, no competing use, no circumvention of license verification.
- Source access is offered from the first customer contract; reproducible builds back it; third-party source escrow is optional for large and public-sector customers.

## Alternatives Considered

- **BSL**: production use can stay paid, but the source is public.
- **FSL**: simple and standard, but the source is public and internal production use is free.
- **Binary-only with no source access** (Aurora ADR-0005): simplest, but regulated buyers cannot audit.

## Consequences

Each release needs a source bundle and reproducible build instructions, and the delivery of source must be tracked per customer. Dependencies stay permissive so the source can be shared. The license text needs legal review before the first agreement.
