//! `levix-hub`: runs the Admin Hub and its host commands.

use clap::Parser;
use hub_server::cli::{AuditCommand, Cli, Command};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    hub_server::logging::init(&cli.log);
    let result = match cli.command {
        Command::Serve(args) => hub_server::serve::run(&cli.data_dir, args).await,
        Command::Init(args) => hub_server::init::run(&cli.data_dir, args).await,
        Command::RecoverOwner(args) => {
            hub_server::init::recover_owner(&cli.data_dir, &args.email, &args.public_url()).await
        }
        Command::Migrate => hub_server::migrate::run(&cli.data_dir).await,
        Command::Audit(AuditCommand::Verify) => hub_server::init::verify_audit(&cli.data_dir).await,
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "command failed");
            eprintln!("error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
