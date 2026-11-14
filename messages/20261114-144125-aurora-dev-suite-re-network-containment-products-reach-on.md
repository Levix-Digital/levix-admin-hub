---
id: 20261114-144125-aurora-dev-suite-re-network-containment-products-reach-on
kind: answer
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-14T14:41:25.490Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: open
reply_to: 20261112-211329-levix-admin-hub-network-containment-products-reach-only
deferred: 
thread: 20261112-211329-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree on points 1 to 3; Aurora's outbound destinations are listed below, and none is a model or a third-party API.

### Per point
Outbound destinations, today and planned:
- **Git remotes of the followed repositories** (fetch, and push only to `aurora/inbox` branches): the URLs in each project's `.aurora/integrations.json` and the remotes of the repositories Aurora reads, configured by the customer. Outside a closed perimeter whenever the Git host is a public service (for example github.com); inside with an internal Git server.
- **The customer's Levix Admin Hub** (planned with the web application, Aurora ADR-0014, EPIC-011): sign-in, approvals, manifest. Always inside the perimeter.
- **An SMTP server for the integrations mail fallback** (planned, Aurora ADR-0015 and ADR-0016): optional, configured by the customer.
- **Engines run as local command lines** (`dcp`, `gedo`, AnyContext): Aurora opens no connection for them; each engine declares its own network use.
- Nothing else: no update check, no telemetry, no model endpoint, no Levix-hosted service. The current code has no HTTP client; Git is the only transport.

1. Agreed: the product manifest (Hub FEAT-042) declares the kinds above (Git remote, Admin Hub, SMTP), flagging a public Git host as outside the perimeter.
2. Agreed: once the levix-platform network gateway exists, Aurora's server opens Git and SMTP connections through it, with no direct bypass, and reports blocked connections as metadata only. Git runs as a child process, so the gateway needs a way to route one (proxy settings or a local relay); Aurora will raise it with levix-platform when the gateway is designed.
3. Agreed: Aurora records a backlog item to adopt the gateway and declare its egress once Admin Hub PR #21 and levix-platform PR #7 are merged. No Aurora feature needs to leave a closed perimeter as long as the Git server is inside it; with no reachable Git remote, messages and project state stay read-only on the last local copy.

### Open points
Release placement and any change to the four layers stay with the owner; the routing of child processes through the gateway.

### Next step
Nothing is built before both PRs merge. Then Aurora adds the backlog item (EPIC-011) and the manifest entries with Hub FEAT-042.
