//! The metadata editor: one track's tags, as a form.
//!
//! The window is deliberately the only place in the interface that can change
//! what a file says, and it does that only through the button that names the
//! file: **Save** writes the values into the library, **Write to file** writes
//! them into the audio file's own tags. Nothing else in the application touches
//! a user's tags, which is why the difference between the two buttons is
//! spelled out underneath them rather than left to the tooltips.
//!
//! The draft, its validation and its outcome all live in
//! [`crate::library::edit`]; this module draws them and reports which button was
//! pressed, so the interesting behaviour stays testable without a window.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::library::edit::Field;
use crate::ui::theme;

/// Width of the editor window, in points.
const WIDTH: f32 = 440.0;
/// Height of one text field.
const FIELD_HEIGHT: f32 = 22.0;
/// Size of the square the cover art is drawn in.
const COVER_SIZE: f32 = 104.0;
/// The two number fields sit side by side, so they are narrower than the rest.
const NUMBER_WIDTH: f32 = 90.0;

/// What the body asked for, applied once the form has been drawn.
///
/// A click is recorded rather than acted on, so the draft is not mutated while
/// the widgets that read it are still being built.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Actions {
    save: bool,
    write_to_file: bool,
    use_image: bool,
    close: bool,
}

/// Draw the editor, when one is open.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if app.editor().is_none() {
        return;
    }

    let mut open = true;
    egui::Window::new("Edit details")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            let actions = body(app, ui);
            apply(app, actions);
        });

    // The ✕ in the window's corner closes the editor without saving anything,
    // exactly like the Cancel button beside Save.
    if !open {
        app.close_editor();
    }
}

/// Draw the form and report what was clicked.
fn body(app: &mut MusicCraftApp, ui: &mut egui::Ui) -> Actions {
    ui.set_max_width(WIDTH);

    let Some(editor) = app.editor() else {
        return Actions::default();
    };

    let path = editor.draft().path().clone();
    let artwork = editor.draft().artwork().cloned();
    let track_id = editor.track_id();
    let edited = app.track_of(track_id).is_some_and(|track| track.tags_dirty);

    ui.label(
        RichText::new(path.display().to_string())
            .small()
            .weak()
            .monospace(),
    );

    if edited {
        ui.label(RichText::new(EDIT_NOT_WRITTEN).small().color(theme::ACCENT));
    }

    ui.add_space(6.0);
    cover(app, ui, artwork.as_ref());

    let mut actions = Actions::default();
    ui.add_space(6.0);

    if let Some(editor) = app.editor_mut() {
        let draft = editor.draft_mut();
        let width = ui.available_width();

        text_field(ui, "Title", Field::Title.rule(), draft.title_mut(), width);
        text_field(ui, "Artist", "who performs it", draft.artist_mut(), width);
        text_field(
            ui,
            "Album",
            "the release it came from",
            draft.album_mut(),
            width,
        );
        text_field(
            ui,
            "Album artist",
            "used to group a compilation",
            draft.album_artist_mut(),
            width,
        );
        text_field(ui, "Genre", "free-form", draft.genre_mut(), width);

        ui.horizontal(|ui| {
            number_field(ui, "Track number", draft.track_number_mut());
            number_field(ui, "Disc number", draft.disc_number_mut());
        });

        ui.add_space(6.0);
        ui.label(RichText::new("Cover art").small().weak());
        ui.horizontal(|ui| {
            let field_width = (ui.available_width() - 96.0).max(120.0);
            ui.add_sized(
                [field_width, FIELD_HEIGHT],
                TextEdit::singleline(draft.artwork_input_mut()).hint_text("/home/you/cover.png"),
            );
            if ui.button("Use image").clicked() {
                actions.use_image = true;
            }
        });
        ui.label(
            RichText::new(
                "Only an image you choose is written into the file; the cover already \
                           there is left alone.",
            )
            .small()
            .weak(),
        );
    }

    ui.add_space(10.0);

    let outcome = app.editor().and_then(|editor| {
        editor
            .outcome()
            .map(|outcome| (outcome.message(), outcome.is_failure()))
    });

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            actions.save = true;
        }
        if ui
            .button("Write to file")
            .on_hover_text("Rewrites the tags inside the audio file itself")
            .clicked()
        {
            actions.write_to_file = true;
        }
        if ui.button("Cancel").clicked() {
            actions.close = true;
        }
    });

    ui.add_space(4.0);
    ui.label(
        RichText::new(
            "Save changes the library; Write to file changes the file's own tags as well. The \
             two differ until you write, and the marker in the library says which tracks are \
             waiting.",
        )
        .small()
        .weak(),
    );

    if let Some((message, failure)) = outcome {
        ui.add_space(6.0);
        ui.label(RichText::new(message).small().color(if failure {
            theme::WARNING
        } else {
            theme::ACCENT
        }));
    }

    actions
}

/// Run whatever the body asked for.
fn apply(app: &mut MusicCraftApp, actions: Actions) {
    if actions.use_image {
        app.use_editor_artwork();
    }
    if actions.save {
        app.save_editor();
    }
    if actions.write_to_file {
        app.write_editor_tags();
    }
    if actions.close {
        app.close_editor();
    }
}

/// The line shown while a row holds an edit its file has not been told about.
const EDIT_NOT_WRITTEN: &str =
    "This track has been edited here; the file's own tags still hold the old values.";

/// One labelled text field across the window.
fn text_field(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String, width: f32) {
    ui.label(RichText::new(label).small().weak());
    ui.add_sized(
        [width, FIELD_HEIGHT],
        TextEdit::singleline(value).hint_text(hint),
    );
}

/// One of the two numeric fields, laid out beside each other.
fn number_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).small().weak());
        ui.add_sized(
            [NUMBER_WIDTH, FIELD_HEIGHT],
            TextEdit::singleline(value).hint_text("1"),
        )
        .on_hover_text(Field::TrackNumber.rule());
    });
}

/// The cover the library currently shows for this track.
///
/// A file with no picture gets a placeholder that says so, rather than an empty
/// hole: "there is no cover art in this file" is information, and it is what
/// makes the "Use image" field beside it worth filling in.
fn cover(app: &mut MusicCraftApp, ui: &mut egui::Ui, artwork: Option<&std::path::PathBuf>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(COVER_SIZE), egui::Sense::hover());
    let texture = artwork
        .and_then(|path| app.artwork_mut().texture(ui.ctx(), path))
        .map(|texture| texture.id());

    match texture {
        Some(texture) => {
            ui.painter().image(
                texture,
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(4), ui.visuals().faint_bg_color);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "No cover art",
                FontId::proportional(12.0),
                ui.visuals().weak_text_color(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{Library, Track, TrackFields, TrackId};

    #[test]
    fn drawing_without_an_editor_is_harmless() {
        let mut app = app();

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(app.editor().is_none());
    }

    #[test]
    fn the_form_shows_the_tracks_own_fields() {
        let mut app = imported();
        app.open_editor(TrackId(1));

        let painted = painted(&mut app);

        assert!(
            painted.iter().any(|text| text == "Edit details"),
            "the window is not titled: {painted:?}"
        );
        assert!(
            painted.iter().any(|text| text.contains("Roygbiv")),
            "the title is not in the form: {painted:?}"
        );
        assert!(
            painted.iter().any(|text| text.contains("Boards of Canada")),
            "the artist is not in the form: {painted:?}"
        );
    }

    #[test]
    fn opening_the_editor_does_not_write_anything() {
        let mut app = imported();
        app.open_editor(TrackId(1));
        let before = app.track_of(TrackId(1)).expect("the track");

        egui::__run_test_ui(|ui| window(&mut app, ui));

        let after = app.track_of(TrackId(1)).expect("the track");
        assert!(TrackFields::from_track(&before).matches(&after));
        assert!(!after.tags_dirty, "drawing is not saving");
    }

    #[test]
    fn a_track_is_named_after_its_row_not_its_title() {
        let mut app = imported();
        assert!(app.open_editor(TrackId(1)));
        assert!(
            !app.open_editor(TrackId(404)),
            "a row that is not in the library has nothing to edit"
        );
    }

    /// Every line of text the frame painted.
    ///
    /// The window is drawn more than once because a floating area has to
    /// measure itself before it paints: on the first frame of its life it lays
    /// its contents out and paints nothing.
    fn painted(app: &mut MusicCraftApp) -> Vec<String> {
        let ctx = egui::Context::default();
        let mut output = None;

        for _ in 0..3 {
            let mut frame = ctx.run_ui(screen(), |ui| window(app, ui));
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let output = output.expect("`run_ui` ran at least once");
        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }

        texts
    }

    /// Collect the text of a shape and everything nested inside it.
    fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, texts);
                }
            }
            _ => {}
        }
    }

    /// An input for a window-sized screen: an overlay paints nothing on a
    /// degenerate one.
    fn screen() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1180.0, 808.0),
            )),
            ..Default::default()
        }
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

    /// An application whose library holds one tagged track.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-edit-artwork"))
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

        let mut track = Track::from_path("/music/Boards of Canada/Roygbiv.mp3");
        track.id = Some(TrackId(1));
        track.title = "Roygbiv".to_owned();
        track.artist = Some("Boards of Canada".to_owned());
        track.album = Some("Music Has the Right to Children".to_owned());

        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        app
    }
}
