//! Command line of `levix-hub`. Every option also reads an environment variable, so a
//! container is configured without a configuration file.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "levix-hub", version, about = "Levix Admin Hub")]
pub struct Cli {
    /// Directory holding the database, the key files and the generated certificate.
    #[arg(
        long,
        env = "LEVIX_HUB_DATA_DIR",
        default_value = "/var/lib/levix-hub",
        global = true
    )]
    pub data_dir: PathBuf,

    /// Log filter (for example `info` or `hub_server=debug,info`).
    #[arg(long, env = "LEVIX_HUB_LOG", default_value = "info", global = true)]
    pub log: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the Hub.
    Serve(ServeArgs),
    /// Initialize the Hub on this host (once): instance, secret key, certificate and the
    /// instance owner's setup link.
    Init(InitArgs),
    /// Issue a recovery link for the instance owner who lost their password or authenticator.
    RecoverOwner(RecoverArgs),
    /// Apply pending database migrations and exit.
    Migrate,
    /// Commands on the administrative audit.
    #[command(subcommand)]
    Audit(AuditCommand),
}

#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Check the hash chain of the whole audit.
    Verify,
}

#[derive(Debug, Clone, Args)]
pub struct PublicUrl {
    /// Address administrators open, used in printed links (default: the first host name on
    /// port 8443).
    #[arg(long, env = "LEVIX_HUB_PUBLIC_URL")]
    pub public_url: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// E-mail of the instance owner, who claims the Hub with the printed setup link.
    #[arg(long, env = "LEVIX_HUB_OWNER_EMAIL")]
    pub owner_email: String,

    #[command(flatten)]
    pub url: PublicUrl,

    #[command(flatten)]
    pub tls: TlsArgs,
}

impl InitArgs {
    pub fn public_url(&self) -> String {
        public_url(&self.url, &self.tls.hostnames)
    }
}

#[derive(Debug, Clone, Args)]
pub struct RecoverArgs {
    /// E-mail of the instance owner to recover.
    #[arg(long)]
    pub email: String,

    #[command(flatten)]
    pub url: PublicUrl,

    /// Host names of the generated certificate, used for the default public address.
    #[arg(
        long,
        env = "LEVIX_HUB_HOSTNAMES",
        value_delimiter = ',',
        default_value = "localhost"
    )]
    pub hostnames: Vec<String>,
}

impl RecoverArgs {
    pub fn public_url(&self) -> String {
        public_url(&self.url, &self.hostnames)
    }
}

fn public_url(url: &PublicUrl, hostnames: &[String]) -> String {
    url.public_url.clone().unwrap_or_else(|| {
        let host = hostnames.first().map_or("localhost", String::as_str);
        format!("https://{host}:8443")
    })
}

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    /// Address and port to listen on.
    #[arg(long, env = "LEVIX_HUB_LISTEN", default_value = "0.0.0.0:8443")]
    pub listen: SocketAddr,

    /// Serve plain HTTP, only behind a proxy that terminates TLS.
    #[arg(long, env = "LEVIX_HUB_INSECURE_BEHIND_PROXY", default_value_t = false)]
    pub insecure_behind_proxy: bool,

    #[command(flatten)]
    pub tls: TlsArgs,
}

#[derive(Debug, Clone, Args)]
pub struct TlsArgs {
    /// PEM certificate chain of the customer; a self-signed one is generated when absent.
    #[arg(long, env = "LEVIX_HUB_TLS_CERT", requires = "tls_key")]
    pub tls_cert: Option<PathBuf>,

    /// PEM private key matching `--tls-cert`.
    #[arg(long, env = "LEVIX_HUB_TLS_KEY", requires = "tls_cert")]
    pub tls_key: Option<PathBuf>,

    /// Host names written into a generated certificate (comma separated).
    #[arg(
        long,
        env = "LEVIX_HUB_HOSTNAMES",
        value_delimiter = ',',
        default_value = "localhost"
    )]
    pub hostnames: Vec<String>,
}
