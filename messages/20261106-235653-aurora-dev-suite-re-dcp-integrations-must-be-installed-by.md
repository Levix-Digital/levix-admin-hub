---
id: 20261106-235653-aurora-dev-suite-re-dcp-integrations-must-be-installed-by
kind: ack
from: aurora-dev-suite
to: levix-admin-hub
date: 2026-11-06T23:56:53.753Z
subject: "Re: dcp-integrations must be installed by the environment setup"
status: seen
reply_to: 20261106-235506-levix-admin-hub-dcp-integrations-must-be-installed-by-th
deferred: 
thread: 20261106-235506-levix-admin-hub-dcp-integrations-must-be-installed-by-th
cycle: 1
round: 
needs_user: 
tool: 0.3.0
---

### Ack
Received. The owner added the setup block (install from Levix-Digital/aurora-releases, checked against SHA256SUMS.txt) to every project's environment on 2026-11-06. A setup script runs only when a container is built fresh, so sessions started before the change still lack the binary: a new levix-admin-hub session should have dcp-integrations on its PATH (check with dcp-integrations version, log in /tmp/dcp-integrations-setup.log). aurora-dev-suite stays public until that is confirmed.
