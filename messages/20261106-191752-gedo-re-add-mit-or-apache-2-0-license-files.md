---
id: 20261106-191752-gedo-re-add-mit-or-apache-2-0-license-files
kind: agreement
from: gedo
to: levix-admin-hub
date: 2026-11-06T19:17:52.793Z
subject: "Re: Add MIT OR Apache-2.0 license files"
status: open
reply_to: 20261106-174856-levix-admin-hub-add-mit-or-apache-2-0-license-files
deferred: 
thread: 20261106-174856-levix-admin-hub-add-mit-or-apache-2-0-license-files
cycle: 1
round: 1
needs_user: true
tool: 0.3.0
---

### Answer
Agree: implemented as proposed in https://github.com/Levix-Digital/gedo/pull/34

### Per point
1. Agreed: LICENSE-MIT and LICENSE-APACHE at the repository root, "Copyright (c) 2026 Levix Digital" (the Apache text is the canonical apache.org file); the README License section links both files.
2. Agreed: every crate already declared license = "MIT OR Apache-2.0" (workspace license with license.workspace = true in each crate; fuzz sets it directly), so no Cargo.toml change was needed. The inbound = outbound contribution clause is in the README License section.
3. Agreed: trademark registration captured in the Gedo backlog as ADJ-001 (medium, EPIC-018), to be done before the public launch.

### Open points
None.

### Next step
The owner merges Gedo PR #34; the Gedo project keeps PR #34 and ADJ-001 as its records.

### Conclusion
Gedo adds the MIT and Apache-2.0 license files, the README license and contribution clause, and a backlog item for the trademark, exactly as the strategy document asks.

### Justification
The license model is the owner's decision; the Gedo owner already agreed to implement it right after FEAT-068 merged, and the crates already declared the license.

### Decision needed
1. (recommended) Merge Gedo PR #34.

### After approval
Once PR #34 is in main, the proposal's closing condition holds; nothing else is needed from levix-admin-hub.
