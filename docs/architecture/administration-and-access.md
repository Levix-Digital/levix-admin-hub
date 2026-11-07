# Administration and access

Who may govern the Hub, how they prove it, and how every action is recorded. The use cases live
in `hub-core` and are the same for the console, the CLI and any API client. The controls are
described in [security-architecture.md](../security/security-architecture.md); the threats they
answer (T-01 to T-16, T-20 to T-27) in [threat-model.md](../security/threat-model.md).

## Instance and organizations

- An instance has one or more **instance owners**. The first comes from `levix-hub init`; nobody
  can create an account over the network before that (T-01).
- Instance owners create **organizations** (a slug and a name) and change the instance settings:
  the inactivity limit (default 15 minutes) and the absolute session limit (default 12 hours).
- An administrator holds one **role per organization**: owner, administrator, approver, auditor
  or read-only. The [permission matrix](../contracts/permissions.md) maps each role to its
  actions; the core checks it on every request against the organization the request targets
  (T-08).
- Only owners change roles and add or remove members. The last active owner of an organization
  cannot be removed or demoted, and the last active instance owner cannot be suspended (T-10).

## Credentials

- **Links.** An administrator gets credentials only through a single-use link (`lvxl_...`): the
  setup link from `init` (24 hours), an invitation when an owner adds a member (72 hours), a
  reset when an owner resets someone's authenticator (72 hours), or a host recovery link from
  `recover-owner` (24 hours). The Hub sends no e-mail: the person who issues the link delivers it
  through a channel they trust. A newer link for the same person replaces the older one.
- **Enrollment.** With a link, the administrator sets a display name, a password (12 to 256
  characters, not a common password) and a TOTP authenticator, confirmed with a first code, and
  receives 10 single-use recovery codes shown once.
- **Sign-in** needs the password and a TOTP code (or a recovery code). Each TOTP time step is
  accepted once per account (T-06). Failed attempts add increasing delays per account and lock
  it for a while (T-02).
- **Password change** ends the administrator's other sessions.

## Sessions and tokens

| Credential | Used by | Lifetime | Stored as |
|---|---|---|---|
| Console session (`__Host-lvx_session` cookie, `HttpOnly`, `Secure`, `SameSite=Strict`) and its CSRF token | Browser | Inactivity and absolute limits of the instance | SHA-256 hash |
| CLI session (`lvxs_...` bearer token) | CLI | Same limits | SHA-256 hash |
| Personal API token (`lvxh_...`) | Scripts and integrations | Chosen at creation, at most 90 days; revocable | SHA-256 hash |

A request is authorized with the caller's role at the time of the request, so a token never
outlives or exceeds its creator's rights, and suspending an administrator stops their sessions
and tokens (T-09). Every state-changing console request carries the CSRF token (T-05).

## Audit

- Every administrative and authentication event, accepted or refused, is one audit record:
  time, actor (an administrator or `host`), organization, action, target, result and source
  address. Records carry metadata only.
- Records are chained by SHA-256: each record's hash covers the previous one. `levix-hub audit
  verify` on the host, and the verify operation of the API, report the first broken link (T-22).
- Owners and auditors of an organization read and verify its audit; the instance owner holds
  every right of an organization owner in every organization.
- Account actions (suspend, reactivate, reset the second factor, revoke sessions) apply to
  another administrator only when the caller owns every organization that administrator
  belongs to; nobody uses them on their own account.
