//! The Stage 1 user interface shell.
//!
//! [`render`] is the single entry point: it lays out the header strip, the
//! sidebar, the content area and the playback bar, in that order, on top of the
//! root `Ui` that `eframe` hands to [`crate::app::MusicCraftApp::ui`]. Each
//! sub-module draws one region and holds its own small amount of logic.

pub mod libraryview;
pub mod navigation;
pub mod nowplaying;
pub mod playlistview;
pub mod settingsview;
pub mod theme;

use egui::{CentralPanel, RichText, ScrollArea};

use crate::app::MusicCraftApp;

use navigation::NavSection;

/// Draw the whole window.
pub fn render(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Panels are added in the order they should claim space: the two full-width
    // strips first, then the sidebar, then whatever is left over.
    top_bar(app, ui);
    nowplaying::bar(app, ui);
    navigation::sidebar(app, ui);
    content(app, ui);
}

/// Header strip: application name, version and any load failure.
fn top_bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let frame = theme::top_bar_frame(ui.style().as_ref());
    let error = app.config_error().map(str::to_owned);

    egui::Panel::top("musiccraft_top_bar")
        .frame(frame)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::APP_NAME).heading());
                ui.label(
                    RichText::new(format!("v{}", crate::APP_VERSION))
                        .small()
                        .weak(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(app.section().description()).small().weak());
                });
            });

            if let Some(message) = error {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("!").strong().color(theme::WARNING));
                    ui.label(
                        RichText::new(format!("Settings could not be loaded: {message}")).small(),
                    );
                    if ui.small_button("Dismiss").clicked() {
                        app.dismiss_config_error();
                    }
                });
            }
        });
}

/// Content area: heading for the active section plus its view.
fn content(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    CentralPanel::default().show(ui, |ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(app.section().label()).heading());
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
