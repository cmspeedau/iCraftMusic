//! Error types shared by the application.
//!
//! Library-level errors use [`thiserror`] so they stay cheap, typed and easy to
//! match on. The binary boundary converts them to `anyhow` for reporting.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Everything that can go wrong inside iCraftMusic.
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

    /// A file's audio tags could not be read.
    #[error("failed to read metadata from `{}`: {source}", path.display())]
    Metadata {
        /// The file whose tags could not be read.
        path: PathBuf,
        /// The underlying `lofty` failure.
        #[source]
        source: lofty::error::FileParseError,
    },

    /// A file's audio tags could not be written.
    #[error("the tags of `{}` could not be written: {source}", path.display())]
    TagWrite {
        /// The file whose tags could not be written.
        path: PathBuf,
        /// The underlying `lofty` failure.
        #[source]
        source: lofty::error::FileEncodingError,
    },

    /// A file's format has no tag block this player knows how to write.
    ///
    /// Nothing is written for such a file, and the user is told so rather than
    /// being left with an edit that silently never reaches the disk.
    #[error("`{}` is in a format whose tags this player cannot write", path.display())]
    TagUnsupported {
        /// The file that was refused.
        path: PathBuf,
    },

    /// An image the user chose as cover art could not be read as an image.
    #[error("`{}` could not be read as an image", path.display())]
    NotAnImage {
        /// The file that was rejected.
        path: PathBuf,
    },

    /// A file's extension is not one of the formats the player understands.
    #[error("`{}` is not a supported audio format", path.display())]
    UnsupportedFormat {
        /// The file that was rejected.
        path: PathBuf,
    },

    /// Reading from or writing to the library database failed.
    #[error("library database error: {source}")]
    Database {
        /// The underlying SQLite failure.
        #[source]
        source: rusqlite::Error,
    },

    /// A playlist name was empty or only whitespace.
    #[error("a playlist needs a name")]
    InvalidPlaylistName,

    /// A playlist referenced by identifier is not in the database.
    #[error("playlist {0} does not exist")]
    PlaylistNotFound(crate::library::PlaylistId),

    /// A track was used before it was stored, so it has no row to reference.
    #[error("track {0} has not been stored in the library yet")]
    TrackHasNoRow(crate::library::TrackId),
}

impl AppError {
    /// Build an [`AppError::Io`] without repeating the path plumbing.
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Build an [`AppError::Metadata`] without repeating the path plumbing.
    pub fn metadata(path: impl AsRef<Path>, source: lofty::error::FileParseError) -> Self {
        Self::Metadata {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Build an [`AppError::UnsupportedFormat`] without repeating the plumbing.
    pub fn unsupported_format(path: impl AsRef<Path>) -> Self {
        Self::UnsupportedFormat {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Build an [`AppError::TagWrite`] without repeating the plumbing.
    pub fn tag_write(path: impl AsRef<Path>, source: lofty::error::FileEncodingError) -> Self {
        Self::TagWrite {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Build an [`AppError::NotAnImage`] without repeating the plumbing.
    pub fn not_an_image(path: impl AsRef<Path>) -> Self {
        Self::NotAnImage {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Build an [`AppError::Database`] without repeating the plumbing.
    pub fn database(source: rusqlite::Error) -> Self {
        Self::Database { source }
    }
}

/// Convenience alias for fallible operations in this crate.
pub type Result<T> = std::result::Result<T, AppError>;
