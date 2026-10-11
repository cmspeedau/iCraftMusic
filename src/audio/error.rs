//! Error types for the audio playback engine.
//!
//! Failures are typed so the application layer can decide what to show the
//! user, for example a missing file versus a file whose format the bundled
//! decoder cannot read.

use std::path::{Path, PathBuf};

use rodio::DeviceSinkError;
use rodio::decoder::DecoderError;
use thiserror::Error;

use super::SUPPORTED_EXTENSIONS;

/// Everything that can go wrong while opening the output device, loading a
/// track or controlling playback.
#[derive(Debug, Error)]
pub enum AudioError {
    /// No audio output device could be opened.
    ///
    /// The machine may have no sound card, or the device may be in use
    /// exclusively by another application.
    #[error("no audio output device is available: {source}")]
    OutputDevice {
        /// The failure reported by the audio backend.
        #[source]
        source: DeviceSinkError,
    },

    /// Playback was requested but no output device is open.
    ///
    /// The engine stays usable in this state: the queue and the transport
    /// controls keep working, only sound is impossible.
    #[error("playback is unavailable because no audio output device could be opened")]
    OutputUnavailable,

    /// Playback was requested but the queue holds no track.
    #[error("there is nothing to play because the queue is empty")]
    EmptyQueue,

    /// The file could not be opened.
    #[error("failed to open `{}`: {source}", path.display())]
    Open {
        /// The file that could not be opened.
        path: PathBuf,
        /// The underlying I/O failure: missing file, permissions, and so on.
        #[source]
        source: std::io::Error,
    },

    /// The decoder did not recognise the file's contents as audio.
    #[error(
        "`{}` is not in a supported audio format (supported: {})",
        path.display(),
        SUPPORTED_EXTENSIONS.join(", ")
    )]
    UnsupportedFormat {
        /// The file the decoder rejected.
        path: PathBuf,
    },

    /// The file was recognised but could not be decoded.
    #[error("failed to decode `{}`: {source}", path.display())]
    Decode {
        /// The file that could not be decoded.
        path: PathBuf,
        /// The underlying decoder failure.
        #[source]
        source: DecoderError,
    },

    /// The current track cannot be seeked to the requested position.
    #[error("cannot seek in `{}`: {source}", path.display())]
    Seek {
        /// The track that rejected the seek.
        path: PathBuf,
        /// The underlying seek failure.
        #[source]
        source: rodio::source::SeekError,
    },
}

impl AudioError {
    /// The file this error refers to, if it is about a file.
    ///
    /// Device-level failures are not tied to a file and return [`None`].
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::OutputDevice { .. } | Self::OutputUnavailable | Self::EmptyQueue => None,
            Self::Open { path, .. }
            | Self::UnsupportedFormat { path }
            | Self::Decode { path, .. }
            | Self::Seek { path, .. } => Some(path),
        }
    }

    /// Whether the user can fix this by picking a different file.
    #[must_use]
    pub fn is_file_problem(&self) -> bool {
        matches!(
            self,
            Self::Open { .. } | Self::UnsupportedFormat { .. } | Self::Decode { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_problems_expose_their_path() {
        let error = AudioError::UnsupportedFormat {
            path: PathBuf::from("/music/song.txt"),
        };

        assert_eq!(error.path(), Some(Path::new("/music/song.txt")));
        assert!(error.is_file_problem());
        assert!(
            error.to_string().contains("/music/song.txt"),
            "the message should name the file: {error}"
        );
        assert!(
            error.to_string().contains("mp3"),
            "the message should list what is supported: {error}"
        );
    }

    #[test]
    fn device_problems_are_not_tied_to_a_file() {
        let error = AudioError::OutputUnavailable;

        assert_eq!(error.path(), None);
        assert!(!error.is_file_problem());
    }
}
