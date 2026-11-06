-- Schema 2: instance settings, administrators, roles, credential links, sessions, API tokens,
-- recovery codes and the hash-chained administrative audit.
-- Portable SQL (SQLite and PostgreSQL) except the append-only triggers, which each adapter
-- writes in its own dialect.

ALTER TABLE instance ADD COLUMN session_inactivity_minutes INTEGER NOT NULL DEFAULT 15;
ALTER TABLE instance ADD COLUMN session_absolute_hours INTEGER NOT NULL DEFAULT 12;

CREATE TABLE administrator (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('invited', 'active', 'suspended')),
    password_hash TEXT,
    totp_secret_sealed BLOB,
    totp_pending_sealed BLOB,
    totp_confirmed_at TEXT,
    totp_last_step INTEGER,
    locale TEXT NOT NULL,
    is_instance_owner INTEGER NOT NULL DEFAULT 0 CHECK (is_instance_owner IN (0, 1)),
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE role_assignment (
    administrator_id TEXT NOT NULL REFERENCES administrator (id),
    organization_id TEXT NOT NULL REFERENCES organization (id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'administrator', 'approver', 'auditor', 'read_only')),
    granted_by TEXT REFERENCES administrator (id),
    granted_at TEXT NOT NULL,
    PRIMARY KEY (administrator_id, organization_id)
);

CREATE INDEX role_assignment_organization ON role_assignment (organization_id);

CREATE TABLE credential_link (
    id TEXT PRIMARY KEY NOT NULL,
    administrator_id TEXT NOT NULL REFERENCES administrator (id),
    purpose TEXT NOT NULL CHECK (purpose IN ('setup', 'invitation', 'recovery')),
    token_hash TEXT NOT NULL UNIQUE,
    created_by TEXT REFERENCES administrator (id),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used_at TEXT
);

CREATE TABLE recovery_code (
    id TEXT PRIMARY KEY NOT NULL,
    administrator_id TEXT NOT NULL REFERENCES administrator (id),
    code_hash TEXT NOT NULL,
    used_at TEXT
);

CREATE INDEX recovery_code_administrator ON recovery_code (administrator_id);

CREATE TABLE session (
    id TEXT PRIMARY KEY NOT NULL,
    administrator_id TEXT NOT NULL REFERENCES administrator (id),
    kind TEXT NOT NULL CHECK (kind IN ('console', 'cli')),
    token_hash TEXT NOT NULL UNIQUE,
    csrf_hash TEXT,
    created_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    source_address TEXT,
    user_agent TEXT
);

CREATE INDEX session_administrator ON session (administrator_id);

CREATE TABLE api_token (
    id TEXT PRIMARY KEY NOT NULL,
    administrator_id TEXT NOT NULL REFERENCES administrator (id),
    name TEXT NOT NULL,
    display_prefix TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    last_used_at TEXT,
    revoked_at TEXT
);

CREATE INDEX api_token_administrator ON api_token (administrator_id);

CREATE TABLE audit_record (
    seq INTEGER PRIMARY KEY NOT NULL,
    id TEXT NOT NULL UNIQUE,
    at TEXT NOT NULL,
    actor_kind TEXT NOT NULL,
    actor_id TEXT,
    organization_id TEXT,
    action TEXT NOT NULL,
    target_kind TEXT,
    target_id TEXT,
    result TEXT NOT NULL CHECK (result IN ('success', 'denied', 'failed')),
    source_address TEXT,
    details TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL
);

CREATE INDEX audit_record_organization ON audit_record (organization_id, seq);

-- The audit is append-only: the database refuses changes and deletions.
CREATE TRIGGER audit_record_no_update BEFORE UPDATE ON audit_record
BEGIN
    SELECT RAISE(ABORT, 'audit records are append-only');
END;

CREATE TRIGGER audit_record_no_delete BEFORE DELETE ON audit_record
BEGIN
    SELECT RAISE(ABORT, 'audit records are append-only');
END;
