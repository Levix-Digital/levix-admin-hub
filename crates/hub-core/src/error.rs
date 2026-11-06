//! The error type every use case returns. Adapters translate it once, at their edge.

use thiserror::Error;

/// What went wrong in a use case, independent of the surface that called it.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// The input breaks a rule of the domain; the message names the field and the rule.
    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: String },
    /// The record does not exist (or is not visible to the caller).
    #[error("{0} not found")]
    NotFound(&'static str),
    /// The change conflicts with the current state (duplicate slug, archived record...).
    #[error("conflict: {0}")]
    Conflict(String),
    /// The instance has not been initialized on its host yet.
    #[error("the Hub is not initialized")]
    NotInitialized,
    /// The instance was already initialized; initialization runs once.
    #[error("the Hub is already initialized")]
    AlreadyInitialized,
    /// The caller is not authenticated, or the credentials are wrong. Deliberately vague.
    #[error("authentication required or failed")]
    Unauthenticated,
    /// The caller is authenticated but lacks the permission named.
    #[error("missing permission: {permission}")]
    Forbidden { permission: &'static str },
    /// Too many failed attempts; retry after the given number of seconds.
    #[error("too many failed attempts; retry in {retry_after_secs} s")]
    Throttled { retry_after_secs: u64 },
    /// A port failed (database, key file); the message carries the adapter's context.
    #[error("storage failure: {0}")]
    Storage(String),
}

/// Result of a use case.
pub type CoreResult<T> = Result<T, CoreError>;
