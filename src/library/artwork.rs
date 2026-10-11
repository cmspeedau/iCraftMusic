//! Caching of cover art extracted from the library's files.
//!
//! Embedded pictures are written to a cache directory next to the database so
//! the interface can load a small image instead of re-parsing a whole audio
//! file. The file name is a stable hash of the track's path, so scanning the
//! same folder twice reuses the same cache entry, and the extension is decided
//! from the image's magic bytes rather than from the tag's self-report.

use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// Name of the directory, under the application data directory, that holds
/// cached artwork.
pub const ARTWORK_DIRECTORY: &str = "artwork";

/// Extension used when the image format cannot be identified.
pub const FALLBACK_EXTENSION: &str = "bin";

/// The directory cover art is cached in, given the application data directory.
#[must_use]
pub fn cache_directory(data_directory: &Path) -> PathBuf {
    data_directory.join(ARTWORK_DIRECTORY)
}

/// The cache file name for a track, e.g. `7f3a91c2.jpg`.
#[must_use]
pub fn file_name(track_path: &Path, image: &[u8]) -> String {
    format!("{:016x}.{}", hash(track_path), extension_for(image))
}

/// Guess an image format from its leading bytes.
///
/// Taggers are frequently wrong about the MIME type they advertise, so the
/// bytes themselves are the authority. Anything unrecognised is stored as
/// `bin`, which the interface treats as "do not show".
#[must_use]
pub fn extension_for(image: &[u8]) -> &'static str {
    match image {
        [0xFF, 0xD8, 0xFF, ..] => "jpg",
        [0x89, b'P', b'N', b'G', ..] => "png",
        [b'G', b'I', b'F', ..] => "gif",
        [b'B', b'M', ..] => "bmp",
        _ => FALLBACK_EXTENSION,
    }
}

/// Store `image` in `directory`, returning the path it was written to.
///
/// The write goes to a temporary file first and is then renamed, so a reader
/// never observes a half-written cover.
///
/// # Errors
///
/// Returns [`AppError::Io`] when the directory cannot be created or the file
/// cannot be written.
pub fn store(directory: &Path, track_path: &Path, image: &[u8]) -> Result<PathBuf> {
    std::fs::create_dir_all(directory).map_err(|error| AppError::io(directory, error))?;

    let destination = directory.join(file_name(track_path, image));
    let temporary = destination.with_extension(format!("{}.part", extension_for(image)));

    std::fs::write(&temporary, image).map_err(|error| AppError::io(&temporary, error))?;
    std::fs::rename(&temporary, &destination).map_err(|error| AppError::io(&destination, error))?;

    Ok(destination)
}

/// Store `image` for `track_path`, discarding the result.
///
/// Cover art is a nicety: a scan must not fail because a cache directory is
/// read-only, so the caller that does not care about the path uses this and the
/// reason is logged instead.
pub fn store_best_effort(directory: &Path, track_path: &Path, image: &[u8]) {
    if let Err(error) = store(directory, track_path, image) {
        tracing::warn!(
            "cover art for `{}` was not cached: {error}",
            track_path.display()
        );
    }
}

/// FNV-1a over the path's bytes.
///
/// A local hash keeps the crate free of a hashing dependency and is entirely
/// adequate here: the value only needs to be stable across runs, not
/// collision-proof.
fn hash(path: &Path) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut value = OFFSET_BASIS;
    for byte in path.to_string_lossy().as_bytes() {
        value ^= u64::from(*byte);
        value = value.wrapping_mul(PRIME);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_directory_sits_under_the_data_directory() {
        let directory = cache_directory(Path::new("/data/musiccraft"));
        assert_eq!(directory, PathBuf::from("/data/musiccraft/artwork"));
    }

    #[test]
    fn image_formats_are_detected_from_their_magic_bytes() {
        assert_eq!(extension_for(&[0xFF, 0xD8, 0xFF, 0xE0]), "jpg");
        assert_eq!(extension_for(&[0x89, b'P', b'N', b'G', 0x0D]), "png");
        assert_eq!(extension_for(b"GIF89a"), "gif");
        assert_eq!(extension_for(b"BM\x36"), "bmp");
    }

    #[test]
    fn unknown_image_data_falls_back_to_the_generic_extension() {
        assert_eq!(extension_for(b"nonsense"), FALLBACK_EXTENSION);
    }

    #[test]
    fn file_names_are_stable_for_the_same_track() {
        let path = Path::new("/music/Boards of Canada/Roygbiv.mp3");
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];

        assert_eq!(file_name(path, &jpeg), file_name(path, &jpeg));
        assert!(file_name(path, &jpeg).ends_with(".jpg"));
    }

    #[test]
    fn different_tracks_get_different_file_names() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];

        assert_ne!(
            file_name(Path::new("/music/a.mp3"), &jpeg),
            file_name(Path::new("/music/b.mp3"), &jpeg)
        );
    }

    #[test]
    fn storing_artwork_writes_a_readable_file() {
        let directory = std::env::temp_dir().join(format!(
            "musiccraft-artwork-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let track = Path::new("/music/Boards of Canada/Roygbiv.mp3");
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0x00];

        let written = store(&directory, track, &jpeg).expect("artwork is written");

        assert_eq!(std::fs::read(&written).expect("read back"), jpeg);
        assert_eq!(written.parent(), Some(directory.as_path()));

        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn best_effort_storing_never_panics() {
        // A path that cannot be created (a file used as a directory).
        let file =
            std::env::temp_dir().join(format!("musiccraft-artwork-file-{}", std::process::id()));
        std::fs::write(&file, b"not a directory").expect("scratch file");

        store_best_effort(&file, Path::new("/music/a.mp3"), &[0xFF, 0xD8, 0xFF]);
        assert_eq!(std::fs::read(&file).expect("unchanged"), b"not a directory");

        std::fs::remove_file(&file).ok();
    }
}
