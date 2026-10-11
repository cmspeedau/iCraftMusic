//! Process-wide logging setup built on [`tracing`].

use tracing_subscriber::EnvFilter;

/// Environment variable consulted for log filtering.
const ENV_FILTER: &str = "RUST_LOG";

/// Default filter used when `RUST_LOG` is unset.
const DEFAULT_FILTER: &str = "musiccraft=info,warn";

/// Install the global tracing subscriber.
///
/// Verbosity is controlled with `RUST_LOG` (for example
/// `RUST_LOG=musiccraft=debug`). Calling this more than once is harmless: the
/// second call is ignored because a global subscriber already exists.
pub fn init() {
    let filter =
        EnvFilter::try_from_env(ENV_FILTER).unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    if installed.is_err() {
        // Another component already installed a subscriber; nothing to do.
        tracing::debug!("logging subscriber was already installed");
    }
}
