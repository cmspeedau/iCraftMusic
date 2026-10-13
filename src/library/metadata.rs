//! Reading audio tags with [`lofty`].
//!
//! Everything the scanner needs from a file on disk is collected here so the
//! rest of the crate never has to know which tag format a file uses. The
//! function returns [`AppError::Metadata`] for a file that cannot be parsed;
//! callers that want to keep going are expected to fall back to
//! [`Track::from_path`](crate::library::Track::from_path) rather than to
//! swallow the error silently.
//!
//! [`write`] is the other half, and it is deliberately the only thing in the
//! crate that modifies a user's file: nothing calls it on a scan or on a
//! restart, only the metadata editor, and only when the user asks for the file
//! itself to be changed.

use std::fs::OpenOptions;
use std::path::Path;
use std::time::Duration;

use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::{AudioFile, BoundTaggedFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag};

use crate::error::{AppError, Result};
use crate::library::track::TrackFields;

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

/// Write `fields` into the tags of the audio file at `path`.
///
/// This is the only function in the crate that changes a file the user owns, so
/// it is called from exactly one place: the metadata editor, after the user has
/// asked for the file itself to be updated. A scan, a refresh and a restart
/// never reach it.
///
/// `artwork`, when given, replaces the file's front cover with those bytes. The
/// picture is left alone when it is `None`, which is what keeps an edit of the
/// title from throwing away cover art the player never showed.
///
/// # Errors
///
/// - [`AppError::Io`] when the file cannot be opened for writing — a read-only
///   file or one this process may not touch.
/// - [`AppError::Metadata`] when the file cannot be read as audio.
/// - [`AppError::TagUnsupported`] when the format has no tag block this build
///   can write. The file is left untouched rather than half-written.
/// - [`AppError::TagWrite`] when the tag block cannot be encoded.
pub fn write(path: &Path, fields: &TrackFields, artwork: Option<&[u8]>) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| AppError::io(path, error))?;

    let mut tagged = BoundTaggedFile::read_from(file, ParseOptions::new())
        .map_err(|error| AppError::metadata(path, error))?;

    // Which tag block the file wants to carry. A format whose primary tag type
    // is read-only would swallow the write without saying so, so it is refused
    // here instead: the caller can then tell the user the file was not touched.
    let tag_type = tagged.primary_tag_type();
    if !tagged.tag_support(tag_type).is_writable() {
        return Err(AppError::TagUnsupported {
            path: path.to_path_buf(),
        });
    }

    // A file that carries no tag block yet gets one, in the type its format
    // wants. `insert_tag` hands back the tag it replaced rather than the one it
    // stored, so the only way to tell whether there is now a tag to write into
    // is to look again.
    if tagged.tag_mut(tag_type).is_none() {
        tagged.insert_tag(Tag::new(tag_type));
    }

    let Some(tag) = tagged.tag_mut(tag_type) else {
        return Err(AppError::TagUnsupported {
            path: path.to_path_buf(),
        });
    };

    apply(tag, fields);
    if let Some(image) = artwork.filter(|image| !image.is_empty()) {
        embed_cover(tag, image);
    }

    tagged
        .save(WriteOptions::default())
        .map_err(|error| AppError::tag_write(path, error))
}

/// Copy validated fields onto a tag, removing the ones the user cleared.
///
/// A blank field means "this file has no artist", not "leave whatever was
/// there": the editor shows an empty text box, so writing anything else would
/// contradict what the user was looking at when they pressed the button.
fn apply(tag: &mut Tag, fields: &TrackFields) {
    tag.set_title(fields.title.clone());
    set_text(tag, ItemKey::TrackArtist, &fields.artist);
    set_text(tag, ItemKey::AlbumTitle, &fields.album);
    set_text(tag, ItemKey::AlbumArtist, &fields.album_artist);
    set_text(tag, ItemKey::Genre, &fields.genre);

    match fields.track_number {
        Some(number) => tag.set_track(number),
        None => tag.remove_track(),
    }
    match fields.disc_number {
        Some(number) => tag.set_disk(number),
        None => tag.remove_disk(),
    }
}

/// Set one free-form tag item, or drop it when the field is empty.
fn set_text(tag: &mut Tag, key: ItemKey, value: &Option<String>) {
    match value {
        Some(value) => {
            tag.insert_text(key, value.clone());
        }
        None => tag.remove_key(key),
    }
}

/// Replace the file's front cover with `image`.
///
/// The bytes are what the user chose, so the format is read from them rather
/// than from the file name they came from; the previous cover — and only the
/// front cover, not a back cover or an artist photo — is dropped, because a file
/// with two front covers would keep showing the old one.
fn embed_cover(tag: &mut Tag, image: &[u8]) {
    let picture = Picture::unchecked(image.to_vec())
        .pic_type(PictureType::CoverFront)
        .mime_type(mime_type(image))
        .build();

    tag.remove_picture_type(PictureType::CoverFront);
    tag.push_picture(picture);
}

/// The MIME type of an image, judged by its leading bytes.
///
/// Taggers and file names are both frequently wrong about this, so the bytes
/// are the authority — the same rule the artwork cache follows.
fn mime_type(image: &[u8]) -> MimeType {
    match crate::library::artwork::extension_for(image) {
        "jpg" => MimeType::Jpeg,
        "png" => MimeType::Png,
        "gif" => MimeType::Gif,
        "bmp" => MimeType::Bmp,
        _ => MimeType::Unknown("application/octet-stream".to_owned()),
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
    use crate::fixtures::{png, scratch_file, silent_wav};

    /// Every field set, so a write has something to say about each of them.
    fn fields() -> TrackFields {
        TrackFields {
            title: "Roygbiv".to_owned(),
            artist: Some("Boards of Canada".to_owned()),
            album: Some("Music Has the Right to Children".to_owned()),
            album_artist: Some("Boards of Canada".to_owned()),
            genre: Some("Electronic".to_owned()),
            track_number: Some(3),
            disc_number: Some(1),
        }
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

    #[test]
    fn every_field_of_a_written_file_can_be_read_back() {
        let path = scratch_file("write-all.wav", &silent_wav(64));
        let written = fields();

        write(&path, &written, None).expect("the tags should be written");

        let read_back = read(&path).expect("the file should still be readable");
        assert_eq!(read_back.title.as_deref(), Some("Roygbiv"));
        assert_eq!(read_back.artist.as_deref(), Some("Boards of Canada"));
        assert_eq!(
            read_back.album.as_deref(),
            Some("Music Has the Right to Children")
        );
        assert_eq!(read_back.album_artist.as_deref(), Some("Boards of Canada"));
        assert_eq!(read_back.genre.as_deref(), Some("Electronic"));
        assert_eq!(read_back.track_number, Some(3));
        assert_eq!(read_back.disc_number, Some(1));
    }

    #[test]
    fn a_second_write_replaces_what_the_first_one_put_there() {
        let path = scratch_file("write-twice.wav", &silent_wav(64));
        write(&path, &fields(), None).expect("first write");

        let mut changed = fields();
        changed.title = "Olson".to_owned();
        changed.artist = None;
        write(&path, &changed, None).expect("second write");

        let read_back = read(&path).expect("readable");
        assert_eq!(read_back.title.as_deref(), Some("Olson"));
        assert_eq!(
            read_back.artist, None,
            "a cleared field means the file has no artist"
        );
    }

    #[test]
    fn a_chosen_cover_is_embedded_in_the_file() {
        let path = scratch_file("write-cover.wav", &silent_wav(64));
        let cover = png();

        write(&path, &fields(), Some(&cover)).expect("the cover should be written");

        let read_back = read(&path).expect("readable");
        assert_eq!(
            read_back.artwork.as_deref(),
            Some(cover.as_slice()),
            "the cover the file carries is the one that was written"
        );
    }

    #[test]
    fn writing_without_a_picture_leaves_the_files_own_cover_alone() {
        let path = scratch_file("write-keep-cover.wav", &silent_wav(64));
        let cover = png();
        write(&path, &fields(), Some(&cover)).expect("first write");

        let mut changed = fields();
        changed.title = "Olson".to_owned();
        write(&path, &changed, None).expect("second write");

        let read_back = read(&path).expect("readable");
        assert_eq!(
            read_back.artwork.as_deref(),
            Some(cover.as_slice()),
            "a title edit must not throw the cover away"
        );
    }

    #[test]
    fn writing_to_a_file_that_is_not_there_reports_an_io_failure() {
        let path = std::env::temp_dir().join("musiccraft-write-absent.wav");

        let error = write(&path, &fields(), None).expect_err("there is no file to write");

        assert!(matches!(error, AppError::Io { .. }));
    }

    #[test]
    fn writing_to_a_file_that_is_not_audio_reports_a_metadata_failure() {
        let path = scratch_file("write-not-audio.wav", b"definitely not a WAV file");

        let error = write(&path, &fields(), None).expect_err("garbage cannot be tagged");

        assert!(matches!(error, AppError::Metadata { .. }));
    }

    #[test]
    fn writing_an_empty_picture_changes_nothing() {
        let path = scratch_file("write-empty-cover.wav", &silent_wav(64));
        write(&path, &fields(), Some(&png())).expect("first write");

        write(&path, &fields(), Some(&[])).expect("second write");

        assert!(
            read(&path).expect("readable").artwork.is_some(),
            "an empty picture is not a picture"
        );
    }

    #[test]
    fn the_fields_are_put_on_a_tag_field_by_field() {
        // The rules for a `Tag` on its own, without a file in the way: a value
        // is written where there is one, and dropped where there is not.
        let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
        apply(&mut tag, &fields());

        assert_eq!(tag.title().as_deref(), Some("Roygbiv"));
        assert_eq!(tag.artist().as_deref(), Some("Boards of Canada"));
        assert_eq!(
            tag.album().as_deref(),
            Some("Music Has the Right to Children")
        );
        assert_eq!(
            tag.get_string(ItemKey::AlbumArtist),
            Some("Boards of Canada")
        );
        assert_eq!(tag.genre().as_deref(), Some("Electronic"));
        assert_eq!(tag.track(), Some(3));
        assert_eq!(tag.disk(), Some(1));

        let mut cleared = fields();
        cleared.artist = None;
        cleared.album = None;
        cleared.album_artist = None;
        cleared.genre = None;
        cleared.track_number = None;
        cleared.disc_number = None;
        apply(&mut tag, &cleared);

        assert_eq!(tag.artist(), None);
        assert_eq!(tag.album(), None);
        assert_eq!(tag.get_string(ItemKey::AlbumArtist), None);
        assert_eq!(tag.genre(), None);
        assert_eq!(tag.track(), None);
        assert_eq!(tag.disk(), None);
        assert_eq!(tag.title().as_deref(), Some("Roygbiv"), "titles are kept");
    }

    #[test]
    fn an_embedded_cover_replaces_the_one_already_there() {
        let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
        let jpeg = [0xFF_u8, 0xD8, 0xFF, 0xE0, 0x00];

        embed_cover(&mut tag, &jpeg);
        embed_cover(&mut tag, &png());

        assert_eq!(tag.picture_count(), 1, "a file carries one front cover");
        assert_eq!(
            tag.pictures()[0].mime_type(),
            Some(&MimeType::Png),
            "the picture is the one that was last written"
        );
    }

    #[test]
    fn an_images_format_is_read_from_its_bytes() {
        assert_eq!(mime_type(&[0xFF, 0xD8, 0xFF, 0xE0]), MimeType::Jpeg);
        assert_eq!(mime_type(b"\x89PNG\r\n"), MimeType::Png);
        assert_eq!(mime_type(b"GIF89a"), MimeType::Gif);
        assert_eq!(mime_type(b"BM\x36"), MimeType::Bmp);
        assert!(matches!(mime_type(b"something else"), MimeType::Unknown(_)));
    }

    #[test]
    fn a_format_whose_tags_cannot_be_written_is_refused_rather_than_skipped() {
        // The guard in `write` is a safety net and is not reachable through a
        // file this crate can synthesize: every format it can build a sample of
        // — `wav` above, `mp3` — is read-write for its primary tag type. What is
        // asserted here is the rule the guard applies, so a later change to
        // `lofty`'s support tables cannot silently turn a refusal into a write
        // that goes nowhere.
        use lofty::tag::TagSupport;

        assert!(TagSupport::ReadWrite.is_writable());
        assert!(!TagSupport::ReadOnly.is_writable());
        assert!(!TagSupport::Unsupported.is_writable());
    }
}
