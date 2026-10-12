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

pub mod artwork;
pub mod libraryview;
pub mod navigation;
pub mod nowplaying;
pub mod playlistview;
pub mod queueview;
pub mod settingsview;
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
                NavSection::Playlists => playlistview::show(ui),
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
