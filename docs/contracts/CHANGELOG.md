# Contract changelog

Versions of the public contract of the Admin Hub (ADR-0006): the REST API ([api/openapi.json](api/openapi.json)), the device channel ([channel/v1/](channel/v1/README.md)) and the [permission matrix](permissions.md). The contract has its own semantic version, independent of the Hub's. A breaking change needs a new major version; CI compares the API with the last release and fails on a breaking change within one major version.

## 1.0.0 (Hub v0.1.0)

First version.

- **API** `/api/v1`: health and readiness, instance status and settings, sign-in and sign-out, credential links (inspect, TOTP enrollment, completion), the caller's profile, password, recovery codes, sessions and API tokens, organizations and their members, account actions of owners, the administrative audit and its verification. The document is served by every Hub at `/api/v1/openapi.json`.
- **Authentication**: console session cookie `__Host-lvx_session` with the `X-CSRF-Token` header on writes; bearer tokens for CLI sessions (`lvxs_`) and personal API tokens (`lvxh_`).
- **Errors**: one object, `{"error": {"code", "message"}}`, with the codes `invalid`, `bad_request`, `unsupported_media_type`, `not_found`, `conflict`, `not_initialized`, `already_initialized`, `unauthenticated`, `forbidden`, `throttled` (with `Retry-After`), `not_ready` and `internal`.
- **Zero knowledge**: request bodies refuse unknown fields.
- **Device channel** v1 (served from Hub v0.2.0): envelope, `hello`, `challenge`, `authenticate`, `welcome` and `error`; Ed25519 nonce handshake; Hub signature over the JCS form of the envelope; version negotiation.
- **Permission matrix** for the roles owner, administrator, approver, auditor and read-only, plus the instance owner.
