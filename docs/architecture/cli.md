# Administration CLI

`levix-hub-cli` (crate `hub-cli`) administers a Hub from a terminal or a script. It depends on no
Hub crate and talks only to the public API, so it can do exactly what the API allows.

## Commands

- One command for each API operation, grouped by resource: `login`, `logout`, `status`,
  `health`, `link`, `instance`, `me`, `session`, `token`, `org`, `member`, `admin`, `audit`. A test compares the
  command table with the committed OpenAPI document and with the command-line parser, so a new
  endpoint without a command fails CI.
- `--output human` (default) prints tables; `--output json` prints the API's JSON unchanged.
- `audit list --all` follows the pages of the audit until the end.

## Trust and connection

- The Hub address comes from `--url` or `LEVIX_HUB_URL`, else from the signed-in session.
- The certificate is checked against the system authorities, plus a CA file (`--ca-file`), or
  pinned by its SHA-256 fingerprint (`--fingerprint`), as printed by `levix-hub init` for a
  generated certificate (T-07). There is no option to skip verification.

## Credentials

- `login` prompts for the password and the code, or reads the password from standard input
  (`--password-stdin`). The session is kept in `session.json`, written with owner-only
  permissions in a private directory under the user's configuration directory, or in
  `LEVIX_HUB_CLI_CONFIG_DIR`. On Windows the file inherits the permissions of the user's
  profile directory.
- `LEVIX_HUB_TOKEN` (a personal API token) replaces the session for scripts.
- A password is never a command-line argument, so it never stays in the shell history; the
  authenticator code (`--code`) may be, since it is used once.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Other failure |
| 2 | Usage error |
| 3 | Not signed in (401) |
| 4 | Not allowed (403) |
| 5 | Not found (404) |
| 6 | Conflict (409) |
| 7 | Invalid input (400, 413, 415, 422) |
| 8 | Too many attempts (429) |
| 9 | Cannot reach or trust the Hub |
| 10 | Hub error (5xx) |
