## Shell
app-name = Levix Admin Hub
skip-to-content = Skip to content
loading = Loading…
language = Language
dismiss = Dismiss
sign-out = Sign out
nav-label = Main
nav-organizations = Organizations
nav-audit = Audit
nav-profile = Profile
nav-sessions = Sessions
nav-tokens = API tokens
nav-instance = Instance
save = Save
saved = Saved.

## Notices
notice-session-ended = Your session ended. Sign in again.
notice-signed-out = You are signed out.
notice-inactivity = You were signed out after a period of inactivity.
notice-reenroll = Your authenticator was reset. Set it up again to sign in.

## Inactivity
idle-title = Still there?
idle-text = You will be signed out in one minute because of inactivity.
idle-stay = Stay signed in

## Not initialized and not found
not-initialized-title = This Hub is not initialized yet
not-initialized-text = Initialize it once on the host where it runs:
not-initialized-next = The command prints the certificate fingerprint and a setup link for the instance owner. Open that link here.
not-found-title = Page not found
not-found-home = Go to the start page

## Sign-in
login-title = Sign in
login-submit = Sign in
field-email = E-mail
field-password = Password
field-code = Authenticator code
field-code-help = The 6-digit code of your authenticator app, or one of your recovery codes.

## Links
link-title = Set up your account
link-intro = This { $purpose } link is for { $email } and is valid until { $expires }.
purpose-setup = setup
purpose-invitation = invitation
purpose-recovery = recovery
enroll-intro = First, add the Hub to your authenticator app.
enroll-start = Set up the authenticator
enroll-title = Authenticator
enroll-scan = Scan this code with your authenticator app.
enroll-qr-alt = QR code of the authenticator secret
enroll-manual = Or enter this secret by hand:
field-display-name = Display name
field-new-password = New password
field-password-help = At least 12 characters. Common passwords are refused.
field-confirm-password = Repeat the new password
link-submit = Save and sign in
link-deliver-help = Deliver this single-use link only to its holder; anyone with it can set the password.

## Recovery codes
codes-title = Recovery codes
codes-text = Keep these codes offline. Each one works once, in place of an authenticator code. They are shown only now.
codes-continue = I saved them, continue

## Organizations
orgs-title = Organizations
orgs-empty = You do not belong to any organization yet.
orgs-create-title = Create an organization
orgs-create = Create
field-org-name = Name
field-slug = Short name
field-slug-help = Lowercase letters, digits and hyphens, for example acme-corp.
col-name = Name
col-slug = Short name
col-status = Status
col-your-role = Your role
col-created = Created
org-status-active = Active
org-status-archived = Archived
role-instance-owner = Instance owner

## One organization
org-title = Organization
members-title = Members
members-add-title = Add a member
members-add = Add
org-invitation-issued = { $email } was invited. Send them this link:
col-administrator = Administrator
col-role = Role
col-actions = Actions
member-role-for = Role of { $email }
member-change-role = Change role
member-remove = Remove
member-remove-confirm = Remove { $email } from this organization?
account-suspend = Suspend
account-reactivate = Reactivate
account-reset-second-factor = Reset authenticator
account-revoke-sessions = End sessions
account-suspend-confirm = Suspend { $email }? Their sessions end.
account-reactivate-confirm = Reactivate { $email }?
account-reset-second-factor-confirm = Reset the authenticator of { $email }? Their sessions end and they must set it up again.
account-revoke-sessions-confirm = End all sessions of { $email }?
account-reset-issued = The authenticator of { $email } was reset. Send them this link:
org-manage-title = Manage the organization
field-new-org-name = New name
org-rename = Rename
org-archive = Archive the organization
org-archive-confirm = Archive this organization?
role-owner = Owner
role-administrator = Administrator
role-approver = Approver
role-auditor = Auditor
role-read-only = Read-only
admin-status-invited = Invited
admin-status-active = Active
admin-status-suspended = Suspended

## Profile
profile-title = Profile
profile-details = Details
password-title = Password
password-text = Changing the password ends your other sessions.
field-current-password = Current password
password-submit = Change the password
password-changed = The password was changed.
codes-regenerate-title = New recovery codes
codes-regenerate-text = New codes replace the old ones, which stop working.
codes-regenerate = Issue new codes

## Sessions
sessions-title = Sessions
col-kind = Kind
col-last-seen = Last activity
col-expires = Expires
col-source = Source address
session-kind-console = Console
session-kind-cli = Command line
session-current = (this session)
session-revoke = End

## Tokens
tokens-title = API tokens
tokens-text = A token acts with your permissions, for scripts and integrations. It lasts at most 90 days.
field-token-name = Token name
field-token-days = Valid for (days)
tokens-create = Create a token
tokens-secret-once = Copy the token now; it is shown only once:
col-prefix = Starts with
token-revoke = Revoke

## Audit
audit-title = Audit
audit-filter = Organization
audit-all = All organizations
audit-verify = Verify the chain
audit-intact = The audit chain is intact ({ $count } records).
audit-broken = The audit chain is broken at record { $seq }.
audit-older = Older records
col-when = When
col-actor = Actor
col-action = Action
col-target = Target
col-result = Result
result-success = Success
result-denied = Denied
result-failed = Failed

## Instance
instance-title = Instance
instance-id = Instance id
instance-initialized = Initialized
instance-sessions = Session limits
field-inactivity = Sign out after this many minutes without activity (5 to 60)
field-absolute = End sessions this many hours after sign-in (1 to 24)

## Errors
error-invalid = Some information is not valid.
error-not-found = Not found, or not visible to you.
error-conflict = This conflicts with the current state.
error-not-initialized = The Hub is not initialized.
error-already-initialized = The Hub is already initialized.
error-unauthenticated = The sign-in failed, or your session ended.
error-forbidden = You are not allowed to do this.
error-throttled = Too many attempts. Wait a moment and try again.
error-internal = The Hub had an internal error; see its logs.
error-bad-request = The request was not understood.
error-network = The Hub cannot be reached.
error-reauthentication-failed = The password or the authenticator code is wrong.
error-password-mismatch = The two passwords differ.
error-unexpected = Something went wrong.
error-with-detail = { $message } ({ $detail })
