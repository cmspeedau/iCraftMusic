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
pub use output::{AudioDevice, Output, available_devices};
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

/// Music formats the bundled decoder cannot read.
///
/// These are kept apart from everything else that is not audio for one reason:
/// a file with one of these extensions is music the user expects to hear, so a
/// scan counts them and names the formats rather than passing over them in
/// silence. A `.txt`, a cover image or a playlist is not listed here and is
/// ignored, because telling a user their text file is an unsupported format
/// would be noise.
pub const UNREADABLE_EXTENSIONS: &[&str] = &[
    "aif", "aiff", "alac", "ape", "dff", "dsf", "mka", "mpc", "opus", "ra", "tak", "tta", "wma",
    "wv",
];

/// Whether the engine can decode `path`, judged by its file extension.
///
/// This is a cheap check for scanning folders and for enabling UI actions; the
/// decoder is still the final authority on whether a file really plays.
#[must_use]
pub fn is_supported(path: &Path) -> bool {
    extension(path).is_some_and(|extension| SUPPORTED_EXTENSIONS.contains(&extension.as_str()))
}

/// Whether `path` is a music file in a format this build cannot decode.
///
/// The counterpart to [`is_supported`]: together they split the files in a
/// user's folders into what can be played and what cannot be, and everything
/// else — text, images, playlists — falls outside both.
#[must_use]
pub fn is_unreadable(path: &Path) -> bool {
    extension(path).is_some_and(|extension| UNREADABLE_EXTENSIONS.contains(&extension.as_str()))
}

/// The lowercased extension of a path, when it has one.
fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
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
    fn music_in_a_format_the_engine_cannot_read_is_recognised_as_such() {
        assert!(is_unreadable(Path::new("/music/song.wma")));
        assert!(is_unreadable(Path::new("/music/song.APE")));
        assert!(is_unreadable(Path::new("/music/song.aiff")));
    }

    #[test]
    fn what_can_be_played_is_not_also_reported_as_unreadable() {
        for path in ["a.mp3", "b.flac", "c.wav", "d.m4a"] {
            let path = Path::new(path);
            assert!(!(is_supported(path) && is_unreadable(path)));
        }
    }

    #[test]
    fn files_that_are_not_audio_are_neither_playable_nor_unreadable() {
        // The distinction matters: a scan reports the second list to the user
        // and stays quiet about the first, so a text file must not land in it.
        for path in ["notes.txt", "cover.png", "playlist.m3u", "LICENSE"] {
            let path = Path::new(path);
            assert!(!is_supported(path));
            assert!(!is_unreadable(path), "{path:?} is not music");
        }
    }

    #[test]
    fn the_two_lists_do_not_overlap() {
        for extension in UNREADABLE_EXTENSIONS {
            assert!(
                !SUPPORTED_EXTENSIONS.contains(extension),
                "{extension} is in both lists"
            );
        }
    }

    #[test]
    fn the_supported_list_is_usable_in_messages() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(!SUPPORTED_EXTENSIONS.is_empty());
    }
}
