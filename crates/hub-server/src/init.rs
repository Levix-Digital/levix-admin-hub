//! Host commands: `init` claims the Hub once and `recover-owner` issues a recovery link for
//! the instance owner. Holding the host is the trust anchor: no account can be created over
//! the network (threat model T-01, T-15, T-23).

use std::path::Path;

use hub_core::context::Context;
use hub_core::credential::IssuedLink;
use time::format_description::well_known::Rfc3339;

use crate::cli::InitArgs;
use crate::{hub, tls};

/// What `init` created, for the caller and the tests.
#[derive(Debug)]
pub struct Initialized {
    pub fingerprint: String,
    pub setup_link: IssuedLink,
}

/// The console address of a link: the token sits in the fragment, which browsers never send.
pub fn link_url(public_url: &str, token: &str) -> String {
    format!("{}/#/link/{token}", public_url.trim_end_matches('/'))
}

fn print_link(public_url: &str, link: &IssuedLink) {
    let expires = link
        .link
        .expires_at
        .format(&Rfc3339)
        .unwrap_or_else(|_| link.link.expires_at.to_string());
    println!("Single-use link, valid until {expires}:");
    println!("  {}", link_url(public_url, &link.token));
    println!("Deliver it only to its holder; anyone with it can set the password.");
}

pub async fn run(data_dir: &Path, args: InitArgs) -> anyhow::Result<()> {
    let result = initialize(data_dir, &args).await?;
    println!("The Hub is initialized.");
    println!(
        "TLS certificate SHA-256 fingerprint: {}",
        result.fingerprint
    );
    println!("Check this fingerprint in your browser before using the link.");
    print_link(&args.public_url(), &result.setup_link);
    Ok(())
}

pub async fn initialize(data_dir: &Path, args: &InitArgs) -> anyhow::Result<Initialized> {
    let (hub, _) = hub::open(data_dir).await?;
    let (instance, setup_link) = hub.initialize(&Context::host(), &args.owner_email).await?;
    tracing::info!(instance = %instance.id, actor = "host", action = "instance.initialize", "instance initialized");
    let files = tls::prepare(data_dir, &args.tls)?;
    Ok(Initialized {
        fingerprint: tls::fingerprint(&files.certificate)?,
        setup_link,
    })
}

pub async fn recover_owner(data_dir: &Path, email: &str, public_url: &str) -> anyhow::Result<()> {
    let (hub, _) = hub::open(data_dir).await?;
    let link = hub.recover_owner(&Context::host(), email).await?;
    tracing::info!(
        actor = "host",
        action = "instance.recover_owner",
        "recovery link issued"
    );
    println!("Recovery link for {email}. It replaces the password and the authenticator.");
    print_link(public_url, &link);
    Ok(())
}

/// `audit verify`: fails when the chain is broken.
pub async fn verify_audit(data_dir: &Path) -> anyhow::Result<()> {
    let (hub, _) = hub::open(data_dir).await?;
    let report = hub.verify_audit(&Context::host()).await?;
    if report.intact {
        println!("Audit chain intact: {} records.", report.records);
        Ok(())
    } else {
        anyhow::bail!(
            "audit chain broken at record {} ({} records read); restore a backup and investigate",
            report.first_broken_seq.unwrap_or_default(),
            report.records
        )
    }
}
