//! Failures of the CLI and their stable exit codes.

use serde_json::{json, Value};
use thiserror::Error;

/// Exit codes, part of the CLI's contract with scripts.
pub mod exit {
    pub const OK: u8 = 0;
    pub const FAILURE: u8 = 1;
    pub const USAGE: u8 = 2;
    pub const UNAUTHENTICATED: u8 = 3;
    pub const FORBIDDEN: u8 = 4;
    pub const NOT_FOUND: u8 = 5;
    pub const CONFLICT: u8 = 6;
    pub const INVALID: u8 = 7;
    pub const THROTTLED: u8 = 8;
    pub const CONNECTION: u8 = 9;
    pub const SERVER: u8 = 10;
}

#[derive(Debug, Error)]
pub enum CliError {
    /// The Hub answered with the contract's error object.
    #[error("{message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    /// The Hub could not be reached, or its certificate is not trusted.
    #[error("{0}")]
    Connection(String),
    /// Missing or contradictory options.
    #[error("{0}")]
    Usage(String),
    /// Local configuration or terminal problems.
    #[error("{0}")]
    Local(String),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Api { status, .. } => match status {
                401 => exit::UNAUTHENTICATED,
                403 => exit::FORBIDDEN,
                404 => exit::NOT_FOUND,
                409 => exit::CONFLICT,
                400 | 413 | 415 | 422 => exit::INVALID,
                429 => exit::THROTTLED,
                500..=599 => exit::SERVER,
                _ => exit::FAILURE,
            },
            Self::Connection(_) => exit::CONNECTION,
            Self::Usage(_) => exit::USAGE,
            Self::Local(_) => exit::FAILURE,
        }
    }

    /// The error object printed in JSON mode: the Hub's object as received, or the same
    /// shape for local failures.
    pub fn to_json(&self) -> Value {
        let code = match self {
            Self::Api { code, .. } => code.as_str(),
            Self::Connection(_) => "connection",
            Self::Usage(_) => "usage",
            Self::Local(_) => "local",
        };
        json!({ "error": { "code": code, "message": self.to_string() } })
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Local(error.to_string())
    }
}

pub type CliResult<T> = Result<T, CliError>;
