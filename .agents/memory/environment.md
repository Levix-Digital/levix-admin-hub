---
name: environment
description: Machines, cloud and CI: versions, blocked hosts, setup quirks. Read when a command, install or network call fails or behaves differently between machines.
updated: 2026-11-14
---

- 2026-11-07: Builds can fill the disk of a cloud session; build with `CARGO_INCREMENTAL=0`.
- 2026-11-07: The GitHub GraphQL API is blocked in cloud sessions, so the protocol's release status check fails there; verify a release through the REST API instead.
- 2026-11-07: `dcp-integrations` is installed by the cloud environment's setup script; locally, install it from `Levix-Digital/aurora-releases`.
