# Service and storage

The `levix-hub` binary: one Rust process (crate `hub-server`) that serves the API and the web
console over TLS, and keeps its state in one SQLite database in the data directory. It is the
compact installation profile of
[ADR-0003](../adr/0003-one-codebase-two-installation-profiles-sqlite-and-in-process.md); the
scalable profile (PostgreSQL, Redis, replicas) adds adapters behind the same ports later.

## Crates

| Crate | Role |
|---|---|
| `hub-core` | Domain types, use cases (methods of `Hub`) and the ports they need. Depends on no adapter ([ADR-0001](../adr/0001-hexagonal-architecture-with-a-modular-core.md)). |
| `hub-crypto` | Adapters of the crypto ports: Argon2id passwords, XChaCha20-Poly1305 secret box, TOTP, random tokens, the key file. |
| `hub-storage-sqlite` | Adapter of the storage ports on SQLite through `sqlx`. |
| `hub-api` | Axum router: the REST API, its OpenAPI document, request limits, and the embedded console. |
| `hub-server` | The binary: command line, TLS, logging, readiness, host commands. |

## Host commands

- `serve` (the container's default) opens the database, applies migrations, loads the key file
  and the certificate, and listens (default `0.0.0.0:8443`). It logs one JSON line with the
  startup time and the certificate fingerprint when ready.
- `init` creates the data directory, the key file and, when no certificate is given, a
  self-signed certificate for `--hostnames`; it records the instance and prints the
  fingerprint and the owner's single-use setup link. It refuses an instance that is already
  initialized.
- `recover-owner --email` prints a recovery link (24 hours) for an instance owner.
- `migrate` applies pending migrations and exits.
- `audit verify` checks the hash chain of the whole audit.

Every host command is recorded in the audit with the actor `host`.

## Data directory

| Path | Content |
|---|---|
| `hub.db` (with `-wal`, `-shm`) | The database, in WAL mode with a busy timeout. |
| `keys/secret.key` | The key that encrypts TOTP secrets, created with owner-only permissions. |
| `tls/certificate.pem`, `tls/private-key.pem` | The generated certificate and key, when the customer gives none. |

Backing up the Hub means backing up the whole directory while it is stopped, or the database
with SQLite's online backup. Losing `keys/secret.key` makes every authenticator unusable.

## Storage rules

- Migrations are embedded in the binary and additive. A database whose schema version is newer
  than the build's is refused at startup with an explicit message (threat model T-25).
- Writes are serialized in the process. Every state change is committed in the same
  transaction as its audit record, so an action is never stored without its record.
- Secrets are never stored in clear: passwords as Argon2id, tokens, links and recovery codes as
  SHA-256 hashes, TOTP secrets encrypted (T-20).

## HTTP service

- TLS 1.2 or later through rustls. `--insecure-behind-proxy` serves plain HTTP for a proxy
  that terminates TLS and logs a warning at every start.
- Body size limits, a request timeout, and a limit of 20 requests a minute per source address
  on the unauthenticated endpoints (sign-in and links). Behind a proxy, that address is the
  proxy's (T-16, IMP-001).
- `GET /healthz` answers while the process runs; `GET /readyz` also checks that the database
  and the key file can be used, so an orchestrator stops sending traffic to a broken volume
  (T-27).
- Logs are JSON on standard output, filtered by `--log`. A test runs a whole session and checks
  that no password, code, token or secret reaches the logs (T-24).

## Packaging

`deploy/Containerfile` builds the console with `trunk`, then the binary, into a distroless image
that runs as `nonroot`, exposes 8443 and declares the volume `/var/lib/levix-hub`. The release
workflow builds it for x86_64 and arm64 and attaches it to the GitHub release as an OCI archive,
with a CycloneDX SBOM and SHA-256 sums.
