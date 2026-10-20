//! The track tag editor: what is open, what saves, what is written to the file.

use super::MusicCraftApp;
use crate::error::{AppError, Result};
use crate::library::edit::{Outcome, field_errors};
use crate::library::{TrackEditor, TrackId};
use crate::ui;
use std::path::{Path, PathBuf};

impl MusicCraftApp {
    /// Open the metadata editor on one track of the library.
    ///
    /// Returns whether there was such a track. The editor starts from what the
    /// row currently says rather than from the file, so a user who edits twice
    /// keeps editing what they last typed.
    pub fn open_editor(&mut self, id: TrackId) -> bool {
        let Some(track) = self.track_of(id) else {
            return false;
        };

        self.editor = Some(TrackEditor::new(&track));
        true
    }

    /// Open the editor on the selection, when exactly one track is selected.
    ///
    /// Returns whether an editor was opened: a multi-track selection has no one
    /// set of tags to edit, so it opens nothing rather than guessing.
    pub fn edit_selection(&mut self) -> bool {
        let ids: Vec<TrackId> = self.browse.selection().ids().collect();
        match ids.as_slice() {
            [id] => self.open_editor(*id),
            _ => false,
        }
    }

    /// The open metadata editor, if there is one.
    #[must_use]
    pub fn editor(&self) -> Option<&TrackEditor> {
        self.editor.as_ref()
    }

    /// The open metadata editor, mutably, for the view's text fields.
    pub fn editor_mut(&mut self) -> Option<&mut TrackEditor> {
        self.editor.as_mut()
    }

    /// Close the metadata editor, discarding whatever was typed into it.
    pub fn close_editor(&mut self) {
        self.editor = None;
    }

    /// Save the editor's fields into the library, leaving the file alone.
    ///
    /// The file keeps the tags it had; the row records that the two now differ,
    /// which is what the marker in the track table is for.
    pub fn save_editor(&mut self) {
        let Some(id) = self.editor.as_ref().map(TrackEditor::track_id) else {
            return;
        };

        let Some((fields, artwork)) = self.editor_fields() else {
            return;
        };

        // Nothing typed and nothing to record: the row already says this, so
        // there is no edit to store and no reason to mark the file as behind.
        if self
            .track_of(id)
            .is_some_and(|track| fields.matches(&track) && track.artwork_path == artwork)
        {
            if let Some(editor) = self.editor.as_mut() {
                editor.set_outcome(Outcome::Saved);
            }
            return;
        }

        self.apply_edit(id, &fields, artwork.as_deref(), false);
    }

    /// Save the editor's fields *into the file*, rewriting its tags.
    ///
    /// This is the only path in the application that writes to a user's audio
    /// file, and it runs only when the user presses the button that says so. A
    /// failure — a read-only file, a format whose tags cannot be written — is
    /// reported in the editor, and the library is left as it was.
    pub fn write_editor_tags(&mut self) {
        let Some(id) = self.editor.as_ref().map(TrackEditor::track_id) else {
            return;
        };

        let Some((fields, artwork)) = self.editor_fields() else {
            return;
        };

        let Some(track) = self.track_of(id) else {
            self.report_editor_failure("that track is no longer in the library.");
            return;
        };

        if track.missing {
            self.report_editor_failure(
                "the file is not on disk, so there are no tags to write into it.",
            );
            return;
        }

        // Only a picture the user chose is embedded; the file's own cover is
        // left alone by anything the editor does without being asked.
        let picture = match self.chosen_picture() {
            Ok(picture) => picture,
            Err(error) => {
                self.report_editor_failure(error.to_string());
                return;
            }
        };

        match crate::library::metadata::write(&track.path, &fields, picture.as_deref()) {
            Ok(()) => self.apply_edit(id, &fields, artwork.as_deref(), true),
            Err(error) => self.report_editor_failure(error.to_string()),
        }
    }

    /// Take the image named in the editor as this track's cover.
    ///
    /// The picture is copied into the artwork cache under the track's own name,
    /// so it survives the original being moved, and the editor shows it at once.
    /// Nothing is written to the file until the user asks for that.
    pub fn use_editor_artwork(&mut self) {
        let Some(editor) = self.editor.as_ref() else {
            return;
        };

        let typed = editor.draft().artwork_input().trim().to_owned();
        if typed.is_empty() {
            self.report_editor_failure("type the path of an image first.");
            return;
        }

        let source = PathBuf::from(&typed);
        let image = match std::fs::read(&source) {
            Ok(image) => image,
            Err(error) => {
                self.report_editor_failure(AppError::io(&source, error).to_string());
                return;
            }
        };

        // The bytes are the authority on whether this is an image at all: a
        // file named `.jpg` that holds something else is refused here rather
        // than being cached and shown as a broken tile.
        if ui::artwork::decode(&image).is_none() {
            self.report_editor_failure(AppError::not_an_image(&source).to_string());
            return;
        }

        let track_path = editor.draft().path().clone();
        let directory = self.artwork_directory();

        match crate::library::artwork::store(&directory, &track_path, &image) {
            Ok(cached) => {
                // The cache file is named after the track, so a second choice
                // reuses the same name: the decoded cover has to be dropped or
                // the album view would keep drawing the previous picture.
                self.artwork.forget(&cached);

                if let Some(editor) = self.editor.as_mut() {
                    editor.draft_mut().set_replacement(cached);
                    editor.draft_mut().artwork_input_mut().clear();
                    editor.clear_outcome();
                }
            }
            Err(error) => self.report_editor_failure(error.to_string()),
        }
    }

    /// The validated fields and cover of the open editor.
    ///
    /// Returns `None` when no editor is open, or when what was typed does not
    /// pass validation — in which case the editor is left showing why, and
    /// nothing has been written anywhere.
    fn editor_fields(&mut self) -> Option<(crate::library::TrackFields, Option<PathBuf>)> {
        let editor = self.editor.as_ref()?;

        match editor.fields() {
            Ok(fields) => Some((fields, editor.draft().artwork().cloned())),
            Err(errors) => {
                self.report_editor_failure(field_errors(&errors));
                None
            }
        }
    }

    /// The picture the user chose in the open editor, read ready to embed.
    ///
    /// # Errors
    ///
    /// Returns the read failure when the chosen image has gone since it was
    /// picked, which the editor reports the same way as any other failure.
    fn chosen_picture(&self) -> Result<Option<Vec<u8>>> {
        let Some(path) = self
            .editor
            .as_ref()
            .and_then(|editor| editor.draft().replacement())
        else {
            return Ok(None);
        };

        std::fs::read(path)
            .map(Some)
            .map_err(|error| AppError::io(path, error))
    }

    /// Store what the user typed and bring the library view up to date.
    fn apply_edit(
        &mut self,
        id: TrackId,
        fields: &crate::library::TrackFields,
        artwork: Option<&Path>,
        written: bool,
    ) {
        let stored = match self.library.as_ref() {
            Some(library) => library
                .repository()
                .apply_edit(id, fields, artwork, written),
            None => {
                self.report_editor_failure("there is no library to save into.");
                return;
            }
        };

        match stored {
            Ok(true) => {
                self.settle_editor(written);
                if let Err(error) = self.refresh() {
                    self.library_error = Some(error.to_string());
                }
            }
            Ok(false) => self.report_editor_failure("that track is no longer in the library."),
            Err(error) => self.report_editor_failure(error.to_string()),
        }
    }

    /// Tell the open editor what just happened to the row it is editing.
    fn settle_editor(&mut self, written: bool) {
        if let Some(editor) = self.editor.as_mut() {
            editor.set_outcome(if written {
                Outcome::Written
            } else {
                Outcome::Saved
            });
        }
    }

    /// Show a failure in the open editor.
    fn report_editor_failure(&mut self, reason: impl Into<String>) {
        if let Some(editor) = self.editor.as_mut() {
            editor.report_failure(reason);
        }
    }

    /// Where chosen cover art is cached, following the open library when there
    /// is one and falling back to the scratch directory when there is not.
    pub(super) fn artwork_directory(&self) -> PathBuf {
        self.library.as_ref().map_or_else(
            || std::env::temp_dir().join(crate::library::artwork::ARTWORK_DIRECTORY),
            |library| library.artwork_directory().to_path_buf(),
        )
    }
}
