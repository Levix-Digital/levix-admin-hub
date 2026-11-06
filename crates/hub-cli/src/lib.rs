//! `levix-hub-cli`: administers a Hub over its public API, the same API the console uses.
//! It depends on no Hub crate, so it can only do what the API allows.

pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod operations;
pub mod output;
pub mod tls;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Format};
use crate::error::exit;

/// Parses the command line, runs the command and prints its result.
pub fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = cli.global.output;
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: cannot start: {error}");
            return ExitCode::from(exit::FAILURE);
        }
    };
    let result = runtime.block_on(async {
        let context = commands::Context::load(cli.global)?;
        commands::run(&context, cli.command).await
    });
    match result {
        Ok(report) => {
            match format {
                Format::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&report.json).unwrap_or_default()
                ),
                Format::Human => println!("{}", report.human),
            }
            ExitCode::from(report.exit)
        }
        Err(error) => {
            match format {
                Format::Json => eprintln!("{}", error.to_json()),
                Format::Human => eprintln!("error: {error}"),
            }
            ExitCode::from(error.exit_code())
        }
    }
}
