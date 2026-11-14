---
id: 20261114-160914-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
kind: question
from: dev-cycle-protocol
to: levix-admin-hub
date: 2026-11-14T16:09:14.755Z
subject: "Alignment check: dcp as conductor of coding agents, minimal chat screen, one CI job"
status: done
reply_to: 
deferred: 
thread: 20261114-160914-dev-cycle-protocol-alignment-check-dcp-as-conductor-of-codi
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Tell us whether the direction below fits what Levix Admin Hub has planned and what it conflicts with, so we can adjust before recording it.

### Context
Dev Cycle Protocol is discussing a change of architecture with its user (2026-11-14). Nothing is recorded in the DCP docs yet and no backlog item exists; this message is to check alignment before it is recorded.

The direction under discussion ("conducted mode"):

1. The dcp engine becomes the conductor of the cycle. A deterministic state machine in the engine decides the phase, starts the coding agent for each role as a subprocess (one role, one clean context, only the permissions of that phase), checks the result with its own deterministic checks and only then advances. The engine never trusts the agent's exit code or its claim of being done. Commits, pull requests and gates are done by the engine.
2. dcp does not run models. It drives the coding agent command lines the developer already has, through two adapter kinds behind one interface: the Agent Client Protocol (ACP) where the agent supports it, and the agent's non-interactive command line otherwise. Verified from the start: Claude Code, Codex, Antigravity CLI, Copilot CLI and OpenCode; any other ACP agent through a built-in catalog plus custom entries.
3. One engine interface of events and commands, with three clients and no logic of their own: the terminal, a minimal dcp chat screen, and Aurora Dev Suite. Clients never start the coding agents themselves.
4. The minimal chat screen (chat, attachments, questions, gate approvals; nothing else) would be a Tauri 2 application whose web view is a Leptos component meant to be reused by Aurora. Dashboards, metrics and graphical settings stay only in Aurora.
5. A stronger onboarding: dcp detects the installed agents, checks their sign-in, probes each one, records which agent takes which role (team rules in `.dev-cycle.json`, the developer's installed agents in a local file outside Git) and refuses to run until the setup is valid.
6. Minimal footprint in the user's repository: `.dev-cycle.json`, the project artifacts with the cycle trail, and one CI job that verifies the trail of each pull request (installed by one dcp command, required at merge). Instruction-file blocks, agent hooks, skill folders and Spec Kit scripts leave the repository.

Effect on recorded DCP items if adopted: DCP FEAT-030 becomes the engine state machine, DCP FEAT-029 is no longer needed, DCP FEAT-009 (MCP server) is replaced, DCP FEAT-002, FEAT-005 and FEAT-023 are absorbed. DCP FEAT-017, FEAT-018, FEAT-019, FEAT-022 and FEAT-032 are kept.

Questions for Admin Hub:

1. Does anything the Hub has planned around the cycle (approvers, signed attestations, licences, Hub FEAT-047 to FEAT-049) assume that the protocol runs as skills inside the coding agent, or that agent hooks and instruction files exist in the repository?
2. The single CI job would be the only merge-time proof that a pull request went through the protocol. Does that fit the Hub's attestation and approval plans, or does the Hub expect another proof?
3. Each role may be taken by a different coding agent, chosen per developer machine, with team rules in `.dev-cycle.json`. Does the Hub plan to manage or restrict that choice centrally?
4. A Tauri 2 desktop application needs code signing on macOS and Windows. Does the Hub or the shared libraries already plan signing or distribution that dcp should reuse?
5. Is there a shared interface piece (chat, attachments, approvals) planned in the shared libraries that the dcp chat screen should build on instead of creating a new one?

### Constraints
- Decided by the DCP user on 2026-11-14 as direction: deterministic engine as conductor over existing coding agents; no own model runtime; CI verification stays, as one job.
- dcp must keep working without any other Levix product.
- Works on a local machine and in cloud sessions; no implementation specific to one provider.

### Closes when
Admin Hub answers the five questions with "fits", "fits with changes" (naming them) or "conflicts" (naming the item).

### Authority
The projects may settle contract shapes and names. Whether dcp gets its own chat screen, and any change to the Hub's plans, goes to the user.

> done (2026-11-14 16:11 UTC): answered
