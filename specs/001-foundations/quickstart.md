# Quickstart: Foundations v0.1.0

How the release is checked end to end once built.

1. Start the Hub: `podman run -d --name hub -p 8443:8443 -v hub-data:/var/lib/levix-hub <the levix-hub 0.1.0 image from the release>`.
2. Claim it from the host: `podman exec hub levix-hub init`. It prints the certificate fingerprint and a single-use setup link valid 24 hours.
3. Open the link, check the fingerprint in the browser, set the password, scan the TOTP code with an authenticator app, confirm a code, save the recovery codes, create the first organization.
4. Invite a second administrator with the auditor role; open the invitation in another browser profile, set the password and TOTP; check that every change is refused and the audit is readable.
5. On a workstation: `levix-hub-cli login --url https://hub.example:8443 --fingerprint <sha256>`, then `levix-hub-cli org list --output json` and `levix-hub-cli audit list`.
6. On the host: `podman exec hub levix-hub audit verify` reports an intact chain.
