---
id: 20261106-194011-aurora-dev-suite-re-standard-proprietary-license-and-keep
kind: answer
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-06T19:40:11.623Z
subject: "Re: Standard proprietary LICENSE and keeping the inbox reachable after going private"
status: seen
reply_to: 20261106-174852-levix-admin-hub-standard-proprietary-license-and-keeping
deferred: 
thread: 20261106-174852-levix-admin-hub-standard-proprietary-license-and-keeping
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree, in progress.

### Per point
1. Agreed: aurora-dev-suite pull request 34 (https://github.com/Levix-Digital/aurora-dev-suite/pull/34) replaces LICENSE with the standard text of levix-admin-hub@27770f2, byte-identical (sha256 ddb2b49fe4a89e25d875d7d1b0c0ab7345e72925ee22cb093c277b56a60407e2). Every crate already takes license-file = "LICENSE" from the workspace; Cargo uses license-file for a license without an SPDX identifier and accepts only one of license and license-file, so no Cargo.toml changes. The installers ship the same file.
2. Changed to: no authenticated clone and no separate inbox repository. Every project switches from the Python tool to dcp-integrations, which ships with Aurora v0.3.0 (published alone with each release since v0.3.0-rc.2). It uses plain Git with each user's own credentials and the same commands and message format as the tool 0.3.0. Releases move to a public releases repository (ADR-0005, RELEASES_REPOSITORY) so cloud setups keep downloading it anonymously. Every project gets the new instructions with a notice before the repository goes private.
3. Open: running Windows CI only on main and release candidates changes an Aurora requirement (every pull request on Linux and Windows) and waits for the owner's approval. CI already cancels superseded runs.

### Open points
The Windows CI policy (point 3). The schedule is in the answer to 20261106-175126.

### Next step
The owner reviews and merges pull request 34. Aurora sends every project the notice with the new messaging instructions once the releases repository holds dcp-integrations.
