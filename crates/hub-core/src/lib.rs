//! The core of the Admin Hub: domain types, use cases and the ports they need.
//!
//! Adapters (storage, HTTP, console, CLI) depend on this crate; it depends on none of them
//! (ADR-0001). Every surface calls the same use cases (methods of [`Hub`]), so rules are
//! enforced once.

pub mod administrator;
pub mod audit;
pub mod context;
pub mod credential;
pub mod error;
pub mod hub;
pub mod instance;
pub mod organization;
pub mod ports;
pub mod role;
pub mod session;

pub use error::{CoreError, CoreResult};
pub use hub::Hub;
