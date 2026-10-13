//! The Stage 1 user interface shell, grown by the stages after it.
//!
//! [`render`] is the single entry point: it lays out the status bar, the
//! playback bar, the sidebar and the content area, in that order, on top of the
//! root `Ui` that `eframe` hands to [`crate::app::MusicCraftApp::ui`]. Each
//! sub-module draws one region and holds its own small amount of logic.
//!
//! The window has no header strip: what one showed (the application name and
//! version, and the failure to load the stored settings) lives in the Settings
//! section's About pane, so the first row of the window is the transport.
//!
//! Two things are drawn over everything rather than inside a section: the
//! metadata editor ([`editview`]) and the confirmation a destructive library
//! action asks for ([`dialog`]). Both are floating windows, so they can be
//! opened from any section and cannot be clipped by one. The keyboard is read
//! last ([`shortcuts`]), which is what keeps a shortcut from stealing a key
//! from a widget that has just claimed it.

pub mod artwork;
pub mod dialog;
pub mod editview;
pub mod libraryview;
pub mod navigation;
pub mod nowplaying;
pub mod playlistedit;
pub mod playlistview;
pub mod queueview;
pub mod settingsview;
pub mod shortcuts;
pub mod statusbar;
pub mod theme;

use egui::{CentralPanel, RichText, ScrollArea};

use crate::app::MusicCraftApp;

use navigation::NavSection;

/// Draw the whole window.
pub fn render(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Panels are added in the order they should claim space: the two bars that
    // frame the window first — the status bar below, the transport above — then
    // the sidebar, then whatever is left over.
    statusbar::bar(app, ui);
    nowplaying::bar(app, ui);
    navigation::sidebar(app, ui);
    content(app, ui);

    // The editor and the confirmation are overlays on top of whatever section
    // is open, and the keyboard is read last so a shortcut cannot fight with a
    // widget that has just claimed the key for itself.
    editview::window(app, ui);
    playlistedit::window(app, ui);
    dialog::confirm(app, ui);
    shortcuts::handle(app, ui);
}

/// Content area: heading for the active section plus its view.
fn content(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    CentralPanel::default().show(ui, |ui| {
        ui.add_space(4.0);

        // The Library is the one section with something to search, so its field
        // rides on the heading line instead of taking a row of its own.
        ui.horizontal(|ui| {
            ui.label(RichText::new(app.section().label()).heading());

            if app.section() == NavSection::Library {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    libraryview::search_field(app, ui);
                });
            }
        });

        ui.label(RichText::new(app.section().description()).small().weak());
        ui.add_space(10.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match app.section() {
                NavSection::Library => libraryview::show(app, ui),
                NavSection::Playlists => playlistview::show(app, ui),
                NavSection::Settings => settingsview::show(app, ui),
            });
    });
}

/// A centred placeholder for sections that have no data yet.
///
/// `bullets` are shown as a short list under the explanation; pass `&[]` to
/// show the title and detail only.
pub fn empty_state(ui: &mut egui::Ui, title: &str, detail: &str, bullets: &[&str]) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(18.0).strong());
        ui.add_space(6.0);
        ui.label(RichText::new(detail).weak());

        if !bullets.is_empty() {
            ui.add_space(14.0);
            ui.label(RichText::new("What is coming").small().weak());
            for bullet in bullets {
                ui.label(RichText::new(format!("• {bullet}")).small().weak());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{ClickMode, Library, Track, TrackId};

    #[test]
    fn the_editor_and_the_question_draw_over_the_section_that_is_open() {
        let mut app = imported();
        let visible = app.browse().visible_track_ids();
        app.browse_mut()
            .selection_mut()
            .apply(&visible, visible[0], ClickMode::Replace);
        assert!(app.edit_selection());
        app.ask_to_remove_selection();

        let painted = shell(&mut app);

        assert!(
            painted.iter().any(|text| text == "Edit details"),
            "the editor is not on screen: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Remove from the library?")),
            "the question is not on screen: {painted:?}"
        );
    }

    #[test]
    fn nothing_extra_is_drawn_when_nothing_is_open() {
        let mut app = imported();

        let painted = shell(&mut app);

        assert!(
            !painted.iter().any(|text| text == "Edit details"),
            "no editor is open: {painted:?}"
        );
        assert!(painted.iter().any(|text| text == "Library"));
    }

    /// Every line of text a whole frame of the shell paints.
    ///
    /// Drawn more than once because a floating area has to measure itself
    /// before it paints anything.
    fn shell(app: &mut MusicCraftApp) -> Vec<String> {
        let ctx = egui::Context::default();
        let mut output = None;

        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1180.0, 808.0),
                    )),
                    ..Default::default()
                },
                |ui| render(app, ui),
            );
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

    /// An application whose library holds one track.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-shell-artwork"))
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

        let mut track = Track::from_path("/music/Roygbiv.mp3");
        track.id = Some(TrackId(1));
        track.title = "Roygbiv".to_owned();
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        app
    }
}
