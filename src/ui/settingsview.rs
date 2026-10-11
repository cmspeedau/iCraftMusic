//! The Settings section.
//!
//! Every control here writes straight through to [`MusicCraftApp`], which is
//! what marks the settings dirty and schedules the write to disk at the end of
//! the frame.

use egui::{RichText, Slider};

use crate::app::MusicCraftApp;
use crate::config::{DEFAULT_VOLUME, ThemeMode};

/// Draw the settings pane.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    appearance(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    playback(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    storage(app, ui);
}

/// Theme choice.
fn appearance(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Appearance").strong());
    ui.label(
        RichText::new("Changes apply immediately and are saved automatically.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    let current = app.theme();
    ui.horizontal(|ui| {
        for mode in ThemeMode::ALL {
            if ui.selectable_label(current == mode, mode.label()).clicked() {
                app.set_theme(mode);
            }
        }
    });
}

/// Playback preferences. Stage 1 only persists the value.
fn playback(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Playback").strong());
    ui.add_space(6.0);

    let mut volume = app.volume();
    let response = ui.add(Slider::new(&mut volume, 0.0..=1.0).text("Volume"));
    if response.changed() {
        app.set_volume(volume);
    }

    ui.add_space(2.0);
    ui.label(
        RichText::new(format!("Default volume is {:.0}%.", DEFAULT_VOLUME * 100.0))
            .small()
            .weak(),
    );
}

/// Where settings live on disk.
fn storage(app: &MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Storage").strong());
    ui.add_space(6.0);

    match app.config_path() {
        Some(path) => {
            ui.label(RichText::new("Settings file").small().weak());
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .monospace(),
            );
        }
        None => {
            ui.label(
                RichText::new("No configuration directory is available on this platform.")
                    .small()
                    .weak(),
            );
        }
    }
}
