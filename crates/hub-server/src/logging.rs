//! Structured JSON logs on standard output. Never log passwords, codes, tokens or secrets.

use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::EnvFilter;

/// The Hub's log format writing to `writer` (standard output in production).
pub fn subscriber<W>(filter: &str, writer: W) -> impl Subscriber + Send + Sync
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    let filter = EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_target(true)
        .with_writer(writer)
        .finish()
}

pub fn init(filter: &str) {
    // A second initialization (in tests) is harmless and ignored.
    let _ignored = tracing::subscriber::set_global_default(subscriber(filter, std::io::stdout));
}
