//! The OpenAPI document of the REST API, generated from the handlers and committed as
//! `docs/contracts/api/openapi.json` (ADR-0006). The contract has its own version.

use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::caller::{CSRF_HEADER, SESSION_COOKIE};
use crate::{administrators, audit, auth, health, instance, me, organizations};

/// Version of the public contract (not of the Hub): breaking changes need a new major.
pub const CONTRACT_VERSION: &str = "1.0.0";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Levix Admin Hub API",
        description = "Administration API of the Levix Admin Hub. Every request and response \
                       body refuses unknown fields. Errors use one object: \
                       `{\"error\": {\"code\", \"message\"}}`. Console sessions use the \
                       session cookie and send the CSRF token on writes; the CLI and \
                       automation use a bearer token (CLI session or personal API token). \
                       Permissions follow the published matrix (permissions.md).",
        license(name = "Proprietary, source available to customers"),
    ),
    paths(
        health::live,
        health::ready,
        instance::status,
        instance::settings,
        instance::update_settings,
        auth::login,
        auth::logout,
        auth::inspect_link,
        auth::begin_enrollment,
        auth::complete_link,
        me::get,
        me::update,
        me::change_password,
        me::regenerate_recovery_codes,
        me::sessions,
        me::revoke_session,
        me::tokens,
        me::create_token,
        me::revoke_token,
        organizations::list,
        organizations::create,
        organizations::get,
        organizations::rename,
        organizations::archive,
        organizations::members,
        organizations::add_member,
        organizations::change_role,
        organizations::remove_member,
        administrators::suspend,
        administrators::reactivate,
        administrators::reset_second_factor,
        administrators::revoke_sessions,
        audit::list,
        audit::verify,
    ),
    modifiers(&Security, &Version),
    tags(
        (name = "health", description = "Liveness and readiness"),
        (name = "instance", description = "The Hub installation"),
        (name = "auth", description = "Sign-in and credential links (limited per source address)"),
        (name = "me", description = "The caller's own account"),
        (name = "organizations", description = "Organizations"),
        (name = "members", description = "Roles of administrators in an organization"),
        (name = "administrators", description = "Account actions of owners"),
        (name = "audit", description = "Hash-chained administrative audit"),
    )
)]
struct ApiDoc;

struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "session_cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                SESSION_COOKIE,
                "Console session (HttpOnly, Secure, SameSite=Strict), set by sign-in",
            ))),
        );
        components.add_security_scheme(
            "csrf_token",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                CSRF_HEADER,
                "CSRF token of the console session, required on every write",
            ))),
        );
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some(
                        "CLI session token (lvxs_) or personal API token (lvxh_)",
                    ))
                    .build(),
            ),
        );
    }
}

struct Version;

impl Modify for Version {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        CONTRACT_VERSION.clone_into(&mut openapi.info.version);
    }
}

/// The document, as served at `/api/v1/openapi.json` and committed in the repository.
pub fn document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// The committed form: pretty JSON with a final newline.
pub fn document_json() -> String {
    let mut text = document()
        .to_pretty_json()
        .unwrap_or_else(|error| format!("{{\"error\": \"{error}\"}}"));
    text.push('\n');
    text
}
