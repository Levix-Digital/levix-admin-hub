//! The core of the Admin Hub: domain types, use cases and the ports they need.
//!
//! Adapters (storage, HTTP, console, CLI) depend on this crate; it depends on none of them
//! (ADR-0001). Every surface calls the same use cases, so rules are enforced once.

pub mod error;
pub mod instance;
pub mod organization;
pub mod ports;

#[cfg(test)]
mod testing;

pub use error::{CoreError, CoreResult};
