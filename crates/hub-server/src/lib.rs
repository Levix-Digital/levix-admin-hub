//! Composition root of the Admin Hub: builds the adapters, wires them into the core and runs
//! the server or a host command. Nothing here holds a business rule.

pub mod cli;
pub mod init;
pub mod logging;
pub mod migrate;
pub mod paths;
pub mod readiness;
pub mod serve;
pub mod tls;
