//! The metadata editor's model: a draft of the fields a user can change.
//!
//! The editor is built out of text fields, so what the user works with is
//! strings, and what the rest of the crate works with is [`TrackFields`] — the
//! same values, already parsed and checked. [`MetadataDraft::fields`] is the one
//! crossing point between the two, and it is where a bad track number or a
//! missing title is turned into a message next to the field that caused it.
//!
//! Nothing here touches a file or the database: a draft is just data, which is
//! why every rule below is testable without a window.

use std::path::PathBuf;

use crate::library::track::{Track, TrackFields, TrackId};

/// A field of the editor a validation failure can be attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// The display title.
    Title,
    /// Position within the disc.
    TrackNumber,
    /// Disc number.
    DiscNumber,
}

impl Field {
    /// Every field that can be reported on.
    pub const ALL: [Field; 3] = [Self::Title, Self::TrackNumber, Self::DiscNumber];

    /// Name shown in the editor.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::TrackNumber => "Track number",
            Self::DiscNumber => "Disc number",
        }
    }

    /// What the field accepts, in one line, for the editor's hint text.
    #[must_use]
    pub fn rule(self) -> &'static str {
        match self {
            Self::Title => "required",
            Self::TrackNumber | Self::DiscNumber => "a whole number, 1 or more, or empty",
        }
    }
}

/// What is wrong with one field of a draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// The field the message belongs beside.
    pub field: Field,
    /// What is wrong with it.
    pub message: String,
}

impl FieldError {
    /// Build an error for one field.
    #[must_use]
    pub fn new(field: Field, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }

    /// The message as the editor shows it: `Title: a track needs a title`.
    #[must_use]
    pub fn text(&self) -> String {
        format!("{}: {}", self.field.label(), self.message)
    }
}

/// A track being edited, as the editor's text fields hold it.
///
/// Every field is a `String` even where the stored value is optional, because a
/// blank text box is the natural way to say "this track has no album"; the
/// conversion back to `Option` happens once, in [`MetadataDraft::fields`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataDraft {
    id: TrackId,
    path: PathBuf,
    title: String,
    artist: String,
    album: String,
    album_artist: String,
    genre: String,
    track_number: String,
    disc_number: String,
    /// The cover the library currently shows for this track, if any.
    artwork: Option<PathBuf>,
    /// The picture the user chose to put in the file, if they chose one.
    replacement: Option<PathBuf>,
    /// Text in the editor's "use this image" field.
    artwork_input: String,
}

impl MetadataDraft {
    /// Start editing `track`, filling every field from what is known.
    #[must_use]
    pub fn new(track: &Track) -> Self {
        let fields = TrackFields::from_track(track);

        Self {
            id: track.id.unwrap_or(TrackId(0)),
            path: track.path.clone(),
            title: fields.title,
            artist: fields.artist.unwrap_or_default(),
            album: fields.album.unwrap_or_default(),
            album_artist: fields.album_artist.unwrap_or_default(),
            genre: fields.genre.unwrap_or_default(),
            track_number: fields
                .track_number
                .map_or_else(String::new, |n| n.to_string()),
            disc_number: fields
                .disc_number
                .map_or_else(String::new, |n| n.to_string()),
            artwork: track.artwork_path.clone(),
            replacement: None,
            artwork_input: String::new(),
        }
    }

    /// The row being edited.
    #[must_use]
    pub fn id(&self) -> TrackId {
        self.id
    }

    /// The file being edited.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// The title field.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The title field, for the view to edit.
    pub fn title_mut(&mut self) -> &mut String {
        &mut self.title
    }

    /// The artist field.
    #[must_use]
    pub fn artist(&self) -> &str {
        &self.artist
    }

    /// The artist field, for the view to edit.
    pub fn artist_mut(&mut self) -> &mut String {
        &mut self.artist
    }

    /// The album field.
    #[must_use]
    pub fn album(&self) -> &str {
        &self.album
    }

    /// The album field, for the view to edit.
    pub fn album_mut(&mut self) -> &mut String {
        &mut self.album
    }

    /// The album-artist field.
    #[must_use]
    pub fn album_artist(&self) -> &str {
        &self.album_artist
    }

    /// The album-artist field, for the view to edit.
    pub fn album_artist_mut(&mut self) -> &mut String {
        &mut self.album_artist
    }

    /// The genre field.
    #[must_use]
    pub fn genre(&self) -> &str {
        &self.genre
    }

    /// The genre field, for the view to edit.
    pub fn genre_mut(&mut self) -> &mut String {
        &mut self.genre
    }

    /// The track-number field.
    #[must_use]
    pub fn track_number(&self) -> &str {
        &self.track_number
    }

    /// The track-number field, for the view to edit.
    pub fn track_number_mut(&mut self) -> &mut String {
        &mut self.track_number
    }

    /// The disc-number field.
    #[must_use]
    pub fn disc_number(&self) -> &str {
        &self.disc_number
    }

    /// The disc-number field, for the view to edit.
    pub fn disc_number_mut(&mut self) -> &mut String {
        &mut self.disc_number
    }

    /// The cover the library is showing for this track.
    #[must_use]
    pub fn artwork(&self) -> Option<&PathBuf> {
        self.artwork.as_ref()
    }

    /// Point the draft at a cover file.
    pub fn set_artwork(&mut self, artwork: Option<PathBuf>) {
        self.artwork = artwork;
    }

    /// The picture the user chose as the file's own cover.
    #[must_use]
    pub fn replacement(&self) -> Option<&PathBuf> {
        self.replacement.as_ref()
    }

    /// Record the picture the user chose, which the caller has already copied
    /// into the artwork cache.
    pub fn set_replacement(&mut self, artwork: PathBuf) {
        self.artwork = Some(artwork.clone());
        self.replacement = Some(artwork);
    }

    /// Text in the editor's "use this image" field.
    #[must_use]
    pub fn artwork_input(&self) -> &str {
        &self.artwork_input
    }

    /// That field, for the view to edit.
    pub fn artwork_input_mut(&mut self) -> &mut String {
        &mut self.artwork_input
    }

    /// The validated fields this draft describes.
    ///
    /// # Errors
    ///
    /// Returns one [`FieldError`] per field that cannot be accepted, so the
    /// editor can mark all of them at once rather than making the user fix the
    /// form one message at a time. Nothing is written when a single field
    /// fails: a half-applied edit would leave the library disagreeing with
    /// itself.
    pub fn fields(&self) -> Result<TrackFields, Vec<FieldError>> {
        let mut errors = Vec::new();

        let title = self.title.trim().to_owned();
        if title.is_empty() {
            errors.push(FieldError::new(Field::Title, "a track needs a title"));
        }

        let track_number = number(&self.track_number, Field::TrackNumber, &mut errors);
        let disc_number = number(&self.disc_number, Field::DiscNumber, &mut errors);

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(TrackFields {
            title,
            artist: text(&self.artist),
            album: text(&self.album),
            album_artist: text(&self.album_artist),
            genre: text(&self.genre),
            track_number,
            disc_number,
        })
    }
}

/// Read one number field, recording why it was rejected.
///
/// An empty box is not a failure — plenty of files carry no track number — but a
/// word or a zero in it is, because writing either into a tag would produce a
/// track that sorts into nonsense.
fn number(value: &str, field: Field, errors: &mut Vec<FieldError>) -> Option<u32> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    match value.parse::<u32>() {
        Ok(number) if number >= 1 => Some(number),
        _ => {
            errors.push(FieldError::new(
                field,
                "use a whole number of 1 or more, or leave it empty",
            ));
            None
        }
    }
}

/// Read one free-form field, treating blank as "not set".
fn text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// The message the editor shows for a set of validation failures.
#[must_use]
pub fn field_errors(errors: &[FieldError]) -> String {
    errors
        .iter()
        .map(FieldError::text)
        .collect::<Vec<_>>()
        .join("; ")
}

/// What the editor last did, shown in the line under its buttons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The edit is in the library; the file has not been touched.
    Saved,
    /// The edit went into the library and into the file's own tags.
    Written,
    /// The last attempt failed, for the reason given.
    Failed(String),
}

impl Outcome {
    /// The line the editor shows for this outcome.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Saved => {
                "Saved to the library. The file's own tags are unchanged — use Write to file \
                 to put the values in the file itself."
                    .to_owned()
            }
            Self::Written => "Saved and written into the file's own tags.".to_owned(),
            Self::Failed(reason) => reason.clone(),
        }
    }

    /// Whether the line reports a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

/// An open editor: the draft, plus what the last action did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackEditor {
    draft: MetadataDraft,
    outcome: Option<Outcome>,
}

impl TrackEditor {
    /// Open the editor on `track`.
    #[must_use]
    pub fn new(track: &Track) -> Self {
        Self {
            draft: MetadataDraft::new(track),
            outcome: None,
        }
    }

    /// The draft being edited.
    #[must_use]
    pub fn draft(&self) -> &MetadataDraft {
        &self.draft
    }

    /// The draft, for the view to edit.
    pub fn draft_mut(&mut self) -> &mut MetadataDraft {
        &mut self.draft
    }

    /// The row being edited.
    #[must_use]
    pub fn track_id(&self) -> TrackId {
        self.draft.id()
    }

    /// The validated fields the draft describes.
    ///
    /// # Errors
    ///
    /// See [`MetadataDraft::fields`].
    pub fn fields(&self) -> Result<TrackFields, Vec<FieldError>> {
        self.draft.fields()
    }

    /// What the last action did.
    #[must_use]
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// Record what the last action did.
    pub fn set_outcome(&mut self, outcome: Outcome) {
        self.outcome = Some(outcome);
    }

    /// Forget what the last action did, which closes its message.
    pub fn clear_outcome(&mut self) {
        self.outcome = None;
    }

    /// Record a failure that has no fields to attach it to — a read-only file,
    /// a format whose tags cannot be written.
    pub fn report_failure(&mut self, reason: impl Into<String>) {
        self.set_outcome(Outcome::Failed(reason.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        let mut track = Track::from_path("/music/Boards of Canada/Roygbiv.mp3");
        track.id = Some(TrackId(7));
        track.title = "Roygbiv".to_owned();
        track.artist = Some("Boards of Canada".to_owned());
        track.album = Some("Music Has the Right to Children".to_owned());
        track.track_number = Some(3);
        track.artwork_path = Some(PathBuf::from("/cache/cover.jpg"));
        track
    }

    #[test]
    fn a_draft_starts_from_what_the_track_already_says() {
        let draft = MetadataDraft::new(&track());

        assert_eq!(draft.id(), TrackId(7));
        assert_eq!(draft.title(), "Roygbiv");
        assert_eq!(draft.artist(), "Boards of Canada");
        assert_eq!(draft.album(), "Music Has the Right to Children");
        assert_eq!(
            draft.album_artist(),
            "",
            "an unset tag reads as an empty box"
        );
        assert_eq!(draft.track_number(), "3");
        assert_eq!(draft.disc_number(), "");
        assert_eq!(draft.artwork(), Some(&PathBuf::from("/cache/cover.jpg")));
        assert!(draft.replacement().is_none());
    }

    #[test]
    fn a_draft_with_nothing_changed_validates_to_the_tracks_own_fields() {
        let track = track();
        let draft = MetadataDraft::new(&track);

        let fields = draft.fields().expect("the track's own fields are valid");

        assert!(fields.matches(&track));
    }

    #[test]
    fn whitespace_is_trimmed_out_of_every_field() {
        let mut draft = MetadataDraft::new(&track());
        *draft.title_mut() = "  Olson  ".to_owned();
        *draft.artist_mut() = "  Boards of Canada  ".to_owned();

        let fields = draft.fields().expect("valid");

        assert_eq!(fields.title, "Olson");
        assert_eq!(fields.artist.as_deref(), Some("Boards of Canada"));
    }

    #[test]
    fn an_empty_field_becomes_no_value_rather_than_an_empty_string() {
        let mut draft = MetadataDraft::new(&track());
        *draft.album_mut() = "   ".to_owned();
        *draft.track_number_mut() = String::new();

        let fields = draft.fields().expect("valid");

        assert_eq!(fields.album, None);
        assert_eq!(fields.track_number, None);
    }

    #[test]
    fn a_track_without_a_title_is_refused() {
        let mut draft = MetadataDraft::new(&track());
        *draft.title_mut() = "   ".to_owned();

        let errors = draft.fields().expect_err("a title is required");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, Field::Title);
        assert!(errors[0].text().contains("needs a title"));
    }

    #[test]
    fn a_number_field_accepts_only_whole_numbers_from_one() {
        let mut draft = MetadataDraft::new(&track());

        for bad in ["zero", "3.5", "0", "-1", "1 2"] {
            *draft.track_number_mut() = bad.to_owned();
            let errors = draft
                .fields()
                .expect_err("`{bad}` is not a track number")
                .into_iter()
                .map(|error| error.field)
                .collect::<Vec<_>>();

            assert_eq!(errors, vec![Field::TrackNumber], "for `{bad}`");
        }

        *draft.track_number_mut() = "12".to_owned();
        assert_eq!(draft.fields().expect("valid").track_number, Some(12));
    }

    #[test]
    fn every_broken_field_is_reported_at_once() {
        let mut draft = MetadataDraft::new(&track());
        *draft.title_mut() = String::new();
        *draft.disc_number_mut() = "two".to_owned();

        let errors = draft.fields().expect_err("both fields are wrong");
        let fields: Vec<Field> = errors.iter().map(|error| error.field).collect();

        assert_eq!(fields, vec![Field::Title, Field::DiscNumber]);
        assert_eq!(
            field_errors(&errors),
            "Title: a track needs a title; Disc number: use a whole number of 1 or more, \
             or leave it empty"
        );
    }

    #[test]
    fn every_field_has_a_label_and_a_rule_to_show() {
        for field in Field::ALL {
            assert!(!field.label().is_empty(), "{field:?} has no label");
            assert!(!field.rule().is_empty(), "{field:?} states no rule");
        }
    }

    #[test]
    fn choosing_a_picture_updates_the_cover_and_is_the_one_that_gets_written() {
        let mut draft = MetadataDraft::new(&track());
        let chosen = PathBuf::from("/cache/chosen.png");

        draft.set_replacement(chosen.clone());

        assert_eq!(draft.replacement(), Some(&chosen));
        assert_eq!(
            draft.artwork(),
            Some(&chosen),
            "the cover the library shows follows the choice"
        );
    }

    #[test]
    fn an_editor_reports_what_it_last_did() {
        let mut editor = TrackEditor::new(&track());
        assert!(editor.outcome().is_none());
        assert_eq!(editor.track_id(), TrackId(7));

        editor.set_outcome(Outcome::Written);
        assert_eq!(editor.outcome(), Some(&Outcome::Written));
        assert!(!editor.outcome().expect("set").is_failure());
        assert!(
            editor
                .outcome()
                .expect("set")
                .message()
                .contains("written into the file")
        );

        editor.report_failure("`/music/a.mp3` is read-only");
        let outcome = editor.outcome().expect("set");
        assert!(outcome.is_failure());
        assert_eq!(outcome.message(), "`/music/a.mp3` is read-only");

        editor.clear_outcome();
        assert!(editor.outcome().is_none());
    }

    #[test]
    fn saving_to_the_library_alone_says_the_file_was_not_touched() {
        let message = Outcome::Saved.message();

        assert!(message.contains("library"), "{message}");
        assert!(message.contains("file"), "{message}");
    }
}
