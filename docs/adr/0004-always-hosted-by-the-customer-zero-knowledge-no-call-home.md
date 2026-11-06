---
id: 4
title: Always hosted by the customer, zero knowledge, no call home
status: accepted
date: '2026-11-06'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0004: Always hosted by the customer, zero knowledge, no call home

## Context

The Levix products promise that content never leaves the customer's perimeter and that the customer governs everything. The initial specification listed a Levix SaaS as a hosting option and embedded the Hub public key in the product binaries; the AnyContext v2 redesign (October 2026) corrected both.

## Decision

- The Hub always runs in the customer's VPC or on-premises; there is no Levix-hosted SaaS and the Hub never calls Levix.
- The Hub never receives, stores or processes content (documents, chunks, vectors, prompts, conversations, file contents). It stores identities, devices, licenses, policies, approvals and metadata-only audit.
- Passwords and user keys are created on the device and never go through the Hub; the organization recovery private key never lives in the Hub.
- Each Hub has its own key pair; devices pin its fingerprint at enrollment. No Hub key is embedded in any product binary.
- Licenses signed by Levix are verified offline with the Levix public key.

## Alternatives Considered

- **Levix SaaS option**: easier onboarding for small customers, but breaks the core promise and the product category.
- **Hub key embedded in product binaries**: no pinning step, but one key for every customer, or one build per customer.

## Consequences

The Hub must be easy to install and operate without Levix (guided installation, diagnostics, backup and restore). Every new feature is checked against the zero-knowledge rule in review. Source telemetry is limited to the level the policy sets.
