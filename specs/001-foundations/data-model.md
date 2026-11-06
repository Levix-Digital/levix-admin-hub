# Data Model: Foundations v0.1.0

One SQLite database per Hub (`hub.db`); migrations in `crates/hub-storage-sqlite/migrations/`, portable SQL, additive only. Identifiers are UUID v7 stored as text; times are UTC, RFC 3339 text.

| Entity | Fields | Rules |
|---|---|---|
| `instance` | id, initialized_at, schema_version, settings (session limits) | Exactly one row; created by `init`. |
| `organization` | id, slug (unique), name, status (`active`, `archived`), created_at | Archived organizations are read-only; never deleted in v0.1.0. |
| `administrator` | id, email (unique, case-folded), display_name, status (`invited`, `active`, `suspended`), password_hash (Argon2id PHC string), totp_secret_enc, totp_confirmed_at, locale, is_instance_owner, failed_attempts, locked_until, created_at | No action allowed until `totp_confirmed_at` is set. At least one active instance owner. |
| `recovery_code` | id, administrator_id, code_hash, used_at | 10 per enrollment; regenerated replaces all. |
| `role_assignment` | administrator_id, organization_id, role (`owner`, `administrator`, `approver`, `auditor`, `read_only`), granted_by, granted_at | One role per administrator per organization. At least one owner per active organization. |
| `credential_link` | id, administrator_id, purpose (`setup`, `invitation`, `recovery`), token_hash, expires_at, used_at, created_by | Single use; setup 24 h, invitation 72 h. |
| `session` | id, administrator_id, kind (`console`, `cli`), token_hash, csrf_hash, created_at, last_seen_at, expires_at, source_address, user_agent | Inactivity and absolute expiry enforced on every request. |
| `api_token` | id, administrator_id, name, prefix, token_hash, created_at, expires_at (≤ 90 days), last_used_at, revoked_at | Permissions = creator's current permissions at use time. |
| `audit_record` | seq (monotonic), id, at, actor_kind (`administrator`, `api_token`, `host`, `anonymous`), actor_id, organization_id, action, target_kind, target_id, result (`success`, `denied`, `failed`), source_address, details (JSON, metadata only), prev_hash, hash | Insert only; `hash = SHA-256(prev_hash ‖ canonical record)`; a verification command checks the chain. |

State transitions:

- Administrator: `invited` → (invitation accepted, password set) → TOTP pending → `active` ⇄ `suspended`.
- Organization: `active` → `archived` (instance owner or organization owner).
- Credential link: created → used | expired (both final).
