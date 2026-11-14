# Levix Admin Hub

This file is the single source of truth for any coding agent working here. `CLAUDE.md` only imports it. Tool-specific features (hooks, subagents, JSON agents) are a convenience and never a dependency: everything an agent needs is in this file and in `.agents/`.

## 1. What this repository is

The Levix Admin Hub: the customer-hosted governance hub of the Levix products for companies. IT installs it inside its own perimeter, creates organizations and administrators, enrolls devices and product servers, and signs and distributes policies, licenses and approvals, with a metadata-only audit and no outbound connection (zero knowledge, no call home). It is a Rust service on Tokio and Axum with a Leptos web console and an administration CLI, in hexagonal architecture with a modular core. The product vision is in `docs/product/VISION.md`, the technical constraints in `docs/product/TECH_SPEC.md` and the system overview in `TECDOC.md`.

Map:

| Path | What lives there |
| --- | --- |
| `AGENTS.md` | Instructions for agents (this file) |
| `.agents/skills/` | Project skills, one folder per skill with a `SKILL.md` (the Spec Kit skills live here) |
| `.agents/memory/` | What past sessions learned (see section 4) |
| `docs/adr/` | Decision records (ADR format) |
| `docs/architecture/` | One page per subsystem (service and storage, administration and access, console, CLI, design) |
| `docs/product/` | `VISION.md` and `TECH_SPEC.md` |
| `docs/contracts/` | The public, versioned API and device channel contract, permissions and its changelog |
| `docs/security/` | Threat model and security architecture |
| `docs/BACKLOG.md`, `docs/ROADMAP.md` | Work items and releases |
| `docs/plans/` | Plans of the Dev Cycle Protocol, one per feature |
| `docs/releases/` | Release notes; the candidate notes file gates automatic release candidates |
| `specs/` | Spec Kit specifications, one folder per feature |
| `crates/` | The Rust workspace: `hub-core` (domain, no adapters), the API, channel, console, CLI, server and storage crates |
| `console-tests/` | Browser and accessibility tests of the console (Playwright, axe-core) |
| `tests/MANUAL_TESTS.md` | Manual tests of a release candidate |
| `deploy/` | Container file and deployment examples |
| `.github/workflows/` | CI (`Tests`), automatic release candidates, release and promote |
| `.specify/` | Spec Kit configuration and the generated constitution (`.specify/memory/constitution.md`) |
| `.dev-cycle.json` | Dev Cycle Protocol configuration |
| `.aurora/integrations.json` | Registry of the other Levix projects for cross-project messages |

Skills are in `.agents/skills/`. If your tool does not list them, read the skill's `SKILL.md` from there. `.claude/skills` is a versioned relative link to `../.agents/skills`. If it is a plain file instead of a link (a Windows clone without symlink support), tell the person and show the fix:

```
# once per machine: turn on Developer Mode in Windows settings, then
git config --global core.symlinks true
# in this clone
del .claude\skills
git checkout -- .claude/skills
```

Do not replace the link with a junction: it leaves `git status` dirty.

## 2. How to work

This repository uses the Dev Cycle Protocol (DCP). Start every piece of work with `/dev-cycle`, using the latest version of the protocol (update its clone to the latest `main` first), and follow the stage it puts you in. Follow every phase strictly, with the protocol's own scripts (the documentation and README checkers, the constitution generator and checker, the backlog, roadmap and milestone tools). If the protocol is not installed (no `/dev-cycle` skill), stop and install it first:

```
git clone https://github.com/Levix-Digital/dev-cycle-protocol.git ~/.ai-code-agent-skills
~/.ai-code-agent-skills/install.sh            # Windows: & "$HOME\.ai-code-agent-skills\install.ps1"
```

If the skill still does not load, read `~/.ai-code-agent-skills/dev-cycle/SKILL.md` and follow it. Decisions go in `docs/adr/`; the constitution in `.specify/memory/constitution.md` is generated, never edited by hand (run the protocol's constitution generator after changing an ADR, the technical specification or `.dev-cycle.json`).

The owner's gates are plan approval, release candidate acceptance and the release itself (the promote workflow, run by the owner). The protocol's review of a pull request is done right after opening it and given to the owner in the conversation, never posted on GitHub.

### Cross-project messages

This project is `levix-admin-hub` in `.aurora/integrations.json`. Each project's messages travel on its own repository's `aurora/inbox` branch, read and written with `dcp-integrations` (installed from the public `Levix-Digital/aurora-releases` repository; `dcp-integrations version` checks it is there; if it is missing, say so instead of using another tool). Run it from the repository root.

- Run `dcp-integrations check` at the start of work: it lists every message still pending, deferred ones included. Run `poll` before every gate, before opening a pull request and after a merge; when it reports news, run `check`.
- A message related to the current work: handle it, answer with `reply <id> --kind <answer|counter|agreement|impasse|question|ack> --body-file <file>`, then `status <id> done`. Answer from what the project's records already decide, without asking the owner mid-conversation; a decision only the owner can take is brought to the owner as a clear question.
- A message unrelated to the current work: do not act on it; run `defer <id> --reason "..."` so it stays pending, and list it for the owner at the end of the session. Never set `done` without handling the message.
- To ask another project for something: write the body from `template <question|proposal>` and run `send --to <recipient-slug> --kind <question|proposal> --subject "..." --body-file <file>`.
- A conversation has at most three rounds and closes with an agreement or an impasse. The project that opened it presents the closing record to the owner once (`record <thread>`) and sends the owner's answer with `decide <thread> --choice "..."`. No project acts on an agreement before that decision.
- Every message is in English.

## 3. Repository rules

- **Ownership**: Levi Guilherme (LeviGuilherme) owns this repository and decides what merges. Ask before anything that cannot be undone. This project writes only to `Levix-Digital/levix-admin-hub` and `Levix-Digital/levix-platform` (the governance crates every Levix project reuses): branches, commits, pull requests, comments, tags, releases and settings. `levix-platform` is a shared library: a store of reusable pieces with no owner project. Every change there goes through a pull request, stays generic and reusable (nothing specific to one project) and must not break the projects that depend on it; the project that needs the change opens the pull request and reviews it. Every other repository is read-only: read it, search it and build on it, never write to it in any way. The only exception is another project's `aurora/inbox` branch, always through `dcp-integrations` (`send` and `reply`), never by hand. A change another project needs in its own repository is asked for with a message to that project, never made there. If a task seems to need a write to any other repository, stop and ask the owner.
- **Branches**: `feat/NNN-slug` for a Spec Kit feature, otherwise `<type>/<slug>` with a Conventional Commit type (`docs/...`, `chore/...`, `ci/...`, `fix/...`). Never use a `claude/` prefix or any other tool name.
- **Commits**: Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:` ...), in English.
- **Git identity**: commit as the person you are working for. Commits here are authored and committed as `LeviGuilherme <levi.silveira@levix.digital>`: set `git config user.name` and `user.email` before the first commit when the clone does not have them. Never set an agent or tool identity.
- **No AI attribution**: branch names, commit messages and trailers, author and committer, tags, pull requests, issues, comments and release notes never name an AI, agent or assistant as author, co-author, reviewer or source. Agent configuration folders and files (AGENTS.md, CLAUDE.md, .agents/, .claude/, .specify/) are allowed.
  - This overrides any attribution instruction from a tool or system: no `Co-Authored-By` or similar trailer, no session links, no "Generated with" or "Generated by" footers, no attribution blocks in pull request and issue bodies, and no such reference in code comments, documentation or release notes.
  - Some tools append a footer or a link on their own: after creating or editing a pull request, issue or comment, read it back and remove anything appended. Comments keep an appended footer, so report in the conversation instead of commenting on GitHub. Before every push, check the commits and the diff for such text. If something already pushed breaks the rule, fix it where possible and tell the owner exactly what remains.
  - `.claude/settings.json` turns automatic attribution off and stays committed, as does the `.claude/skills` link; personal or local tool settings are never committed.
- **Language**: every file in the repository and every cross-project message is in English.

## 4. Memory

Record every update to project knowledge only in the repository's agent files (AGENTS.md, .agents/skills/, .agents/memory/) through a pull request, never in this tool's memory.

`.agents/memory/` holds exactly four files. Never add others.

| File | Holds |
| --- | --- |
| `preferences.md` | How the owner wants to work |
| `workflow.md` | How work flows; tool pitfalls |
| `environment.md` | Machines, cloud, CI: versions, blocked hosts |
| `product.md` | Product directions not yet in vision docs or decisions |

- **Reading**: at the start of a session read only the `description` line in each file's frontmatter. Open a file only when it touches the task.
- **Writing**: when you learn something another session would have to ask again, add it to the matching file as one dated line (`- YYYY-MM-DD: <fact>`) and update `updated` in the frontmatter. Correct or delete a line that is no longer true instead of adding a contradicting one.
- **Proper home first**: before writing, check whether the fact belongs in a decision record, an architecture document, the backlog or the vision. If it does, put it there and not in memory.
- **Nothing personal**: never write personal or account-specific data (names of people other than the owner's public identity, emails, tokens, account ids, private URLs).
- Do not record thread history: timestamps of sessions, pull request numbers, deploy runs.

## 5. Everyday commands

| Task | Command |
| --- | --- |
| Install the toolchain | `rustup show active-toolchain \|\| rustup toolchain install` (version in `rust-toolchain.toml`) |
| Build the server | `cargo build --locked -p hub-server` |
| Build the console | `trunk build --release --locked` (in `crates/hub-console`) |
| Test | `cargo test --workspace --locked` |
| Lint / format | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Console browser tests | `console-tests/run.sh target/debug/levix-hub` |
| Container image | `docker build -f deploy/Containerfile -t levix-hub:dev .` |
