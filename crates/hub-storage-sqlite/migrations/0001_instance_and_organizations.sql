-- Schema 1: the instance and organizations.
-- Portable SQL (SQLite and PostgreSQL): text identifiers (UUID v7), RFC 3339 UTC times as text.

CREATE TABLE instance (
    id TEXT PRIMARY KEY NOT NULL,
    initialized_at TEXT NOT NULL,
    singleton INTEGER NOT NULL DEFAULT 1 UNIQUE CHECK (singleton = 1)
);

CREATE TABLE organization (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    created_at TEXT NOT NULL
);

CREATE INDEX organization_name ON organization (name);
