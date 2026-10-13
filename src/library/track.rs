//! Domain models for the indexed music library.
//!
//! These types are plain data: they carry no database handles and no file
//! handles, so they can be created by the scanner, stored by the repository and
//! handed to the interface layer without further conversion. Nineteen of the
//! fields on a track are optional because real files are messy — a track with
//! no tags at all is still a track, and the interface decides how to present
//! the gaps.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Identifier of a row in the `tracks` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackId(
    /// The primary key of the row.
    pub i64,
);

impl TrackId {
    /// The raw value, for binding into SQL statements.
    #[must_use]
    pub fn get(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for TrackId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Identifier of a row in the `playlists` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaylistId(
    /// The primary key of the row.
    pub i64,
);

impl PlaylistId {
    /// The raw value, for binding into SQL statements.
    #[must_use]
    pub fn get(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// A single audio file and everything the library knows about it.
///
/// `id` is `None` until the repository has stored the track; the scanner works
/// with the same type so a fresh scan and a row read back from SQLite look
/// alike.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    /// Row identifier, or `None` for a track that has not been stored yet.
    pub id: Option<TrackId>,
    /// Absolute path to the audio file.
    pub path: PathBuf,
    /// Display title. Falls back to the file stem when the file carries no
    /// title tag.
    pub title: String,
    /// Performing artist.
    pub artist: Option<String>,
    /// Album the track belongs to.
    pub album: Option<String>,
    /// Artist credited for the whole album, when it differs from the performer.
    pub album_artist: Option<String>,
    /// Free-form genre, as written by the tagger.
    pub genre: Option<String>,
    /// Position of the track within its disc, starting at one.
    pub track_number: Option<u32>,
    /// Disc number for multi-disc albums, starting at one.
    pub disc_number: Option<u32>,
    /// Playing time reported by the decoder, when it could be determined.
    pub duration: Option<Duration>,
    /// Cached cover art extracted from the file's tags.
    pub artwork_path: Option<PathBuf>,
    /// Size of the file in bytes at the time of the last scan.
    pub size_bytes: Option<u64>,
    /// Modification time in seconds since the Unix epoch at the last scan.
    pub modified_at: Option<i64>,
    /// Whether the file has disappeared since it was indexed.
    pub missing: bool,
    /// Whether the row has been edited in the library while the file still
    /// carries the values the last scan read.
    ///
    /// The flag is what lets the interface say "the file does not know about
    /// this edit yet": a rescan leaves an edited row alone as long as the file
    /// itself is unchanged, and writing the tags to the file clears the flag.
    pub tags_dirty: bool,
}

impl Track {
    /// Build a track from nothing but a path.
    ///
    /// The title is taken from the file stem and every tag-derived field is
    /// left empty, which is what the scanner falls back to when a file cannot
    /// be read as audio at all.
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            id: None,
            title: display_title(&path),
            path,
            artist: None,
            album: None,
            album_artist: None,
            genre: None,
            track_number: None,
            disc_number: None,
            duration: None,
            artwork_path: None,
            size_bytes: None,
            modified_at: None,
            missing: false,
            tags_dirty: false,
        }
    }

    /// The artist to show for this track, or `None` when nothing is known.
    ///
    /// Falls back to the album artist so a compilation with only an
    /// album-level credit still groups sensibly.
    #[must_use]
    pub fn display_artist(&self) -> Option<&str> {
        self.artist.as_deref().or(self.album_artist.as_deref())
    }

    /// Whether two tracks describe the same audio content.
    ///
    /// Used by the repository to decide whether an upsert actually changed
    /// anything; the row identifier, the artwork cache path and the
    /// [`Track::tags_dirty`] flag are ignored because they are managed by the
    /// database, not the file. That is what keeps a hand-edited row from being
    /// overwritten by a scan of the file it describes.
    #[must_use]
    pub fn same_content(&self, other: &Self) -> bool {
        self.path == other.path
            && self.title == other.title
            && self.artist == other.artist
            && self.album == other.album
            && self.album_artist == other.album_artist
            && self.genre == other.genre
            && self.track_number == other.track_number
            && self.disc_number == other.disc_number
            && self.duration == other.duration
            && self.size_bytes == other.size_bytes
            && self.modified_at == other.modified_at
    }
}

/// The tag-derived fields the user can edit, already validated.
///
/// The metadata editor holds a draft made of strings, because that is what a
/// text field gives back; this type is what a validated draft becomes. Parsing
/// and checking therefore happen once, at the edge of the interface, and the
/// write paths — the database and the file's tags — only ever see values that
/// are already the right shape.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackFields {
    /// Display title. Never empty: a track with no title is not a track.
    pub title: String,
    /// Performing artist.
    pub artist: Option<String>,
    /// Album the track belongs to.
    pub album: Option<String>,
    /// Artist credited for the whole album.
    pub album_artist: Option<String>,
    /// Free-form genre.
    pub genre: Option<String>,
    /// Position of the track within its disc, starting at one.
    pub track_number: Option<u32>,
    /// Disc number for multi-disc albums, starting at one.
    pub disc_number: Option<u32>,
}

impl TrackFields {
    /// The fields a track currently holds.
    #[must_use]
    pub fn from_track(track: &Track) -> Self {
        Self {
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            album_artist: track.album_artist.clone(),
            genre: track.genre.clone(),
            track_number: track.track_number,
            disc_number: track.disc_number,
        }
    }

    /// Whether every field already equals the track's own.
    #[must_use]
    pub fn matches(&self, track: &Track) -> bool {
        *self == Self::from_track(track)
    }

    /// Copy the fields onto a track.
    ///
    /// Everything the tags do not describe — the path, the playing time, the
    /// cached cover art, the missing flag — is left exactly as it was.
    pub fn apply_to(&self, track: &mut Track) {
        track.title = self.title.clone();
        track.artist = self.artist.clone();
        track.album = self.album.clone();
        track.album_artist = self.album_artist.clone();
        track.genre = self.genre.clone();
        track.track_number = self.track_number;
        track.disc_number = self.disc_number;
    }
}

/// An album, derived from the tracks that make it up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    /// Album title.
    pub name: String,
    /// Album artist, when the tracks agree on one.
    pub album_artist: Option<String>,
    /// Number of indexed tracks on the album.
    pub track_count: usize,
    /// Total playing time of the indexed tracks, when known.
    pub duration: Option<Duration>,
}

/// An artist, derived from the tracks credited to them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    /// Artist name.
    pub name: String,
    /// Number of distinct albums the artist appears on.
    pub album_count: usize,
    /// Number of indexed tracks credited to the artist.
    pub track_count: usize,
}

/// The title to show for a path that carries no tags.
///
/// Falls back to the whole path when the file has no stem at all, so the string
/// is always non-empty and never panics.
#[must_use]
pub fn display_title(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty());

    match stem {
        Some(stem) => stem.to_owned(),
        None => path.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path_uses_the_file_stem_as_the_title() {
        let track = Track::from_path("/music/Boards of Canada/Roygbiv.mp3");

        assert_eq!(track.title, "Roygbiv");
        assert_eq!(
            track.path,
            PathBuf::from("/music/Boards of Canada/Roygbiv.mp3")
        );
        assert!(track.id.is_none());
        assert!(track.artist.is_none());
        assert!(!track.missing);
    }

    #[test]
    fn from_path_never_yields_an_empty_title() {
        let track = Track::from_path("/music/.mp3");
        assert!(!track.title.is_empty());
    }

    #[test]
    fn display_artist_prefers_the_track_artist() {
        let mut track = Track::from_path("/music/a.flac");
        track.artist = Some("Aphex Twin".to_owned());
        track.album_artist = Some("Various Artists".to_owned());

        assert_eq!(track.display_artist(), Some("Aphex Twin"));
    }

    #[test]
    fn display_artist_falls_back_to_the_album_artist() {
        let mut track = Track::from_path("/music/a.flac");
        track.album_artist = Some("Various Artists".to_owned());

        assert_eq!(track.display_artist(), Some("Various Artists"));
    }

    #[test]
    fn same_content_ignores_the_identifier_and_artwork() {
        let first = Track::from_path("/music/a.flac");
        let mut second = first.clone();
        second.id = Some(TrackId(7));
        second.artwork_path = Some(PathBuf::from("/cache/cover.jpg"));

        assert!(first.same_content(&second));

        second.title = "Different".to_owned();
        assert!(!first.same_content(&second));
    }

    #[test]
    fn a_hand_edited_row_is_not_mistaken_for_a_changed_file() {
        let scanned = Track::from_path("/music/a.flac");
        let mut stored = scanned.clone();
        stored.tags_dirty = true;

        assert!(
            scanned.same_content(&stored),
            "the edit flag belongs to the row, not to the file"
        );
    }

    #[test]
    fn the_fields_of_a_track_round_trip_through_the_editor_model() {
        let mut track = Track::from_path("/music/a.flac");
        track.artist = Some("Aphex Twin".to_owned());
        track.track_number = Some(3);

        let fields = TrackFields::from_track(&track);
        assert!(fields.matches(&track));

        let mut edited = fields.clone();
        edited.title = "Xtal".to_owned();
        edited.album = Some("Selected Ambient Works".to_owned());
        edited.track_number = Some(4);

        assert!(!edited.matches(&track));

        edited.apply_to(&mut track);
        assert!(edited.matches(&track));
        assert_eq!(track.title, "Xtal");
        assert_eq!(track.album.as_deref(), Some("Selected Ambient Works"));
        assert_eq!(track.track_number, Some(4));
        assert_eq!(
            track.path,
            PathBuf::from("/music/a.flac"),
            "editing tags never moves the file"
        );
    }

    #[test]
    fn applying_fields_leaves_the_playing_time_and_artwork_alone() {
        let mut track = Track::from_path("/music/a.flac");
        track.duration = Some(Duration::from_secs(200));
        track.artwork_path = Some(PathBuf::from("/cache/cover.jpg"));
        track.missing = true;

        TrackFields::default().apply_to(&mut track);

        assert_eq!(track.duration, Some(Duration::from_secs(200)));
        assert_eq!(track.artwork_path, Some(PathBuf::from("/cache/cover.jpg")));
        assert!(track.missing, "an edit is not a rescan");
    }
}
