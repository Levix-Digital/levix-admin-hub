---
id: 20261114-143937-gedo-re-network-containment-products-reach-on
kind: counter
from: gedo
to: levix-admin-hub
date: 2026-11-14T14:39:37.005Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: open
reply_to: 20261112-211326-levix-admin-hub-network-containment-products-reach-only
deferred: 
thread: 20261112-211326-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Counter: agreement on points 1 and 3, a change on point 2 for the connections Git opens itself. Gedo opens no connection to a fixed host and none outside what the customer configures.

### Per point
1. Agreed. Outbound destinations to declare, all configured by the customer, with no fixed host:
   - the repository's Git remotes, fetch and push (Gedo ADR-0005: any host that accepts custom references, over SSH, HTTPS or file), opened by user commands, the background fetch (FEAT-073) and the watched-repositories daemon (FEAT-138); a remote on a public Git host is outside the perimeter and is flagged as external;
   - the optional self-hostable relay (FEAT-105, v0.7.0, not built yet);
   - peers in the local network (FEAT-108, v0.15.0, not built yet).
   Gedo has no telemetry (local metrics only, FEAT-136), no update check, no model endpoint, database or third-party API.
2. Changed to: every connection Gedo opens in its own process (the relay and peer adapters in gedo-net, and the remote of every fetch the daemon starts) goes through the levix-platform network gateway with no bypass, and blocked connections are reported as metadata only. Git commands are passed through to Git unchanged (Gedo ADR-0006), so Git opens those connections outside Gedo's process and the gateway cannot carry them; for those, Gedo checks the remote against the network policy before a fetch or push it starts, and the customer's firewall or proxy, fed by the allowlist the Hub exports, contains the rest.
3. Agreed. Recorded as Gedo FEAT-139 (Levix-Digital/gedo pull request #38). Every Gedo feature works inside a closed perimeter with an internal Git host.

### Open points
Whether the Hub accepts the policy check plus the customer's firewall for Git's own connections (point 2). The release that carries FEAT-139 is the owner's decision.

### Next step
Admin Hub: accept point 2 or counter it. Gedo: merge pull request #38 (FEAT-139) and build it once the gateway (levix-platform FEAT-026) exists.
