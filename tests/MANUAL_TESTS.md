# Manual Tests

Curated manual test suite: one section per feature, each test describing how the software behaves **today**. Tests are edited in place when behavior changes and removed when they no longer apply. Only what cannot be automated is tested here: the release artifacts on real hosts, real browsers and authenticator apps, and the CLI on each operating system. The API, the CLI commands and the console flows are covered by the automated suites (`cargo test`, `console-tests/`).

How to record a run: in each test of the current run, mark exactly one box of the `Result` line with `x` and, for `Fail` or `Blocked`, describe what happened in `Notes`.

## Installation

### MT-INST-001: The release image loads, initializes and becomes ready on x86_64

- **Covers**: release assets, `deploy/Containerfile`, `levix-hub init`, `levix-hub serve`, `/readyz`
- **Prerequisites**: A Linux x86_64 host with Docker or Podman and no `levix-hub` volume; the release assets `levix-hub-<tag>-linux-x86_64.oci.tar.gz` and `SHA256SUMS.txt` downloaded into one directory.
- **Steps**:
  1. `sha256sum -c SHA256SUMS.txt --ignore-missing`
  2. `docker load < levix-hub-<tag>-linux-x86_64.oci.tar.gz`
  3. `docker volume create levix-hub`
  4. `docker run --rm -v levix-hub:/var/lib/levix-hub levix-hub:<version> init --owner-email owner@example.com --hostnames localhost`
  5. `docker run -d --name levix-hub -p 8443:8443 -v levix-hub:/var/lib/levix-hub levix-hub:<version>`
  6. `curl -fk https://localhost:8443/readyz`
  7. `docker logs levix-hub`
- **Expected**:
  - Step 1 prints `OK` for the image archive.
  - Step 4 prints a SHA-256 fingerprint, a single-use link `https://localhost:8443/#/link/lvxl_...` and its expiry 24 hours later.
  - Step 6 answers `{"status":"ready"}` within 10 seconds of step 5.
  - Step 7 shows JSON log lines, one of them with the same fingerprint as step 4, and none with the setup link.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INST-002: The release image runs on arm64

- **Covers**: arm64 release image
- **Prerequisites**: A Linux arm64 host (for example a Raspberry Pi 5 or an arm64 cloud instance) with Docker or Podman; the asset `levix-hub-<tag>-linux-arm64.oci.tar.gz`.
- **Steps**:
  1. Run the steps of MT-INST-001 with the arm64 archive.
- **Expected**:
  - The same results as MT-INST-001.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INST-003: The Hub serves the customer's certificate

- **Covers**: `--tls-cert`, `--tls-key`
- **Prerequisites**: A certificate chain and key in PEM for a host name that resolves to the test host, issued by an authority the test browser trusts; a new volume.
- **Steps**:
  1. `docker run --rm -v levix-hub2:/var/lib/levix-hub -v "$PWD/certs:/certs:ro" levix-hub:<version> init --owner-email owner@example.com --tls-cert /certs/hub.pem --tls-key /certs/hub.key --public-url https://<host>:8443`
  2. `docker run -d --name levix-hub2 -p 8443:8443 -v levix-hub2:/var/lib/levix-hub -v "$PWD/certs:/certs:ro" levix-hub:<version> serve --tls-cert /certs/hub.pem --tls-key /certs/hub.key`
  3. Open the printed link in the browser.
- **Expected**:
  - The browser shows no certificate warning and the setup page opens.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INST-004: Data survives removing and recreating the container

- **Covers**: data volume, migrations at startup
- **Prerequisites**: The Hub of MT-INST-001 with the owner set up (MT-CLAIM-001) and one organization.
- **Steps**:
  1. `docker rm -f levix-hub`
  2. Start the container again with the command of MT-INST-001, step 5.
  3. Sign in to the console.
- **Expected**:
  - Sign-in works with the same password and authenticator, and the organization is listed.
  - On the Audit page, **Verify** reports an intact chain.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Claiming the Hub

### MT-CLAIM-001: The owner claims the Hub with a real authenticator app

- **Covers**: console Link page, TOTP enrollment, recovery codes
- **Prerequisites**: The Hub of MT-INST-001 and its setup link; a phone with an authenticator app (for example Google Authenticator, Microsoft Authenticator or 1Password).
- **Steps**:
  1. Open the setup link in a desktop browser.
  2. Compare the certificate fingerprint the browser shows (certificate details, SHA-256) with the one printed by `init`, then accept the warning.
  3. Choose **Set up the authenticator** and scan the QR code with the phone.
  4. Enter a display name, a password of at least 12 characters and the code shown by the app.
  5. Save the recovery codes, then continue.
- **Expected**:
  - The fingerprints of step 2 are identical.
  - The app adds an entry named "Levix Admin Hub" with the owner's e-mail.
  - The code is accepted, 10 recovery codes are shown once, and the console opens on **Organizations** signed in as the owner.
  - Opening the setup link again shows that it is no longer valid.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-CLAIM-002: The host recovers a locked-out instance owner

- **Covers**: `levix-hub recover-owner`, recovery link
- **Prerequisites**: The Hub of MT-CLAIM-001; the owner signed out.
- **Steps**:
  1. `docker exec levix-hub /usr/local/bin/levix-hub recover-owner --email owner@example.com`
  2. Open the printed link, set up a new authenticator entry and a new password.
  3. Try to sign in with the old authenticator entry.
- **Expected**:
  - Step 1 prints a single-use link valid for 24 hours.
  - Step 2 ends signed in with new recovery codes.
  - Step 3 is refused; the Audit page lists the host recovery.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Console

### MT-CON-001: The console works in Firefox and Safari

- **Covers**: web console in browsers other than the Chromium of the automated suite
- **Prerequisites**: The Hub of MT-CLAIM-001; Firefox (latest) and Safari (latest, on macOS).
- **Steps**:
  1. In each browser, sign in with the password and an authenticator code.
  2. Create an organization, add a member with the role auditor, and copy the invitation link.
  3. Open the invitation link in a private window of the same browser and complete the enrollment with the authenticator app.
  4. Sign out.
- **Expected**:
  - Every step works as in MT-CLAIM-001, with no layout broken and no error in the browser console.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-CON-002: The texts read naturally in French and Spanish

- **Covers**: `locales/fr.ftl`, `locales/es.ftl`
- **Prerequisites**: The Hub of MT-CLAIM-001; a reviewer who reads French or Spanish.
- **Steps**:
  1. On **Profile**, choose Français, then visit every page, including an invitation link and the inactivity warning.
  2. Repeat with Español.
- **Expected**:
  - Every text is translated, grammatical and consistent; French uses French typography (non-breaking spaces before `:` `?` `!`).
  - No text is cut or overflows its button or table cell.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-CON-003: The console can be used with the keyboard and a screen reader

- **Covers**: WCAG 2.2 AA beyond the automated axe-core scan
- **Prerequisites**: The Hub of MT-CLAIM-001; NVDA on Windows or VoiceOver on macOS.
- **Steps**:
  1. Sign in, create an organization and add a member using only the keyboard.
  2. Repeat with the screen reader on.
- **Expected**:
  - The focus is always visible and follows the reading order; no step needs a mouse.
  - The screen reader announces every field, button, error and the inactivity warning.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## CLI

### MT-CLI-001: The CLI signs in with the fingerprint on Linux and macOS

- **Covers**: CLI release binaries `linux-x86_64`, `linux-arm64`, `macos-arm64`; fingerprint pinning; session file
- **Prerequisites**: The Hub of MT-CLAIM-001 reachable from the test machine; the CLI archive for the machine, checked against `SHA256SUMS.txt`.
- **Steps**:
  1. `./levix-hub-cli login --url https://<host>:8443 --email owner@example.com`
  2. `./levix-hub-cli login --url https://<host>:8443 --fingerprint <fingerprint from init> --email owner@example.com`
  3. `./levix-hub-cli org list`
  4. On Linux `ls -l ~/.config/levix-hub/session.json`; on macOS `ls -l ~/Library/Application\ Support/levix-hub/session.json`
- **Expected**:
  - Step 1 fails with exit code 9 and a message about the untrusted certificate.
  - Step 2 prompts for the password and the code, then reports the session.
  - Step 3 lists the organizations.
  - Step 4 shows `session.json` readable and writable by the user only (`-rw-------`).
  - On macOS, the binary runs after being allowed in System Settings, Privacy & Security, if Gatekeeper blocks it.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-CLI-002: The CLI works on Windows

- **Covers**: CLI release binary `windows-x86_64`
- **Prerequisites**: Windows 11; the Hub of MT-CLAIM-001 reachable; the zip archive extracted.
- **Steps**:
  1. In PowerShell: `.\levix-hub-cli.exe login --url https://<host>:8443 --fingerprint <fingerprint> --email owner@example.com`
  2. `.\levix-hub-cli.exe org list --output json`
  3. `Get-Acl "$env:APPDATA\levix-hub\session.json" | Format-List`
  4. `.\levix-hub-cli.exe logout`
- **Expected**:
  - Sign-in prompts without echoing the password, and the list is printed as JSON.
  - The session file is under the user's profile, and its access list grants no other ordinary user.
  - After logout, `org list` exits with code 3.
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Deployment behind a proxy

### MT-PROXY-001: The Hub works behind a TLS-terminating proxy

- **Covers**: `--insecure-behind-proxy`, console cookies through a proxy
- **Prerequisites**: A reverse proxy (for example Caddy or nginx) that terminates TLS for `https://<host>` and forwards to the Hub on plain HTTP; a new volume initialized with `--public-url https://<host>`.
- **Steps**:
  1. `docker run -d -p 8080:8443 -v levix-hub3:/var/lib/levix-hub levix-hub:<version> serve --insecure-behind-proxy`
  2. `docker logs <container>`
  3. Claim the Hub through `https://<host>` with the setup link, then sign out and sign in again.
- **Expected**:
  - Step 2 shows a warning that the Hub serves plain HTTP.
  - Step 3 works: the session cookie is accepted through the proxy.
  - The Audit page shows the proxy's address as the source of every event (known limit, IMP-001).
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:
