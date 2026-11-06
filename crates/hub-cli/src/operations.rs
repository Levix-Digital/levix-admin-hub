//! Every operation of the public API and the command that calls it (SC-004: the CLI covers
//! the whole API). A test compares this table with the committed OpenAPI document and with
//! the command tree.

/// `(method, path, command)`.
pub const OPERATIONS: &[(&str, &str, &str)] = &[
    ("get", "/healthz", "health"),
    ("get", "/readyz", "health"),
    ("get", "/api/v1/instance", "instance status"),
    ("get", "/api/v1/instance/settings", "instance show"),
    ("put", "/api/v1/instance/settings", "instance set"),
    ("post", "/api/v1/auth/login", "login"),
    ("post", "/api/v1/auth/logout", "logout"),
    ("post", "/api/v1/auth/links/inspect", "link inspect"),
    ("post", "/api/v1/auth/links/enrollment", "link enroll"),
    ("post", "/api/v1/auth/links/complete", "link complete"),
    ("get", "/api/v1/me", "me show"),
    ("patch", "/api/v1/me", "me update"),
    ("post", "/api/v1/me/password", "me password"),
    ("post", "/api/v1/me/recovery-codes", "me recovery-codes"),
    ("get", "/api/v1/me/sessions", "session list"),
    ("delete", "/api/v1/me/sessions/{id}", "session revoke"),
    ("get", "/api/v1/me/tokens", "token list"),
    ("post", "/api/v1/me/tokens", "token create"),
    ("delete", "/api/v1/me/tokens/{id}", "token revoke"),
    ("get", "/api/v1/organizations", "org list"),
    ("post", "/api/v1/organizations", "org create"),
    ("get", "/api/v1/organizations/{id}", "org show"),
    ("patch", "/api/v1/organizations/{id}", "org rename"),
    ("post", "/api/v1/organizations/{id}/archive", "org archive"),
    ("get", "/api/v1/organizations/{id}/members", "member list"),
    ("post", "/api/v1/organizations/{id}/members", "member add"),
    (
        "put",
        "/api/v1/organizations/{id}/members/{administrator}",
        "member set-role",
    ),
    (
        "delete",
        "/api/v1/organizations/{id}/members/{administrator}",
        "member remove",
    ),
    (
        "post",
        "/api/v1/administrators/{id}/suspend",
        "admin suspend",
    ),
    (
        "post",
        "/api/v1/administrators/{id}/reactivate",
        "admin reactivate",
    ),
    (
        "post",
        "/api/v1/administrators/{id}/reset-second-factor",
        "admin reset-second-factor",
    ),
    (
        "post",
        "/api/v1/administrators/{id}/revoke-sessions",
        "admin revoke-sessions",
    ),
    ("get", "/api/v1/audit", "audit list"),
    ("get", "/api/v1/audit/verify", "audit verify"),
];
