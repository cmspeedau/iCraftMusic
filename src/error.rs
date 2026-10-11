//! Error types shared by the application.
//!
//! Library-level errors use [`thiserror`] so they stay cheap, typed and easy to
//! match on. The binary boundary converts them to `anyhow` for reporting.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Everything that can go wrong inside MusicCraft.
#[derive(Debug, Error)]
pub enum AppError {
    /// The operating system did not provide a per-user config directory.
    #[error("no configuration directory is available on this platform")]
    NoConfigDir,

    /// Reading or writing a file failed.
    #[error("failed to access `{}`: {source}", path.display())]
    Io {
        /// The path that could not be accessed.
        path: PathBuf,
        /// The underlying I/O failure.
        #[source]
        source: std::io::Error,
    },

    /// A configuration file exists but does not parse.
    #[error("failed to parse `{}`: {source}", path.display())]
    Toml {
        /// The offending configuration file.
        path: PathBuf,
        /// The underlying deserialisation failure.
        #[source]
        source: toml::de::Error,
    },

    /// A configuration value could not be serialised.
    #[error("failed to serialise configuration: {source}")]
    Serialize {
        /// The underlying serialisation failure.
        #[source]
        source: toml::ser::Error,
    },
}

impl AppError {
    /// Build an [`AppError::Io`] without repeating the path plumbing.
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

/// Convenience alias for fallible operations in this crate.
pub type Result<T> = std::result::Result<T, AppError>;
