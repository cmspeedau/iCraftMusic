//! The audio playback engine.
//!
//! The module is split so that the part which needs a sound card stays small
//! and the part which holds the queue is pure logic that can be unit tested
//! anywhere:
//!
//! - [`AudioEngine`] owns the output device, loads files and exposes the
//!   transport controls (play, pause, stop, seek, volume).
//! - [`PlayQueue`] holds the tracks and decides what plays next under the
//!   repeat and shuffle rules.
//! - [`AudioError`] is the failure type both layers report through.
//!
//! The engine is deliberately forgiving about a missing output device: it
//! opens in that state and reports [`AudioError::OutputUnavailable`] from the
//! operations that need sound, so the application can still browse a library
//! and build a queue on a machine with no sound card.

mod error;
mod output;
mod playback;
mod player;
mod queue;

pub use error::AudioError;
pub use output::Output;
pub use playback::{Playback, PlaybackEvent};
pub use player::AudioEngine;
pub use queue::{PlayQueue, QueueItem, RepeatMode};

use std::path::Path;

/// The loudest volume the engine accepts, matching the configuration range.
pub const MAX_VOLUME: f32 = 1.0;

/// File extensions the bundled decoder can read.
///
/// This mirrors rodio's default feature set (MP3, FLAC, WAV, Vorbis and the
/// AAC/MP4 family) and is used both to filter folder scans and to explain in
/// error messages what the engine understands.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "wave", "ogg", "oga", "m4a", "mp4", "aac",
];

/// Whether the engine can decode `path`, judged by its file extension.
///
/// This is a cheap check for scanning folders and for enabling UI actions; the
/// decoder is still the final authority on whether a file really plays.
#[must_use]
pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|extension| SUPPORTED_EXTENSIONS.contains(&extension.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_are_matched_case_insensitively() {
        assert!(is_supported(Path::new("/music/song.mp3")));
        assert!(is_supported(Path::new("/music/song.FLAC")));
        assert!(is_supported(Path::new("/music/song.M4a")));
    }

    #[test]
    fn unsupported_files_are_rejected() {
        assert!(!is_supported(Path::new("/music/notes.txt")));
        assert!(!is_supported(Path::new("/music/cover.png")));
        assert!(!is_supported(Path::new("/music/no-extension")));
        assert!(!is_supported(Path::new("/music/dot.")));
    }

    #[test]
    fn the_supported_list_is_usable_in_messages() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(!SUPPORTED_EXTENSIONS.is_empty());
    }
}
