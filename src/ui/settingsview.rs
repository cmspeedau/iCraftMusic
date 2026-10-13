//! The Settings section.
//!
//! Every control here writes straight through to [`MusicCraftApp`], which is
//! what marks the settings dirty and schedules the write to disk at the end of
//! the frame. The shortcuts are listed rather than configured: the keys are a
//! fixed part of the interface, and the list is here so a user can find them
//! without hunting through the menus.

use egui::{Key, RichText, Slider, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::config::{DEFAULT_VOLUME, ThemeMode};
use crate::ui::shortcuts::Command;
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
    shortcuts(ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    storage(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    about(app, ui);
}

/// Width of the folder chooser button, in points.
const BROWSE_WIDTH: f32 = 112.0;
/// Width of the "add this folder" button.
const IMPORT_WIDTH: f32 = 72.0;
/// Width of the button that rescans the watched folders.
const RESCAN_WIDTH: f32 = 68.0;
/// Width of the button that stops a running scan.
const STOP_WIDTH: f32 = 54.0;

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

/// The keys the player answers to.
///
/// Two columns, one line per command: what it does on the left, the keys on the
/// right, so the list can be read down either way.
fn shortcuts(ui: &mut egui::Ui) {
    ui.label(RichText::new("Keyboard shortcuts").strong());
    ui.label(
        RichText::new("These are ignored while you are typing into a text field.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    egui::Grid::new("musiccraft_shortcuts")
        .num_columns(2)
        .spacing([24.0, 4.0])
        .show(ui, |ui| {
            for command in Command::ALL {
                ui.label(RichText::new(command.description()).small());
                ui.label(RichText::new(command.keys()).small().weak().monospace());
                ui.end_row();
            }
        });
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

/// What this build is, and whether the settings on disk could be read.
///
/// The window has no header strip, so the name, the version and any load
/// failure live here.
fn about(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("About").strong());
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::APP_NAME).heading());
        ui.label(RichText::new(version_line()).small().weak());
    });

    if let Some(message) = app.config_error().map(str::to_owned) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(format!("Settings could not be loaded: {message}")).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_config_error();
            }
        });
    }
}

/// The version line the About pane shows.
fn version_line() -> String {
    format!("v{}", crate::APP_VERSION)
}

/// The folders the library watches, and how to add or rescan them.
fn library_management(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Library management").strong());
    ui.label(
        RichText::new(
            "Folders you watch are scanned for audio files and indexed. Pick one with the \
             chooser, or type its path.",
        )
        .small()
        .weak(),
    );
    ui.add_space(6.0);

    let scanning = app.is_scanning();

    // Everything on this row except the path field has a fixed width, and the
    // gaps between them are counted as well: the field takes whatever is left,
    // so the row can never overflow the pane and push a button off its edge.
    let fixed = BROWSE_WIDTH
        + IMPORT_WIDTH
        + RESCAN_WIDTH
        + ui.spacing().item_spacing.x * 3.0
        + if scanning {
            STOP_WIDTH + ui.spacing().item_spacing.x
        } else {
            0.0
        };
    let width = (ui.available_width() - fixed).max(140.0);

    ui.horizontal(|ui| {
        let field = ui.add_sized(
            [width, 24.0],
            TextEdit::singleline(app.folder_input_mut()).hint_text("/home/you/Music"),
        );
        let submitted = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));

        // The field is still the way to type a path — a folder on a network
        // share or one behind a path only the user knows. The button beside it
        // is for the much more common case of picking one out of a list.
        if ui
            .add_sized([BROWSE_WIDTH, 24.0], egui::Button::new("Choose a folder…"))
            .on_hover_text("Pick a folder with the desktop's own file chooser")
            .clicked()
        {
            app.choose_library_folder();
        }

        let import = ui
            .add_sized([IMPORT_WIDTH, 24.0], egui::Button::new("Import"))
            .on_hover_text("Add a folder to the library and scan it");

        if import.clicked() || submitted {
            app.import_folder_input();
        }

        let can_scan = !scanning && !app.library_folders().is_empty();
        if ui
            .add_enabled(
                can_scan,
                egui::Button::new("Rescan").min_size(Vec2::new(RESCAN_WIDTH, 24.0)),
            )
            .on_hover_text("Scan the watched folders again")
            .clicked()
        {
            app.start_scan();
        }

        if scanning
            && ui
                .add_sized([STOP_WIDTH, 24.0], egui::Button::new("Stop"))
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::error::AppError;

    #[test]
    fn the_import_row_fits_the_pane_it_is_drawn_in() {
        let mut app = app(None);
        app.set_section(crate::ui::navigation::NavSection::Settings);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        let texts = painted(&mut app, screen);

        for label in ["Choose a folder…", "Import", "Rescan"] {
            let ink = texts
                .iter()
                .find(|(text, _)| text == label)
                .unwrap_or_else(|| panic!("{label} is not drawn: {texts:?}"))
                .1;

            // The margin is the button's own padding: the label is centred in
            // its box, so a label that ends within eight points of the edge
            // means the box around it does not.
            assert!(
                ink.right() <= screen.right() - 8.0,
                "{label} runs off the pane: {ink:?}"
            );
        }
    }

    /// Every line of text the pane paints, with the rectangle its ink covers.
    fn painted(app: &mut MusicCraftApp, screen: egui::Rect) -> Vec<(String, egui::Rect)> {
        let ctx = egui::Context::default();
        let mut output = None;

        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ui| crate::ui::render(app, ui),
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

    /// Collect a text shape's line and the ink it covers.
    fn collect(shape: &egui::Shape, texts: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Text(text) => {
                let size = text.galley.size();
                let min = match text.galley.job.halign {
                    egui::Align::LEFT => text.pos,
                    egui::Align::Center => egui::pos2(text.pos.x - size.x / 2.0, text.pos.y),
                    egui::Align::RIGHT => egui::pos2(text.pos.x - size.x, text.pos.y),
                };
                texts.push((
                    text.galley.text().to_owned(),
                    egui::Rect::from_min_size(min, size),
                ));
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, texts);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn the_library_management_pane_offers_a_folder_chooser() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| show(&mut app, ui));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text.contains("Choose a folder")),
            "the chooser button is not drawn: {painted:?}"
        );
        // Drawing it must not open the dialog: a native chooser blocks the
        // thread it is called from, and a test that called it would hang.
        assert!(
            app.library_folders().is_empty(),
            "drawing the button waits for a click"
        );
        assert!(app.library_error().is_none());
    }

    #[test]
    fn the_shortcuts_list_names_every_command_and_its_keys() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| show(&mut app, ui));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text == "Keyboard shortcuts"),
            "the list is not there: {painted:?}"
        );

        for command in Command::ALL {
            assert!(
                painted.iter().any(|text| text == command.keys()),
                "{} is missing from the list",
                command.keys()
            );
            assert!(
                painted.iter().any(|text| text == command.description()),
                "{command:?} has no line in the list"
            );
        }
    }

    #[test]
    fn the_settings_pane_draws_with_its_about_section() {
        let mut app = app(None);

        egui::__run_test_ui(|ui| show(&mut app, ui));

        assert!(app.config_error().is_none());
    }

    #[test]
    fn the_about_pane_names_the_build() {
        assert_eq!(version_line(), format!("v{}", crate::APP_VERSION));
        assert!(!crate::APP_NAME.is_empty());
    }

    #[test]
    fn a_settings_load_failure_is_still_reported_from_the_about_pane() {
        let mut app = app(Some(AppError::NoConfigDir));
        let expected = AppError::NoConfigDir.to_string();

        egui::__run_test_ui(|ui| show(&mut app, ui));

        // Drawing it does not dismiss it: the message outlives the frame until
        // the user acts on it.
        assert_eq!(
            app.config_error(),
            Some(expected.as_str()),
            "the failure the header strip used to show must survive the move"
        );
    }

    #[test]
    fn the_about_pane_paints_the_name_and_version() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| show(&mut app, ui));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text == crate::APP_NAME),
            "the settings pane should name the application"
        );
        assert!(
            painted.iter().any(|text| text == &version_line()),
            "the settings pane should show the version"
        );
        assert!(
            painted.iter().any(|text| text == "About"),
            "the pane should be labelled"
        );
    }

    /// An application with a player that has no output device.
    fn app(error: Option<AppError>) -> MusicCraftApp {
        MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
        )
    }
}
