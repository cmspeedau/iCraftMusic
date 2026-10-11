//! Reading audio tags with [`lofty`].
//!
//! Everything the scanner needs from a file on disk is collected here so the
//! rest of the crate never has to know which tag format a file uses. The
//! function returns [`AppError::Metadata`] for a file that cannot be parsed;
//! callers that want to keep going are expected to fall back to
//! [`Track::from_path`](crate::library::Track::from_path) rather than to
//! swallow the error silently.

use std::path::Path;
use std::time::Duration;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey};

use crate::error::{AppError, Result};

/// Tagged information read from a single audio file.
///
/// Every field is optional because taggers are inconsistent: a file may carry
/// a title but no artist, or a duration but no tags at all.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioTags {
    /// Title tag.
    pub title: Option<String>,
    /// Performing artist.
    pub artist: Option<String>,
    /// Album title.
    pub album: Option<String>,
    /// Album-level artist credit.
    pub album_artist: Option<String>,
    /// Free-form genre.
    pub genre: Option<String>,
    /// Track number within its disc.
    pub track_number: Option<u32>,
    /// Disc number within the release.
    pub disc_number: Option<u32>,
    /// Playing time reported by the decoder.
    pub duration: Option<Duration>,
    /// Raw bytes of the first embedded picture, when the file carries one.
    pub artwork: Option<Vec<u8>>,
}

impl AudioTags {
    /// Whether the file carried no usable tags at all.
    ///
    /// The scanner uses this to decide whether a track is worth a warning:
    /// a file with a duration but no text tags is still perfectly playable.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.album_artist.is_none()
            && self.genre.is_none()
            && self.track_number.is_none()
            && self.disc_number.is_none()
            && self.artwork.is_none()
    }
}

/// Read the tags of the audio file at `path`.
///
/// # Errors
///
/// Returns [`AppError::Metadata`] when `lofty` cannot open or parse the file —
/// an unsupported container, a truncated file or a damaged tag block.
pub fn read(path: &Path) -> Result<AudioTags> {
    let tagged = Probe::open(path)
        .and_then(Probe::read)
        .map_err(|error| AppError::metadata(path, error))?;

    let duration = Some(tagged.properties().duration()).filter(|value| !value.is_zero());

    let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) else {
        // A valid audio stream with no tags at all: still worth indexing.
        return Ok(AudioTags {
            duration,
            ..AudioTags::default()
        });
    };

    Ok(AudioTags {
        title: owned(tag.title()),
        artist: owned(tag.artist()),
        album: owned(tag.album()),
        album_artist: owned(tag.get_string(ItemKey::AlbumArtist)),
        genre: owned(tag.genre()),
        track_number: tag.track(),
        disc_number: tag.disk(),
        duration,
        artwork: first_picture(tag.pictures()),
    })
}

/// Copy the first embedded picture out of a tag.
///
/// Cover art is the largest thing a tag usually holds and the only part the
/// player caches separately; the caller decides where it is written.
fn first_picture(pictures: &[lofty::picture::Picture]) -> Option<Vec<u8>> {
    let picture = pictures.first()?;
    let data = picture.data();

    if data.is_empty() {
        None
    } else {
        Some(data.to_vec())
    }
}

/// Convert the borrowed strings `lofty` hands back into owned values, dropping
/// empty ones so a blank tag reads as "unknown" rather than as an empty string.
fn owned(value: Option<impl AsRef<str>>) -> Option<String> {
    value
        .map(|value| value.as_ref().trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Write `bytes` to a uniquely named file under the scratch directory.
    fn scratch_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-tags-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("scratch directory");
        let path = directory.join(name);
        let mut file = std::fs::File::create(&path).expect("scratch file");
        file.write_all(bytes).expect("write scratch file");
        path
    }

    #[test]
    fn an_unreadable_file_reports_a_metadata_error() {
        let path = scratch_file("not-audio.mp3", b"this is not an audio file");

        let error = read(&path).expect_err("garbage must not parse");
        assert!(matches!(error, AppError::Metadata { .. }));
    }

    #[test]
    fn a_missing_file_reports_a_metadata_error() {
        let path = std::env::temp_dir().join("musiccraft-does-not-exist.flac");

        let error = read(&path).expect_err("a missing file must not parse");
        assert!(matches!(error, AppError::Metadata { .. }));
    }

    #[test]
    fn empty_tags_are_reported_as_empty() {
        assert!(AudioTags::default().is_empty());
    }

    #[test]
    fn empty_tags_carry_a_title_only_when_they_actually_do() {
        let tags = AudioTags {
            title: Some("Roygbiv".to_owned()),
            ..AudioTags::default()
        };

        assert!(!tags.is_empty());
    }

    #[test]
    fn blank_tags_are_dropped() {
        assert_eq!(owned(Some("   ".to_owned())), None);
        assert_eq!(
            owned(Some(" Boards of Canada ".to_owned())),
            Some("Boards of Canada".to_owned())
        );
        assert_eq!(owned(None::<String>), None);
    }

    #[test]
    fn a_picture_without_data_is_ignored() {
        let picture = lofty::picture::Picture::unchecked(Vec::new())
            .pic_type(lofty::picture::PictureType::CoverFront)
            .mime_type(lofty::picture::MimeType::Jpeg)
            .build();

        assert_eq!(first_picture(&[picture]), None);
    }
}
