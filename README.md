# Levix Admin Hub

The Levix Admin Hub is the control plane of the Levix products (AnyContext, DocMapper and
future ones). The customer always hosts it, and it is zero-knowledge: it governs installations
(administrators, organizations, and later users, devices, policies, licenses and updates) and
never sees content. It never calls home.

The current release is the foundation:

- **Organizations and administrators:** organizations, administrators with a role in each
  organization, and a TOTP second factor that every administrator must use.
- **Access:** console and CLI sessions, and personal API tokens.
- **Audit:** a hash-chained record of every administrative action.
- **Surfaces:** a web console built into the binary, an administration CLI, and a public,
  versioned API.

## Features

- **Claim from the host.** `levix-hub init` prints a single-use setup link for the instance
  owner and the fingerprint of the Hub's TLS certificate. Nobody can claim the Hub over the
  network first.
- **Organizations and roles.** The roles are owner, administrator, approver, auditor and
  read-only, held per organization. The permissions are in
  [`docs/contracts/permissions.md`](docs/contracts/permissions.md).
- **Strong sign-in.**
  - Every account needs a password and TOTP. Each account also gets 10 single-use recovery
    codes.
  - Accounts lock after repeated failures.
  - Sessions end after inactivity (15 minutes by default) and after an absolute limit
    (12 hours by default).
- **Administrative audit.** Every change and every refusal is recorded in an append-only hash
  chain, which can be checked from the console, the CLI or the host.
- **Web console** in English, French and Spanish, checked for accessibility (WCAG 2.2 AA) on
  every page.
- **CLI** (`levix-hub-cli`) with one command for each API operation, JSON output and stable
  exit codes.
- **Public contract.** The REST API is described in OpenAPI and the device channel in AsyncAPI
  ([`docs/contracts/`](docs/contracts/CHANGELOG.md)). Breaking changes are refused within a
  major contract version.

## Requirements

- A Linux host with Docker or Podman, on x86_64 or arm64. Plan for 2 vCPU and 4 GB of RAM; the
  Hub itself uses much less.
- One TCP port reachable by administrators (8443 by default).
- A persistent volume for `/var/lib/levix-hub`. It holds the database, the secret key and the
  generated certificate, so back it up.
- Optional: a TLS certificate and key from your own certificate authority. Without them, the Hub
  generates a self-signed certificate and prints its fingerprint.
- For the CLI: Linux, macOS or Windows.

## Installation

Download the release assets for your architecture from the
[releases page](https://github.com/Levix-Digital/levix-admin-hub/releases) and check them
against `SHA256SUMS.txt`:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
docker load < levix-hub-v0.1.0-linux-x86_64.oci.tar.gz      # or ...-linux-arm64...
```

Initialize the Hub once, on the host. Replace the e-mail and the host name with yours:

```sh
docker volume create levix-hub
docker run --rm -v levix-hub:/var/lib/levix-hub levix-hub:0.1.0 \
  init --owner-email it-admin@example.com --hostnames hub.example.com
```

The command prints three things:
- the SHA-256 fingerprint of the certificate;
- a single-use setup link, valid for 24 hours, such as
  `https://hub.example.com:8443/#/link/lvxl_...`;
- the time until which the link is valid.

Keep the link private: anyone who has it can set the owner's password.

Start the Hub:

```sh
docker run -d --name levix-hub --restart unless-stopped \
  -p 8443:8443 -v levix-hub:/var/lib/levix-hub levix-hub:0.1.0
```

It is ready when `curl -fk https://localhost:8443/readyz` answers `{"status":"ready"}`.

To use your own certificate, mount it and pass `--tls-cert /certs/hub.pem --tls-key
/certs/hub.key` to both `init` and the container command (`serve`), or set
`LEVIX_HUB_TLS_CERT` and `LEVIX_HUB_TLS_KEY`.

## Quick Start

1. Open the setup link in a browser. If the certificate was generated, first check that the
   browser shows the fingerprint that `init` printed.
2. Choose **Set up the authenticator** and scan the QR code with an authenticator app, or type
   the secret into it.
3. Enter your name, a password of at least 12 characters, and the 6-digit code from the app.
4. Write down the 10 recovery codes and keep them offline. They are shown only once.
5. Under **Organizations**, create your first organization.
6. Open it and add a member: enter their e-mail and choose a role. The console shows an
   invitation link. Send it to that person through a channel you trust; they set up their
   account with it in the same way.

The same steps from a terminal, with the CLI:

```sh
export LEVIX_HUB_URL=https://hub.example.com:8443
export LEVIX_HUB_FINGERPRINT=AB:CD:...          # as printed by init; omit with your own certificate
levix-hub-cli link enroll  'https://hub.example.com:8443/#/link/lvxl_...'
levix-hub-cli link complete 'https://hub.example.com:8443/#/link/lvxl_...' --display-name "IT Admin"
levix-hub-cli org create --slug acme --name "Acme"
levix-hub-cli member add acme --email colleague@example.com --role administrator
```

## Configuration

Every option of the server is also an environment variable, so a container needs no
configuration file.

| Option | Variable | Default | Meaning |
|---|---|---|---|
| `--data-dir` | `LEVIX_HUB_DATA_DIR` | `/var/lib/levix-hub` | Database, secret key and generated certificate |
| `--listen` | `LEVIX_HUB_LISTEN` | `0.0.0.0:8443` | Address and port |
| `--hostnames` | `LEVIX_HUB_HOSTNAMES` | `localhost` | Host names written into a generated certificate (comma separated) |
| `--tls-cert`, `--tls-key` | `LEVIX_HUB_TLS_CERT`, `LEVIX_HUB_TLS_KEY` | none | Your certificate chain and key (PEM) |
| `--public-url` | `LEVIX_HUB_PUBLIC_URL` | `https://<first host name>:8443` | Address used in printed links (`init`, `recover-owner`) |
| `--insecure-behind-proxy` | `LEVIX_HUB_INSECURE_BEHIND_PROXY` | off | Serve plain HTTP; only behind a proxy that terminates TLS |
| `--log` | `LEVIX_HUB_LOG` | `info` | Log filter; logs are JSON on standard output and never contain passwords, codes or tokens |

The instance owner changes the session limits in the console (**Instance**) or with
`levix-hub-cli instance set --inactivity-minutes 15 --absolute-hours 12`.

Host commands of the server, run with the same volume:

- `levix-hub migrate` applies pending database migrations and exits.
- `levix-hub audit verify` checks the hash chain of the whole audit.
- `levix-hub recover-owner --email <owner>` issues a recovery link for an instance owner who
  lost their password or authenticator.

## CLI

`levix-hub-cli` uses only the public API. It can do exactly what the console can.

- `levix-hub-cli --help` lists the commands. Each command has its own `--help`.
- **Trust:** the CLI checks the Hub's certificate against the system authorities, against a CA
  file (`--ca-file`), or against the fingerprint printed by `init` (`--fingerprint`).
- **Sign-in:** `login --email <e-mail>` prompts for the password and the code. The session is
  kept in a file that only your user can read, in your configuration directory, or in
  `LEVIX_HUB_CLI_CONFIG_DIR`.
- **Scripts:** create a personal token in the console or with `token create --name ci --days 30`,
  then set it in `LEVIX_HUB_TOKEN`. `--output json` prints the API's JSON.
- **Exit codes:**

  | Code | Meaning |
  |---|---|
  | 0 | ok |
  | 1 | failure |
  | 2 | usage |
  | 3 | not signed in |
  | 4 | not allowed |
  | 5 | not found |
  | 6 | conflict |
  | 7 | invalid input |
  | 8 | too many attempts |
  | 9 | cannot reach or trust the Hub |
  | 10 | Hub error |

## Troubleshooting

- **The browser warns about the certificate.** This is expected with the generated certificate.
  Compare the fingerprint the browser shows with the one printed by `init` (`docker logs
  levix-hub` also shows it). If they match, continue. Otherwise stop: you are not talking to
  your Hub. For a production deployment, use your own certificate.
- **The setup or invitation link says "Not found".** The link was already used, was replaced by a
  newer link for the same person, or expired (setup and host recovery links last 24 hours,
  invitations and resets 72 hours). Have the owner add the member again
  for a new invitation, or run `recover-owner` on the host for the instance owner.
- **"Too many attempts".** Sign-in and link pages accept 20 requests a minute from one address,
  and an account locks for a while after repeated wrong passwords or codes. Wait and try again.
  Behind a reverse proxy, every administrator shares the proxy's address and therefore that
  limit.
- **The CLI exits with code 9.** It cannot reach the Hub, or it does not trust the certificate.
  Pass `--fingerprint` with the value from `init`, or `--ca-file` with your authority's
  certificate.
- **The console signs out by itself.** Sessions end after the inactivity limit (the console
  warns one minute before) and after the absolute limit. Changing the password also ends your
  other sessions.
- **Lost authenticator.** Sign in with a recovery code instead of the TOTP code. If you have no
  recovery code left, an owner of your organizations can reset your authenticator; for the
  instance owner, run `levix-hub recover-owner` on the host.
- **`readyz` fails.** Check the logs: the database or the key file in the data directory cannot
  be read or written, usually because the volume belongs to another user. The container runs as
  `nonroot` (UID 65532): a named volume works as is, a host directory needs
  `chown -R 65532:65532 <directory>`.

## Documentation

- Architecture: [`TECDOC.md`](TECDOC.md) and [`docs/architecture/`](docs/architecture/)
- Contracts: [`docs/contracts/`](docs/contracts/CHANGELOG.md) (OpenAPI, channel AsyncAPI,
  permission matrix)
- Security: [`docs/security/threat-model.md`](docs/security/threat-model.md) and
  [`docs/security/security-architecture.md`](docs/security/security-architecture.md)
- Decisions: [`docs/adr/`](docs/adr/)
- Roadmap and backlog: [`docs/ROADMAP.md`](docs/ROADMAP.md), [`docs/BACKLOG.md`](docs/BACKLOG.md)
- Manual tests: [`tests/MANUAL_TESTS.md`](tests/MANUAL_TESTS.md)

## License

Private repository of Levix Digital. Proprietary software: customers with an active agreement
may receive the source code of the versions they run, under confidentiality, for audit only.
See [`LICENSE`](LICENSE).
