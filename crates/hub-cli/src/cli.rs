//! Command line of `levix-hub-cli`. Commands follow the public API one to one (see
//! `operations.rs`); nothing here reaches the Hub any other way.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "levix-hub-cli",
    version,
    about = "Administer a Levix Admin Hub over its API",
    after_help = "Exit codes: 0 ok, 1 failure, 2 usage, 3 not signed in, 4 not allowed, 5 not found, \
                  6 conflict, 7 invalid input, 8 too many attempts, 9 cannot reach the Hub, 10 Hub error."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Args)]
pub struct Global {
    /// Hub address, for example https://hub.example.com:8443 (default: the signed-in Hub).
    #[arg(long, env = "LEVIX_HUB_URL", global = true)]
    pub url: Option<String>,

    /// Trust exactly the certificate with this SHA-256 fingerprint, as printed by
    /// `levix-hub init` (for the Hub's generated certificate).
    #[arg(
        long,
        env = "LEVIX_HUB_FINGERPRINT",
        global = true,
        conflicts_with = "ca_file"
    )]
    pub fingerprint: Option<String>,

    /// Also trust the certificate authorities in this PEM file.
    #[arg(long, env = "LEVIX_HUB_CA_FILE", global = true)]
    pub ca_file: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Human, global = true)]
    pub output: Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sign in and keep the session for the next commands.
    Login(LoginArgs),
    /// End the session on the Hub and forget it here.
    Logout,
    /// Show the signed-in Hub and account.
    Status,
    /// Check that the Hub is alive and ready.
    Health,
    /// Use a setup, invitation or recovery link.
    #[command(subcommand)]
    Link(LinkCommand),
    /// The instance and its session limits.
    #[command(subcommand)]
    Instance(InstanceCommand),
    /// Your own account.
    #[command(subcommand)]
    Me(MeCommand),
    /// Your sessions.
    #[command(subcommand)]
    Session(SessionCommand),
    /// Your personal API tokens.
    #[command(subcommand)]
    Token(TokenCommand),
    /// Organizations.
    #[command(subcommand)]
    Org(OrgCommand),
    /// Members of an organization and their roles.
    #[command(subcommand)]
    Member(MemberCommand),
    /// Administrator accounts.
    #[command(subcommand)]
    Admin(AdminCommand),
    /// The administrative audit.
    #[command(subcommand)]
    Audit(AuditCommand),
}

#[derive(Debug, Clone, Args)]
pub struct Secrets {
    /// Read the password from the first line of standard input instead of prompting.
    #[arg(long)]
    pub password_stdin: bool,

    /// The 6-digit authenticator code (or a recovery code); prompted when absent.
    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    #[arg(long)]
    pub email: String,

    #[command(flatten)]
    pub secrets: Secrets,
}

#[derive(Debug, Subcommand)]
pub enum LinkCommand {
    /// Show what a link is for and until when it is valid.
    Inspect { link: String },
    /// Start the authenticator enrollment of a link: prints the secret to add.
    Enroll { link: String },
    /// Set the password and the authenticator, then sign in.
    Complete {
        link: String,
        #[command(flatten)]
        secrets: Secrets,
        #[arg(long)]
        display_name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum InstanceCommand {
    /// Whether the instance is initialized (no sign-in needed).
    Status,
    /// The instance and its session limits.
    Show,
    /// Change the session limits (instance owner).
    Set {
        /// Minutes without activity before a session ends (5 to 60).
        #[arg(long)]
        inactivity_minutes: Option<u32>,
        /// Hours after sign-in before a session ends (1 to 24).
        #[arg(long)]
        absolute_hours: Option<u32>,
    },
}

#[derive(Debug, Subcommand)]
pub enum MeCommand {
    /// Your account and roles.
    Show,
    /// Change your display name or language.
    Update {
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        locale: Option<String>,
    },
    /// Change your password; other sessions end. With --password-stdin, the first line is
    /// the current password and the second the new one.
    Password {
        #[command(flatten)]
        secrets: Secrets,
    },
    /// Replace your recovery codes.
    RecoveryCodes {
        #[command(flatten)]
        secrets: Secrets,
    },
}

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// Your open sessions.
    List,
    /// End one of your sessions.
    Revoke { id: String },
}

#[derive(Debug, Subcommand)]
pub enum TokenCommand {
    /// Create a token; its secret is shown once.
    Create {
        #[arg(long)]
        name: String,
        /// Days until it expires.
        #[arg(long, default_value_t = 90)]
        days: i64,
    },
    /// Your tokens.
    List,
    /// Revoke a token.
    Revoke { id: String },
}

#[derive(Debug, Subcommand)]
pub enum OrgCommand {
    /// Organizations you can see.
    List,
    /// Create an organization (instance owner).
    Create {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: String,
    },
    /// One organization, by id or slug.
    Show { org: String },
    /// Rename an organization.
    Rename {
        org: String,
        #[arg(long)]
        name: String,
    },
    /// Archive an organization.
    Archive { org: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Role {
    Owner,
    Administrator,
    Approver,
    Auditor,
    /// Written as in the API.
    #[value(name = "read_only")]
    ReadOnly,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Administrator => "administrator",
            Self::Approver => "approver",
            Self::Auditor => "auditor",
            Self::ReadOnly => "read_only",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum MemberCommand {
    /// Members of an organization.
    List { org: String },
    /// Add an administrator by e-mail; a new one gets an invitation link.
    Add {
        org: String,
        #[arg(long)]
        email: String,
        #[arg(long, value_enum)]
        role: Role,
    },
    /// Change a member's role.
    SetRole {
        org: String,
        /// Member id or e-mail.
        member: String,
        #[arg(long, value_enum)]
        role: Role,
    },
    /// Remove a member from the organization.
    Remove { org: String, member: String },
}

#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Suspend an administrator; their sessions end.
    Suspend { id: String },
    /// Reactivate a suspended administrator.
    Reactivate { id: String },
    /// Clear an administrator's authenticator; they get a re-enrollment link at sign-in.
    ResetSecondFactor { id: String },
    /// End all sessions of an administrator.
    RevokeSessions { id: String },
}

#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Audit records, newest first.
    List {
        /// Only records of this organization (id or slug).
        #[arg(long)]
        organization: Option<String>,
        /// Records before this sequence number.
        #[arg(long)]
        before: Option<i64>,
        #[arg(long)]
        limit: Option<u32>,
        /// Follow the pages to the first record.
        #[arg(long)]
        all: bool,
    },
    /// Check the hash chain of the audit.
    Verify,
}
