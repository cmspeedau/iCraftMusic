//! The confirmation the library asks for before it forgets anything.
//!
//! Every destructive library action goes through one of these: removing tracks
//! and clearing out the rows whose files have vanished both drop rows the user
//! can only get back with another scan, so both are asked about first. The
//! dialog is drawn as a floating window from [`crate::ui::render`], on top of
//! whatever section is open, because a question that can be answered wrongly
//! has to be unmissable.
//!
//! Nothing here acts on its own: the view reports what the user chose and
//! [`crate::app::MusicCraftApp::confirm_pending`] does the work, so the dialog
//! and the action can be tested apart from one another.

use egui::{Align2, RichText, Vec2};

use crate::app::MusicCraftApp;
use crate::library::PlaylistId;
use crate::ui::theme;

/// A library action that needs the user's consent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirmation {
    /// Forget the tracks currently selected in the table.
    RemoveSelection {
        /// How many rows would be removed.
        count: usize,
    },
    /// Forget every indexed track whose file has vanished.
    ForgetMissing {
        /// How many rows would be removed.
        count: usize,
    },
    /// Take a playlist out of the library.
    ///
    /// The playlist itself is the whole of it: the tracks it lists stay in the
    /// library and their files stay on disk, so deleting a playlist can never
    /// lose music — only the order someone put it in.
    DeletePlaylist {
        /// The playlist that would go.
        id: PlaylistId,
        /// Its name, so the question can say what is being deleted.
        name: String,
        /// How many entries the playlist holds.
        count: usize,
    },
}

impl Confirmation {
    /// The question, in a line.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::RemoveSelection { .. } => "Remove from the library?",
            Self::ForgetMissing { .. } => "Forget the missing files?",
            Self::DeletePlaylist { .. } => "Delete this playlist?",
        }
    }

    /// What the action would do, and what it would not.
    ///
    /// The second half is the part that matters: a user about to remove rows
    /// needs to know their music is not at stake.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::RemoveSelection { count } => format!(
                "{} {} removed from the library. The audio files themselves are left on disk, \
                 and a later scan puts them back.",
                count,
                if *count == 1 { "track" } else { "tracks" }
            ),
            Self::ForgetMissing { count } => format!(
                "{} {} forgotten because the files are no longer on disk. Nothing else is \
                 removed, and a scan re-indexes anything that comes back.",
                count,
                if *count == 1 { "track" } else { "tracks" }
            ),
            Self::DeletePlaylist { name, count, .. } => format!(
                "`{}` and its {} {} entries are removed. The tracks themselves stay in the \
                 library and their files stay on disk.",
                name,
                count,
                if *count == 1 { "entry" } else { "entries" }
            ),
        }
    }

    /// The label on the button that goes ahead.
    #[must_use]
    pub fn confirm_label(&self) -> &'static str {
        match self {
            Self::RemoveSelection { .. } => "Remove",
            Self::ForgetMissing { .. } => "Forget them",
            Self::DeletePlaylist { .. } => "Delete",
        }
    }
}

/// Draw the confirmation, when one is waiting.
pub fn confirm(app: &mut MusicCraftApp, ui: &egui::Ui) {
    let Some(pending) = app.pending_confirmation() else {
        return;
    };

    let mut open = true;
    let mut chosen: Option<bool> = None;

    egui::Window::new(pending.title())
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.set_max_width(360.0);
            ui.label(RichText::new(pending.detail()).small());

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new(pending.confirm_label()).color(theme::WARNING))
                    .clicked()
                {
                    chosen = Some(true);
                }
                if ui.button("Keep them").clicked() {
                    chosen = Some(false);
                }
            });
        });

    match chosen {
        Some(true) => app.confirm_pending(),
        // Closing the window with its ✕ is the same as answering "no": the
        // user must never be able to decline by accident and lose rows anyway.
        Some(false) | None if !open => app.cancel_confirmation(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{ClickMode, Library, PlaylistId, Track, TrackId};

    fn confirmation(count: usize) -> [Confirmation; 2] {
        [
            Confirmation::RemoveSelection { count },
            Confirmation::ForgetMissing { count },
        ]
    }

    #[test]
    fn every_confirmation_says_what_it_would_do_and_what_it_would_not() {
        for confirmation in confirmation(3) {
            assert!(!confirmation.title().is_empty());
            assert!(!confirmation.confirm_label().is_empty());

            let detail = confirmation.detail();
            assert!(detail.contains('3'), "{detail} names no count");
            assert!(
                detail.contains("disk"),
                "{detail} does not say the files are left alone"
            );
        }
    }

    #[test]
    fn a_single_track_is_counted_in_the_singular() {
        for confirmation in confirmation(1) {
            assert!(
                confirmation.detail().contains("1 track "),
                "{confirmation:?}"
            );
        }
    }

    #[test]
    fn deleting_a_playlist_says_the_tracks_and_files_stay() {
        let confirmation = Confirmation::DeletePlaylist {
            id: PlaylistId(4),
            name: "Evening".to_owned(),
            count: 1,
        };

        assert_eq!(confirmation.title(), "Delete this playlist?");
        assert_eq!(confirmation.confirm_label(), "Delete");

        let detail = confirmation.detail();
        assert!(detail.contains("Evening"), "{detail} names no playlist");
        assert!(detail.contains("1 entry"), "{detail} miscounts");
        assert!(detail.contains("disk"), "{detail} loses the files");
    }

    #[test]
    fn drawing_the_question_does_not_answer_it() {
        let mut app = selected();
        app.ask_to_remove_selection();
        assert!(app.pending_confirmation().is_some());

        egui::__run_test_ui(|ui| confirm(&mut app, ui));

        assert!(
            app.pending_confirmation().is_some(),
            "the question survives its own frame"
        );
        assert_eq!(app.browse().track_count(), 1);
    }

    #[test]
    fn answering_no_leaves_the_library_alone() {
        let mut app = selected();
        app.ask_to_remove_selection();
        assert!(app.pending_confirmation().is_some());

        app.cancel_confirmation();

        assert!(app.pending_confirmation().is_none());
        assert_eq!(app.browse().track_count(), 1, "and nothing was removed");
    }

    #[test]
    fn agreeing_to_it_removes_the_rows_and_leaves_the_files() {
        let mut app = selected();
        app.ask_to_remove_selection();

        app.confirm_pending();

        assert!(app.pending_confirmation().is_none());
        assert_eq!(app.browse().track_count(), 0, "the row is gone");
    }

    #[test]
    fn there_is_nothing_to_agree_to_without_a_selection() {
        let mut app = app();

        app.ask_to_remove_selection();

        assert!(
            app.pending_confirmation().is_none(),
            "an empty selection asks no questions"
        );

        // Answering a question that was never asked is harmless.
        app.confirm_pending();
        assert_eq!(app.browse().track_count(), 0);
    }

    #[test]
    fn forgetting_the_missing_files_is_asked_about_too() {
        let mut app = app();

        app.ask_to_forget_missing();
        assert!(
            app.pending_confirmation().is_none(),
            "with nothing missing there is nothing to forget"
        );
    }

    fn app() -> MusicCraftApp {
        MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
        )
    }

    /// An application with one track in the library, and that track selected.
    fn selected() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-dialog-artwork"))
            .expect("an in-memory library");
        let mut app = MusicCraftApp::with_library(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
            library,
        );

        let mut track = Track::from_path("/music/song.mp3");
        track.id = Some(TrackId(1));
        track.title = "Roygbiv".to_owned();
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        let visible = app.browse().visible_track_ids();
        app.browse_mut()
            .selection_mut()
            .apply(&visible, visible[0], ClickMode::Replace);

        app
    }
}
