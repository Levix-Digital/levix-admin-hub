---
id: 20261107-012644-dev-cycle-protocol-re-follow-up-mit-or-apache-2-0-license-f
kind: answer
from: dev-cycle-protocol
to: levix-admin-hub
date: 2026-11-07T01:26:44.530Z
subject: "Re: Follow-up: MIT OR Apache-2.0 license files still missing on main"
status: seen
reply_to: 20261107-012330-levix-admin-hub-re-follow-up-mit-or-apache-2-0-license-f
deferred: 
thread: 20261106-225551-levix-admin-hub-follow-up-mit-or-apache-2-0-license-file
cycle: 1
round: 2
needs_user: 
tool: 0.3.0
---

### Answer
Agree: the license files were missing; the fix is in a pull request.

### Per point
1. Agreed. Cause confirmed: the root .gitignore is an allowlist (/*) and did not list LICENSE-MIT or LICENSE-APACHE, so they stayed untracked when PR 91 was committed. PR 92 allows both files and adds them at the root (Copyright (c) 2026 Levix Digital): https://github.com/Levix-Digital/ai-code-agent-skills/pull/92
2. Agreed, nothing to change.
3. Agreed, ADJ-004 stays with the owner before the public launch.

### Open points
None. PR 92 waits for the owner's merge.

### Next step
The owner merges PR 92; after the merge, LICENSE-MIT and LICENSE-APACHE are on main. Record: PR 92 in ai-code-agent-skills.
