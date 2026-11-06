//! One function per command. Each calls the public API and builds a [`Report`].

use std::io::BufRead;

use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::cli::{
    AdminCommand, AuditCommand, Command, Global, InstanceCommand, LinkCommand, LoginArgs,
    MeCommand, MemberCommand, OrgCommand, Secrets, SessionCommand, TokenCommand,
};
use crate::client::{Client, Target};
use crate::config::{self, Session};
use crate::error::{exit, CliError, CliResult};
use crate::output::{fields, table, text, Report};

/// The environment variable holding a personal API token; it replaces the saved session.
pub const TOKEN_VARIABLE: &str = "LEVIX_HUB_TOKEN";

/// What a command needs to reach the Hub.
pub struct Context {
    global: Global,
    session: Option<Session>,
}

impl Context {
    pub fn load(global: Global) -> CliResult<Self> {
        Ok(Self {
            global,
            session: config::load()?,
        })
    }

    /// The saved session, when it belongs to `url`.
    fn session_for(&self, url: &str) -> Option<&Session> {
        self.session.as_ref().filter(|s| s.url == url)
    }

    fn target(&self, hint: Option<&str>) -> CliResult<Target> {
        let url = self
            .global
            .url
            .as_deref()
            .or(hint)
            .or(self.session.as_ref().map(|s| s.url.as_str()))
            .ok_or_else(|| {
                CliError::Usage("no Hub address: pass --url or set LEVIX_HUB_URL".to_owned())
            })?;
        let mut target = Target::new(
            url,
            self.global.fingerprint.clone(),
            self.global.ca_file.clone(),
        )?;
        if target.fingerprint.is_none() && target.ca_file.is_none() {
            if let Some(session) = self.session_for(&target.url) {
                target.fingerprint = session.fingerprint.clone();
                target.ca_file = session.ca_file.clone();
            }
        }
        Ok(target)
    }

    /// A client without credentials.
    fn anonymous(&self, hint: Option<&str>) -> CliResult<Client> {
        Client::new(self.target(hint)?, None)
    }

    /// A client with the API token from the environment, or the saved session.
    fn signed_in(&self) -> CliResult<Client> {
        let target = self.target(None)?;
        let token = std::env::var(TOKEN_VARIABLE)
            .ok()
            .filter(|t| !t.trim().is_empty())
            .or_else(|| self.session_for(&target.url).map(|s| s.token.clone()))
            .ok_or_else(|| CliError::Api {
                status: 401,
                code: "unauthenticated".to_owned(),
                message: format!("not signed in to {}; run `levix-hub-cli login`", target.url),
            })?;
        Client::new(target, Some(token))
    }
}

pub async fn run(context: &Context, command: Command) -> CliResult<Report> {
    match command {
        Command::Login(args) => login(context, args).await,
        Command::Logout => logout(context).await,
        Command::Status => status(context).await,
        Command::Health => health(context).await,
        Command::Link(command) => link(context, command).await,
        Command::Instance(command) => instance(context, command).await,
        Command::Me(command) => me(context, command).await,
        Command::Session(command) => session(context, command).await,
        Command::Token(command) => token(context, command).await,
        Command::Org(command) => org(context, command).await,
        Command::Member(command) => member(context, command).await,
        Command::Admin(command) => admin(context, command).await,
        Command::Audit(command) => audit(context, command).await,
    }
}

// ---- secrets -------------------------------------------------------------------------------

fn stdin_lines(count: usize) -> CliResult<Vec<String>> {
    let stdin = std::io::stdin();
    let mut lines = Vec::with_capacity(count);
    for line in stdin.lock().lines().take(count) {
        lines.push(line?.trim_end_matches(['\r', '\n']).to_owned());
    }
    if lines.len() < count || lines.iter().any(String::is_empty) {
        return Err(CliError::Usage(format!(
            "expected {count} non-empty line(s) on standard input"
        )));
    }
    Ok(lines)
}

fn prompt(label: &str) -> CliResult<String> {
    let value = rpassword::prompt_password(label).map_err(|e| {
        CliError::Usage(format!(
            "cannot prompt ({e}); use --password-stdin and --code"
        ))
    })?;
    if value.is_empty() {
        return Err(CliError::Usage("an empty answer".to_owned()));
    }
    Ok(value)
}

/// The passwords named by `labels`: from standard input, or prompted.
fn passwords(secrets: &Secrets, labels: &[&str]) -> CliResult<Vec<String>> {
    if secrets.password_stdin {
        return stdin_lines(labels.len());
    }
    labels.iter().map(|label| prompt(label)).collect()
}

fn code(secrets: &Secrets) -> CliResult<String> {
    match &secrets.code {
        Some(code) => Ok(code.clone()),
        None => prompt("Authenticator code: "),
    }
}

/// A new password, asked twice when prompted.
fn new_password(secrets: &Secrets) -> CliResult<String> {
    if secrets.password_stdin {
        return Ok(stdin_lines(1)?.remove(0));
    }
    let first = prompt("New password: ")?;
    if prompt("New password again: ")? != first {
        return Err(CliError::Usage("the passwords differ".to_owned()));
    }
    Ok(first)
}

// ---- arguments -----------------------------------------------------------------------------

fn uuid(value: &str, what: &str) -> CliResult<String> {
    Uuid::parse_str(value)
        .map(|id| id.to_string())
        .map_err(|_| CliError::Usage(format!("{what} must be an id (UUID), got {value}")))
}

/// Splits a link as printed (`https://hub/#/link/<token>`) or a bare token.
pub fn parse_link(link: &str) -> (Option<&str>, &str) {
    match link.trim().split_once("/#/link/") {
        Some((base, token)) => (Some(base), token),
        None => (None, link.trim()),
    }
}

/// The console address of a link token.
pub fn link_url(base: &str, token: &str) -> String {
    format!("{}/#/link/{token}", base.trim_end_matches('/'))
}

fn not_found(message: String) -> CliError {
    CliError::Api {
        status: 404,
        code: "not_found".to_owned(),
        message,
    }
}

fn items(value: &Value) -> &[Value] {
    value
        .get("items")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// An organization id from an id or a slug.
async fn organization_id(client: &Client, org: &str) -> CliResult<String> {
    if let Ok(id) = Uuid::parse_str(org) {
        return Ok(id.to_string());
    }
    let list = client.get("/api/v1/organizations", &[]).await?;
    items(&list)
        .iter()
        .find(|o| o.get("slug").and_then(Value::as_str) == Some(org))
        .and_then(|o| o.get("id").and_then(Value::as_str))
        .map(str::to_owned)
        .ok_or_else(|| not_found(format!("no visible organization with slug {org}")))
}

/// A member id from an id or an e-mail.
async fn member_id(client: &Client, organization: &str, member: &str) -> CliResult<String> {
    if let Ok(id) = Uuid::parse_str(member) {
        return Ok(id.to_string());
    }
    let list = client
        .get(
            &format!("/api/v1/organizations/{organization}/members"),
            &[],
        )
        .await?;
    items(&list)
        .iter()
        .find(|m| {
            m.get("email")
                .and_then(Value::as_str)
                .is_some_and(|e| e.eq_ignore_ascii_case(member))
        })
        .and_then(|m| m.get("id").and_then(Value::as_str))
        .map(str::to_owned)
        .ok_or_else(|| not_found(format!("{member} is not a member of this organization")))
}

fn done(value: Value, message: &str) -> Report {
    Report::new(value, message.to_owned())
}

// ---- sign-in -------------------------------------------------------------------------------

/// Handles the answer of a login or a completed link: saves the session, or explains the
/// re-enrollment.
fn signed_in(client: &Client, email: String, mut body: Value) -> CliResult<Report> {
    let target = client.target().clone();
    match body.get("status").and_then(Value::as_str) {
        Some("signed_in") => {
            let token = body
                .get("token")
                .and_then(Value::as_str)
                .ok_or_else(|| CliError::Local("the Hub sent no session token".to_owned()))?
                .to_owned();
            let path = config::save(&Session {
                url: target.url.clone(),
                fingerprint: target.fingerprint.clone(),
                ca_file: target.ca_file.clone(),
                email: email.clone(),
                token,
            })?;
            if let Some(object) = body.as_object_mut() {
                object.insert("token".to_owned(), json!("(saved)"));
            }
            let expires = text(body.pointer("/session/expires_at"));
            let mut human = format!(
                "Signed in to {} as {email}; the session ends by {expires}.\nSession saved in {}.",
                target.url,
                path.display()
            );
            if let Some(codes) = body.get("recovery_codes").and_then(Value::as_array) {
                human.push_str(
                    "\n\nRecovery codes (shown once; keep them offline, each works once):\n",
                );
                for code in codes {
                    human.push_str(&format!("  {}\n", text(Some(code))));
                }
            }
            Ok(Report::new(body, human))
        }
        Some("must_reenroll") => {
            let token = text(body.pointer("/link/token"));
            let human = format!(
                "Your authenticator was reset: enroll it again with the link below (valid until {}).\n  {}\n\
                 Then run `levix-hub-cli link complete <link>`.",
                text(body.pointer("/link/expires_at")),
                link_url(&target.url, &token)
            );
            Ok(Report::new(body, human).with_exit(exit::FAILURE))
        }
        _ => Err(CliError::Local(format!(
            "unexpected sign-in answer: {body}"
        ))),
    }
}

async fn login(context: &Context, args: LoginArgs) -> CliResult<Report> {
    let client = context.anonymous(None)?;
    let password = passwords(&args.secrets, &["Password: "])?.remove(0);
    let code = code(&args.secrets)?;
    let body = client
        .post(
            "/api/v1/auth/login",
            Some(
                &json!({"email": args.email, "password": password, "code": code, "client": "cli"}),
            ),
        )
        .await?;
    signed_in(&client, args.email, body)
}

async fn logout(context: &Context) -> CliResult<Report> {
    let Some(session) = &context.session else {
        return Ok(done(Value::Null, "Not signed in."));
    };
    let target = Target::new(
        &session.url,
        session.fingerprint.clone(),
        session.ca_file.clone(),
    )?;
    let client = Client::new(target, Some(session.token.clone()))?;
    match client.post("/api/v1/auth/logout", None).await {
        Ok(_) | Err(CliError::Api { status: 401, .. }) => {}
        Err(error) => return Err(error),
    }
    config::remove()?;
    Ok(done(
        Value::Null,
        &format!("Signed out of {}.", session.url),
    ))
}

async fn status(context: &Context) -> CliResult<Report> {
    let client = context.signed_in()?;
    let me = client.get("/api/v1/me", &[]).await?;
    let human = format!(
        "Hub      {}\n{}",
        client.target().url,
        fields(
            &me,
            &["email", "display_name", "status", "is_instance_owner"]
        )
    );
    Ok(Report::new(
        json!({"url": client.target().url, "me": me}),
        human,
    ))
}

async fn health(context: &Context) -> CliResult<Report> {
    let client = context.anonymous(None)?;
    let live = client.get("/healthz", &[]).await?;
    let ready = client.get("/readyz", &[]).await?;
    let human = format!(
        "live   {}\nready  {}",
        text(live.get("status")),
        text(ready.get("status"))
    );
    Ok(Report::new(json!({"live": live, "ready": ready}), human))
}

async fn link(context: &Context, command: LinkCommand) -> CliResult<Report> {
    match command {
        LinkCommand::Inspect { link } => {
            let (base, token) = parse_link(&link);
            let client = context.anonymous(base)?;
            let body = client
                .post("/api/v1/auth/links/inspect", Some(&json!({"token": token})))
                .await?;
            let human = fields(&body, &["purpose", "email", "expires_at"]);
            Ok(Report::new(body, human))
        }
        LinkCommand::Enroll { link } => {
            let (base, token) = parse_link(&link);
            let client = context.anonymous(base)?;
            let body = client
                .post(
                    "/api/v1/auth/links/enrollment",
                    Some(&json!({"token": token})),
                )
                .await?;
            let human = format!(
                "Add this secret to your authenticator app (or open the URI):\n  secret  {}\n  uri     {}\n\
                 Then run `levix-hub-cli link complete <link> --code <code>`.",
                text(body.get("secret")),
                text(body.get("uri"))
            );
            Ok(Report::new(body, human))
        }
        LinkCommand::Complete {
            link,
            secrets,
            display_name,
        } => {
            let (base, token) = parse_link(&link);
            let client = context.anonymous(base)?;
            let info = client
                .post("/api/v1/auth/links/inspect", Some(&json!({"token": token})))
                .await?;
            let email = text(info.get("email"));
            let password = new_password(&secrets)?;
            let code = code(&secrets)?;
            let body = client
                .post(
                    "/api/v1/auth/links/complete",
                    Some(&json!({
                        "token": token,
                        "password": password,
                        "code": code,
                        "display_name": display_name,
                        "client": "cli",
                    })),
                )
                .await?;
            signed_in(&client, email, body)
        }
    }
}

// ---- instance and own account ----------------------------------------------------------------

fn instance_text(body: &Value) -> String {
    format!(
        "{}\nsession_inactivity_minutes  {}\nsession_absolute_hours      {}",
        fields(body, &["id", "initialized_at"]),
        text(body.pointer("/settings/session_inactivity_minutes")),
        text(body.pointer("/settings/session_absolute_hours"))
    )
}

async fn instance(context: &Context, command: InstanceCommand) -> CliResult<Report> {
    match command {
        InstanceCommand::Status => {
            let body = context
                .anonymous(None)?
                .get("/api/v1/instance", &[])
                .await?;
            let human = fields(&body, &["initialized"]);
            Ok(Report::new(body, human))
        }
        InstanceCommand::Show => {
            let body = context
                .signed_in()?
                .get("/api/v1/instance/settings", &[])
                .await?;
            let human = instance_text(&body);
            Ok(Report::new(body, human))
        }
        InstanceCommand::Set {
            inactivity_minutes,
            absolute_hours,
        } => {
            if inactivity_minutes.is_none() && absolute_hours.is_none() {
                return Err(CliError::Usage(
                    "pass --inactivity-minutes, --absolute-hours or both".to_owned(),
                ));
            }
            let client = context.signed_in()?;
            let current = client.get("/api/v1/instance/settings", &[]).await?;
            let mut settings = current
                .get("settings")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if let Some(minutes) = inactivity_minutes {
                settings["session_inactivity_minutes"] = json!(minutes);
            }
            if let Some(hours) = absolute_hours {
                settings["session_absolute_hours"] = json!(hours);
            }
            let body = client.put("/api/v1/instance/settings", &settings).await?;
            let human = instance_text(&body);
            Ok(Report::new(body, human))
        }
    }
}

const ME_FIELDS: &[&str] = &[
    "id",
    "email",
    "display_name",
    "locale",
    "status",
    "is_instance_owner",
    "second_factor_enrolled",
    "roles",
];

async fn me(context: &Context, command: MeCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    match command {
        MeCommand::Show => {
            let body = client.get("/api/v1/me", &[]).await?;
            let human = fields(&body, ME_FIELDS);
            Ok(Report::new(body, human))
        }
        MeCommand::Update {
            display_name,
            locale,
        } => {
            let mut changes = Map::new();
            if let Some(name) = display_name {
                changes.insert("display_name".to_owned(), json!(name));
            }
            if let Some(locale) = locale {
                changes.insert("locale".to_owned(), json!(locale));
            }
            if changes.is_empty() {
                return Err(CliError::Usage(
                    "pass --display-name, --locale or both".to_owned(),
                ));
            }
            let body = client.patch("/api/v1/me", &Value::Object(changes)).await?;
            let human = fields(&body, ME_FIELDS);
            Ok(Report::new(body, human))
        }
        MeCommand::Password { secrets } => {
            let (current, new) = if secrets.password_stdin {
                let mut lines = stdin_lines(2)?;
                let new = lines.remove(1);
                (lines.remove(0), new)
            } else {
                (prompt("Current password: ")?, new_password(&secrets)?)
            };
            let code = code(&secrets)?;
            let body = client
                .post(
                    "/api/v1/me/password",
                    Some(&json!({"current_password": current, "new_password": new, "code": code})),
                )
                .await?;
            Ok(done(body, "Password changed; your other sessions ended."))
        }
        MeCommand::RecoveryCodes { secrets } => {
            let password = passwords(&secrets, &["Password: "])?.remove(0);
            let code = code(&secrets)?;
            let body = client
                .post(
                    "/api/v1/me/recovery-codes",
                    Some(&json!({"password": password, "code": code})),
                )
                .await?;
            let mut human =
                "New recovery codes (shown once; the old ones no longer work):\n".to_owned();
            for code in body
                .get("recovery_codes")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
            {
                human.push_str(&format!("  {}\n", text(Some(code))));
            }
            Ok(Report::new(body, human.trim_end().to_owned()))
        }
    }
}

async fn session(context: &Context, command: SessionCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    match command {
        SessionCommand::List => {
            let body = client.get("/api/v1/me/sessions", &[]).await?;
            let human = table(
                &body,
                &[
                    "id",
                    "kind",
                    "current",
                    "last_seen_at",
                    "expires_at",
                    "source_address",
                ],
            );
            Ok(Report::new(body, human))
        }
        SessionCommand::Revoke { id } => {
            let id = uuid(&id, "the session")?;
            let body = client.delete(&format!("/api/v1/me/sessions/{id}")).await?;
            Ok(done(body, "Session ended."))
        }
    }
}

async fn token(context: &Context, command: TokenCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    match command {
        TokenCommand::Create { name, days } => {
            let body = client
                .post(
                    "/api/v1/me/tokens",
                    Some(&json!({"name": name, "days": days})),
                )
                .await?;
            let human = format!(
                "Token {} created, valid until {}.\nSecret (shown once; use it as {TOKEN_VARIABLE}):\n  {}",
                text(body.pointer("/token/id")),
                text(body.pointer("/token/expires_at")),
                text(body.get("secret"))
            );
            Ok(Report::new(body, human))
        }
        TokenCommand::List => {
            let body = client.get("/api/v1/me/tokens", &[]).await?;
            let human = table(
                &body,
                &["id", "name", "display_prefix", "created_at", "expires_at"],
            );
            Ok(Report::new(body, human))
        }
        TokenCommand::Revoke { id } => {
            let id = uuid(&id, "the token")?;
            let body = client.delete(&format!("/api/v1/me/tokens/{id}")).await?;
            Ok(done(body, "Token revoked."))
        }
    }
}

// ---- organizations and members -------------------------------------------------------------

const ORG_FIELDS: &[&str] = &["id", "slug", "name", "status", "created_at"];

async fn org(context: &Context, command: OrgCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    let body = match command {
        OrgCommand::List => {
            let body = client.get("/api/v1/organizations", &[]).await?;
            let human = table(&body, &["slug", "name", "status", "id"]);
            return Ok(Report::new(body, human));
        }
        OrgCommand::Create { slug, name } => {
            client
                .post(
                    "/api/v1/organizations",
                    Some(&json!({"slug": slug, "name": name})),
                )
                .await?
        }
        OrgCommand::Show { org } => {
            let id = organization_id(&client, &org).await?;
            client
                .get(&format!("/api/v1/organizations/{id}"), &[])
                .await?
        }
        OrgCommand::Rename { org, name } => {
            let id = organization_id(&client, &org).await?;
            client
                .patch(
                    &format!("/api/v1/organizations/{id}"),
                    &json!({"name": name}),
                )
                .await?
        }
        OrgCommand::Archive { org } => {
            let id = organization_id(&client, &org).await?;
            client
                .post(&format!("/api/v1/organizations/{id}/archive"), None)
                .await?
        }
    };
    let human = fields(&body, ORG_FIELDS);
    Ok(Report::new(body, human))
}

const MEMBER_FIELDS: &[&str] = &["id", "email", "display_name", "role", "status"];

async fn member(context: &Context, command: MemberCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    match command {
        MemberCommand::List { org } => {
            let id = organization_id(&client, &org).await?;
            let body = client
                .get(&format!("/api/v1/organizations/{id}/members"), &[])
                .await?;
            let human = table(&body, &["email", "display_name", "role", "status", "id"]);
            Ok(Report::new(body, human))
        }
        MemberCommand::Add { org, email, role } => {
            let id = organization_id(&client, &org).await?;
            let body = client
                .post(
                    &format!("/api/v1/organizations/{id}/members"),
                    Some(&json!({"email": email, "role": role.as_str()})),
                )
                .await?;
            let mut human = fields(&body["member"], MEMBER_FIELDS);
            if let Some(token) = body.pointer("/invitation/token").and_then(Value::as_str) {
                human.push_str(&format!(
                    "\n\nInvitation link (single use, valid until {}); deliver it only to {email}:\n  {}",
                    text(body.pointer("/invitation/expires_at")),
                    link_url(&client.target().url, token)
                ));
            }
            Ok(Report::new(body, human))
        }
        MemberCommand::SetRole { org, member, role } => {
            let id = organization_id(&client, &org).await?;
            let member = member_id(&client, &id, &member).await?;
            let body = client
                .put(
                    &format!("/api/v1/organizations/{id}/members/{member}"),
                    &json!({"role": role.as_str()}),
                )
                .await?;
            let human = fields(&body, MEMBER_FIELDS);
            Ok(Report::new(body, human))
        }
        MemberCommand::Remove { org, member } => {
            let id = organization_id(&client, &org).await?;
            let member = member_id(&client, &id, &member).await?;
            let body = client
                .delete(&format!("/api/v1/organizations/{id}/members/{member}"))
                .await?;
            Ok(done(body, "Member removed."))
        }
    }
}

async fn admin(context: &Context, command: AdminCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    let (id, action) = match &command {
        AdminCommand::Suspend { id } => (id, "suspend"),
        AdminCommand::Reactivate { id } => (id, "reactivate"),
        AdminCommand::ResetSecondFactor { id } => (id, "reset-second-factor"),
        AdminCommand::RevokeSessions { id } => (id, "revoke-sessions"),
    };
    let id = uuid(id, "the administrator")?;
    let body = client
        .post(&format!("/api/v1/administrators/{id}/{action}"), None)
        .await?;
    let human = match command {
        AdminCommand::ResetSecondFactor { .. } => format!(
            "Authenticator cleared and sessions ended. Re-enrollment link (single use, valid until {}):\n  {}",
            text(body.get("expires_at")),
            link_url(&client.target().url, &text(body.get("token")))
        ),
        AdminCommand::RevokeSessions { .. } => "All sessions of the administrator ended.".to_owned(),
        _ => fields(&body, &["id", "email", "status"]),
    };
    Ok(Report::new(body, human))
}

// ---- audit -----------------------------------------------------------------------------------

async fn audit(context: &Context, command: AuditCommand) -> CliResult<Report> {
    let client = context.signed_in()?;
    match command {
        AuditCommand::List {
            organization,
            mut before,
            limit,
            all,
        } => {
            let organization = match organization {
                Some(org) => Some(organization_id(&client, &org).await?),
                None => None,
            };
            let mut records = Vec::new();
            let next = loop {
                let mut query = Vec::new();
                if let Some(org) = &organization {
                    query.push(("organization", org.clone()));
                }
                if let Some(before) = before {
                    query.push(("before", before.to_string()));
                }
                if let Some(limit) = limit {
                    query.push(("limit", limit.to_string()));
                }
                let page = client.get("/api/v1/audit", &query).await?;
                records.extend(items(&page).iter().cloned());
                let next = page.get("next_before").and_then(Value::as_i64);
                match next {
                    Some(next) if all => before = Some(next),
                    _ => break next,
                }
            };
            let body = json!({"items": records, "next_before": next});
            let mut human = table(
                &body,
                &[
                    "seq",
                    "at",
                    "actor_kind",
                    "actor_id",
                    "action",
                    "result",
                    "target_id",
                ],
            );
            if let Some(next) = next {
                human.push_str(&format!("\n(more: --before {next}, or --all)"));
            }
            Ok(Report::new(body, human))
        }
        AuditCommand::Verify => {
            let body = client.get("/api/v1/audit/verify", &[]).await?;
            let intact = body.get("intact").and_then(Value::as_bool) == Some(true);
            let human = if intact {
                format!(
                    "The audit chain is intact ({} records).",
                    text(body.get("records"))
                )
            } else {
                format!(
                    "The audit chain is BROKEN at record {} ({} records).",
                    text(body.get("first_broken_seq")),
                    text(body.get("records"))
                )
            };
            let report = Report::new(body, human);
            Ok(if intact {
                report
            } else {
                report.with_exit(exit::FAILURE)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_split_into_address_and_token() {
        assert_eq!(
            parse_link("https://hub:8443/#/link/lvxl_abc"),
            (Some("https://hub:8443"), "lvxl_abc")
        );
        assert_eq!(parse_link(" lvxl_abc "), (None, "lvxl_abc"));
        assert_eq!(link_url("https://hub/", "t"), "https://hub/#/link/t");
    }
}
