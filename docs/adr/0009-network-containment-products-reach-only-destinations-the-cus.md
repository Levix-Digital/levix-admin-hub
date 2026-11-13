---
id: 9
title: 'Network containment: products reach only destinations the customer approves'
status: accepted
date: '2026-11-12'
version: null
subsystems: []
supersedes: []
superseded_by: []
amends: []
amended_by: []
---

# ADR-0009: Network containment: products reach only destinations the customer approves

## Context

Customers must be sure that the products the Hub governs never leave their internal network: no model, database or API outside their VPC or on-premises perimeter. ADR-0004 keeps the Hub itself inside the perimeter with no outbound connection, but says nothing about the products. The shared policy model (ADR-0005, levix-platform FEAT-012) has common categories for accounts, data and quotas only; network destinations exist only as a product-specific category of AnyContext (models and endpoints, web search and domains). Product manifests (ADR-0008) do not declare where a product connects, so the Hub cannot tell a customer which destinations a product needs, nor prove that devices respect them.

## Decision

Containment has four layers. The Hub owns the definition, the approval and the evidence; the products enforce; the customer's network blocks.

1. **Perimeter and approved destinations in the Hub.** The customer declares its internal networks in the Hub (address ranges, DNS suffixes, private endpoints). Every destination that reaches a policy (corporate model endpoints, databases, APIs, the update mirror) is checked against them. A destination outside the perimeter is refused unless an administrator with the right role approves it explicitly, with a second-factor step-up and a reason, recorded in the audit and marked as external in the console.
2. **Egress declared in the product manifest and reviewed at registration.** Each manifest declares the destinations its product needs: kinds configured by the customer (a model endpoint, a database) and any fixed host. When a manifest is imported, the Hub lists them; a product whose manifest declares a fixed destination outside the perimeter stays inactive until an administrator approves that destination or a policy turns the feature that needs it off.
3. **Enforcement on devices and product servers.** Network containment is a common category of the shared policy model, deny by default in the corporate environment: a product may connect only to the Hub and to the approved destinations. The levix-platform governance engine enforces it below every surface of the product; each blocked connection is recorded as metadata only (rule, destination, when, device), never with content.
4. **Evidence and the customer's network.** Devices report in the heartbeat the policy version in force and counters of blocked connections; the console shows containment compliance per device and alerts on drift (an old policy past a threshold, blocked attempts, a device that stops reporting). The Hub exports the approved destinations as an allowlist (JSON and CSV, with templates for a forward proxy and Kubernetes network policies) for the customer's IT to apply in its firewall, proxy or cluster. Blocking at the network remains the customer's responsibility: the Hub never configures the network and keeps making no outbound connection (ADR-0004).

The network category, the gateway that enforces it and the manifest egress declaration are built in levix-platform (ADR-0005); the Hub keeps the perimeter, approvals, compliance view and allowlist export.

## Alternatives Considered

- **Network controls only (customer firewall)**: the strongest block, but the Hub could not show what each product needs nor prove what devices applied, and a laptop outside the corporate network escapes it.
- **Product controls only**: works anywhere, but a compromised or modified binary is not stopped; network controls remain the last barrier.
- **Network destinations as a category of each product**: already the case in AnyContext, but every product would define it differently and the Hub could not validate destinations against the perimeter generically.
- **The Hub configures firewalls or cloud network rules directly**: less work for IT, but it gives the Hub outbound connections and credentials over the customer's network, against ADR-0004.

## Consequences

levix-platform gains a common network category in the policy model, a network gateway enforced by the governance engine and an egress declaration in the manifest format. Every product routes its outbound connections through that gateway and declares its destinations in its manifest; each product project evaluates its side. Heartbeat and audit messages of the channel contract (ADR-0006) gain the policy version in force and blocked-connection counters. Customer documentation states the split: the Hub defines, approves and proves; the products enforce; the customer's network blocks.
