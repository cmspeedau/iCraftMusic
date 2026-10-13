//! The playlist name editor: the one field creating, renaming and copying need.
//!
//! Creating a playlist and renaming one are the same question — what should this
//! be called — so they are asked in the same window, differing only in their
//! title and the label on the button. The draft and its rules live in
//! [`crate::playlists::edit`]; this module draws them and reports which button
//! was pressed, so what can be typed stays testable without a window.
//!
//! A name already in use comes back as a message beside the field rather than as
//! a closed window: the typed name is still there, and the user's next move is
//! obvious.

use egui::{Align2, RichText, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::ui::theme;

/// Width of the window, in points.
const WIDTH: f32 = 360.0;
/// Height of the name field.
const FIELD_HEIGHT: f32 = 22.0;

/// Draw the name editor, when a playlist is being made or renamed.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if app.playlist_draft().is_none() {
        return;
    }

    let mut open = true;
    let mut confirm = false;
    let mut cancel = false;

    egui::Window::new(title(app))
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            let (confirm_label, error) = body(app, ui);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(confirm_label).clicked() {
                    confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });

            if let Some(message) = error {
                ui.add_space(6.0);
                ui.label(RichText::new(message).small().color(theme::WARNING));
            }
        });

    if confirm {
        app.confirm_playlist_edit();
    } else if cancel || !open {
        app.cancel_playlist_edit();
    }
}

/// The window's title, taken from what the editor is doing.
fn title(app: &MusicCraftApp) -> &'static str {
    app.playlist_draft()
        .map_or("Playlist", |draft| draft.kind().title())
}

/// Draw the field, returning the button's label and any failure to show.
fn body(app: &mut MusicCraftApp, ui: &mut egui::Ui) -> (&'static str, Option<String>) {
    ui.set_max_width(WIDTH);

    let Some(draft) = app.playlist_draft_mut() else {
        return ("Save", None);
    };

    let label = draft.kind().confirm_label();
    let error = draft.error().map(str::to_owned);

    ui.label(RichText::new("Name").small().weak());
    let field = ui.add_sized(
        [ui.available_width(), FIELD_HEIGHT],
        TextEdit::singleline(draft.name_mut()).hint_text("Evening"),
    );

    // The Enter key does what the confirm button does, which is what a one-field
    // dialog should do.
    if field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        app.confirm_playlist_edit();
    }

    ui.add_space(4.0);
    ui.label(
        RichText::new("Tracks are added to it from the Library's right-click menu.")
            .small()
            .weak(),
    );

    (label, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::playlists::PlaylistEditKind;

    #[test]
    fn drawing_without_an_editor_is_harmless() {
        let mut app = app();

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(app.playlist_draft().is_none());
    }

    #[test]
    fn the_window_is_titled_for_what_it_is_doing() {
        let mut app = app();

        app.start_playlist_edit(PlaylistEditKind::Create);
        assert_eq!(title(&app), "New playlist");

        app.cancel_playlist_edit();
        app.start_playlist_edit(PlaylistEditKind::Rename(crate::library::PlaylistId(1)));
        assert_eq!(title(&app), "Rename playlist");
    }

    #[test]
    fn drawing_the_editor_does_not_make_a_playlist() {
        let mut app = app();
        app.start_playlist_edit(PlaylistEditKind::Create);

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(
            app.playlist_draft().is_some(),
            "the editor survives its own frame"
        );
        assert!(app.playlists().is_empty(), "and nothing was written");
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
}
