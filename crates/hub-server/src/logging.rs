//! Structured JSON logs on standard output. Never log passwords, codes, tokens or secrets.

use tracing_subscriber::EnvFilter;

pub fn init(filter: &str) {
    let filter = EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info"));
    // A second initialization (in tests) is harmless and ignored.
    let _ignored = tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_target(true)
        .try_init();
}
