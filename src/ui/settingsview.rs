//! The Settings section.
//!
//! Every control here writes straight through to [`MusicCraftApp`], which is
//! what marks the settings dirty and schedules the write to disk at the end of
//! the frame.

use egui::{Key, RichText, Slider, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::config::{DEFAULT_VOLUME, ThemeMode};
use crate::ui::theme;

/// Draw the settings pane.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    library_management(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
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

/// The folders the library watches, and how to add or rescan them.
fn library_management(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Library management").strong());
    ui.label(
        RichText::new("Folders you watch are scanned for audio files and indexed.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    let scanning = app.is_scanning();
    let rescan_width = 68.0;
    let width = (ui.available_width() - rescan_width - 88.0).max(140.0);

    ui.horizontal(|ui| {
        let field = ui.add_sized(
            [width, 24.0],
            TextEdit::singleline(app.folder_input_mut()).hint_text("/home/you/Music"),
        );
        let submitted = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));

        let import = ui
            .add_sized([80.0, 24.0], egui::Button::new("Import"))
            .on_hover_text("Add a folder to the library and scan it");

        if import.clicked() || submitted {
            app.import_folder_input();
        }

        let can_scan = !scanning && !app.library_folders().is_empty();
        if ui
            .add_enabled(
                can_scan,
                egui::Button::new("Rescan").min_size(Vec2::new(rescan_width, 24.0)),
            )
            .on_hover_text("Scan the watched folders again")
            .clicked()
        {
            app.start_scan();
        }

        if scanning
            && ui
                .add_sized([54.0, 24.0], egui::Button::new("Stop"))
                .on_hover_text("Stop the scan after the file it is reading")
                .clicked()
        {
            app.cancel_scan();
        }
    });

    ui.add_space(6.0);

    if let Some(message) = app.library_error().map(str::to_owned) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(message).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_library_error();
            }
        });
    }

    if app.is_scanning() {
        let progress = app.scan_progress();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("Scanning… {} files read", progress.processed))
                    .small()
                    .color(theme::ACCENT),
            );

            if let Some(current) = progress.current.as_ref() {
                ui.label(
                    RichText::new(current.display().to_string())
                        .small()
                        .weak()
                        .monospace(),
                );
            }
        });
    } else if let Some(message) = app.last_import().map(str::to_owned) {
        ui.label(RichText::new(message).small().weak());
    }

    ui.add_space(4.0);

    if app.library_folders().is_empty() {
        ui.label(RichText::new("No folders are watched yet.").small().weak());
    } else {
        ui.label(RichText::new("Watched folders").small().weak());
        for folder in app.library_folders() {
            ui.label(
                RichText::new(folder.display().to_string())
                    .small()
                    .monospace(),
            );
        }
    }
}
