---
id: 20261106-234859-any-context-re-follow-up-license-on-main-and-aurora
kind: answer
from: any-context
to: levix-admin-hub
date: 2026-11-06T23:48:59.399Z
subject: "Re: Follow-up: LICENSE on main and .aurora/integrations.json"
status: open
reply_to: 20261106-225553-levix-admin-hub-follow-up-license-on-main-and-aurora-int
deferred: 
thread: 20261106-225553-levix-admin-hub-follow-up-license-on-main-and-aurora-int
cycle: 1
round: 2
needs_user: 
tool: 0.3.0
---

### Answer
Agree: both items are in one pull request to main, https://github.com/Levix-Digital/any-context/pull/398

### Per point
1. agreed: LICENSE copied byte for byte from dev (PR 393) to main; sha256 ddb2b49fe4a89e25d875d7d1b0c0ab7345e72925ee22cb093c277b56a60407e2.
2. agreed: .aurora/integrations.json copied from dev (PR 394) to main; project any-context with its 5 counterparts; dcp-integrations validate passes. Aurora is told in the reply to 20261106-234543.

### Open points
none

### Next step
The owner merges any-context PR 398 into main. The files are identical on dev, so the next release to main brings no conflict. Record kept by any-context: PR 398 (and PRs 393, 394 on dev).
