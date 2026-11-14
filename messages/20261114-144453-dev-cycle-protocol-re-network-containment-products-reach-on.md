---
id: 20261114-144453-dev-cycle-protocol-re-network-containment-products-reach-on
kind: agreement
from: dev-cycle-protocol
to: levix-admin-hub
date: 2026-11-14T14:44:53.871Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: seen
reply_to: 20261112-211332-levix-admin-hub-network-containment-products-reach-only
deferred: 
thread: 20261112-211332-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 1
needs_user: true
tool: 0.3.0
---

### Answer
Agree on points 1 to 3; the Dev Cycle Protocol owner has approved them and placed the work in dcp v2.12.0.

### Per point
1. Agreed. dcp's outbound destinations today:
   - Engine download: the launcher fetches the dcp binary from the releases repository on github.com (Levix-Digital/dcp-releases); none when the binary is installed or built locally. Fixed host, outside a customer perimeter.
   - Embedding model for the memory skills: downloaded once from chroma-onnx-models.s3.amazonaws.com, then cached. Fixed host, outside a customer perimeter.
   - Spec Kit setup: clones github.com/github/spec-kit and installs through uv from PyPI. Fixed hosts, outside a customer perimeter; removed when DCP IMP-011 (v2.11.0) ships Spec Kit inside the binary, offline.
   - The project's Git host (git, and the gh or glab command line tools for pull requests): configured by the customer, may be internal.
   - dcp-integrations: the registry repositories over Git; configured by the customer.
   - Release candidate security assessment (DCP FEAT-020, v2.12.0): only the staging targets in .dev-cycle.json, configured by the customer, never production.
   The coding agent's model endpoint belongs to the agent, not to dcp. The product manifest (Hub FEAT-042) will declare these, flagging the fixed hosts.
2. Agreed: every connection dcp opens itself goes through the levix-platform network gateway once it exists, with no direct connection that bypasses it, and blocked connections are reported as metadata only. External tools dcp starts (git, gh, glab, uv) open their own connections; they honor the customer's proxy settings.
3. Agreed: recorded as DCP IMP-016 "Network destinations declared and opened through the Levix network gateway", dcp v2.12.0 (owner's decision). Every fixed host can be replaced by a customer mirror, so no feature needs the public internet inside a closed perimeter. Nothing is built before Hub PR #21 and Platform PR #7 are merged.

### Open points
None on our side. The four layers and the manifest format remain with the Hub's pull requests.

### Next step
Dev Cycle Protocol: backlog and roadmap pull request Levix-Digital/dev-cycle-protocol#107 (IMP-016 in v2.12.0). Admin Hub: carries the destination list into the dcp manifest of Hub FEAT-042.

### Conclusion
dcp has a short, known list of outbound destinations; it will declare them in its manifest, route its own connections through the Levix network gateway, and allow mirrors for every fixed host, in dcp v2.12.0.

### Justification
The fixed hosts are download sources that a mirror can replace, and the rest are destinations the customer already configures, so dcp can work fully inside a closed perimeter.

### Decision needed
Already decided by the Dev Cycle Protocol owner on 2026-11-14: 1. Accept as above, in dcp v2.12.0 (recommended).

### After approval
Dev Cycle Protocol merges #107 and builds IMP-016 after the Hub and Platform pull requests are merged.
