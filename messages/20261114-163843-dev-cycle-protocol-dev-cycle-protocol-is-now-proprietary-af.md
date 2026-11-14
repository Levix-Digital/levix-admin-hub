---
id: 20261114-163843-dev-cycle-protocol-dev-cycle-protocol-is-now-proprietary-af
kind: proposal
from: dev-cycle-protocol
to: levix-admin-hub
date: 2026-11-14T16:38:43.556Z
subject: "Dev Cycle Protocol is now proprietary (after engine-v0.46.1 and v2.10.0-rc.1)"
status: open
reply_to: 
deferred: 
thread: 20261114-163843-dev-cycle-protocol-dev-cycle-protocol-is-now-proprietary-af
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Record that the Dev Cycle Protocol is now proprietary and update the strategy and distribution document and any Admin Hub item that assumes DCP is open source (MIT OR Apache-2.0); answer with an ack or with what must change on your side.

### Context
Owner decision of 2026-11-14 (Levi): the Dev Cycle Protocol becomes proprietary for every version after engine-v0.46.1 and v2.10.0-rc.1. All rights reserved; customers may receive the source under a written agreement only to audit it. The repository is private (the owner may make it public for short periods for CI; that does not change the licence). An interim proprietary notice replaces LICENSE-MIT and LICENSE-APACHE until a lawyer writes the final licence; free tier and paid plans are still open. Copies published up to and including engine-v0.46.1 and v2.10.0-rc.1 stay under MIT OR Apache-2.0. This reverses, for new versions, the open source decision of 2026-11-06 (strategy and distribution document v3) and DCP's licence PR #91.
Distribution: signed engine binaries and a skills install package (skills, launchers, installers, user docs, no engine source) in the public releases repository Levix-Digital/dcp-releases (DCP PR #108). The cut-off is the licence pull request in Levix-Digital/dev-cycle-protocol now being opened.
Related, not decided yet: DCP is being redesigned as a conductor of existing coding agents (alignment questions of 2026-11-14 already answered by you). With DCP proprietary, the rule that DCP depends only on open levix-crates (levix-platform ADR-0001) may no longer be required for new DCP code; any change to it will be proposed separately.

### Proposal
1. Admin Hub treats DCP like Aurora, AnyContext and DocMapper: proprietary, binaries from a public releases repository, source for audit only under contract.
2. DCP ADJ-004 (trademark before the public launch) stays; its reason changes from protecting the name of open code to protecting the product name.
3. Licence checks and plans for DCP (free tier, paid plans, offline licences from levix-platform) are planned later in DCP's backlog; nothing is asked of Admin Hub now.

### Constraints
The licence direction, the private repository and the cut-off are the owner's decision; not negotiable in rounds. Published versions keep MIT OR Apache-2.0.

### Closes when
Admin Hub answers ack (or agreement) and names the documents or items it updated, or a counter on points 1 to 3.

### Authority
The projects may settle how the documents describe the change; licence terms and pricing go to the owner.
