//! The Settings window.
//!
//! The gear in the status bar opens it and `Ctrl/⌘ ,` brings it up: the
//! preferences ride in their own window rather than a page, because wanting
//! them never means wanting to stop looking at the music — a page made
//! changing one volume slider a trip out of the library and back.
//!
//! Every control here writes straight through to [`MusicCraftApp`], which is
//! what marks the settings dirty and schedules the write to disk at the end of
//! the frame. The shortcuts are listed rather than configured: the keys are a
//! fixed part of the interface, and the list is here so a user can find them
//! without hunting through the menus.
//!
//! The last pane is the one for when something is wrong: Diagnostics reports
//! what this copy is, where it keeps everything and what the audio side is
//! playing through, and offers the one check the application can run on its own
//! library. It reads rather than writes — the only action there is asking
//! SQLite to check itself, and even that changes nothing.

use egui::{RichText, ScrollArea, Slider, TextEdit, Vec2, Window};

use crate::app::MusicCraftApp;
use crate::audio::MAX_CROSSFADE_SECONDS;
use crate::config::DEFAULT_VOLUME;
use crate::ui::shortcuts::Command;
use crate::ui::theme;

/// Width the Settings window opens at, in points.
const WINDOW_WIDTH: f32 = 560.0;
/// Height it opens at when the screen is one the platform has not described.
const WINDOW_FALLBACK_HEIGHT: f32 = 620.0;

/// Draw the Settings window, when it is open.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if !app.settings_open() {
        return;
    }

    // The panes are long — every preference plus the shortcuts list — so the
    // contents scroll, and the window opens as tall as the window it lives in
    // has room to spare: a short window gets a settings window that fits it,
    // and a tall one shows the panes end to end. When the platform has not
    // said how tall the window is (Wayland never does), the window takes the
    // height that suits most screens. `default_size` only speaks on the first
    // opening; after that the user's resize wins.
    let height = {
        let room = ui.ctx().input(|input| input.content_rect());
        if room.is_finite() {
            (room.height() - 120.0).max(360.0)
        } else {
            WINDOW_FALLBACK_HEIGHT
        }
    };

    // The window opens with its top-left where the middle of a first-run
    // screen leaves room for it, and after that stays wherever the user put
    // it — `default_pos` only speaks on the first opening. An anchor would
    // nail it to the centre and fight every drag, which suits a question but
    // not a window one lives beside.
    let mut open = true;
    Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .default_pos([310.0, 90.0])
        .default_size([WINDOW_WIDTH, height])
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| show(app, ui));
        });

    // The ✕ closes the window; the gear asks for it again, and `Ctrl/⌘ ,`
    // asks in the middle of typing nowhere near it.
    if !open {
        app.close_settings();
    }
}

/// Draw the settings pane.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    library_management(app, ui);

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
    diagnostics(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    about(app, ui);
}

/// Width of the two buttons beside the media home's path, in points. Both
/// share it so the pair reads as one control group.
const HOME_BUTTON_WIDTH: f32 = 76.0;
/// Width of the button that rescans the watched folders.
const RESCAN_WIDTH: f32 = 68.0;
/// Width of the button that stops a running scan.
const STOP_WIDTH: f32 = 54.0;

/// Playback preferences: how loud, and through what.
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

    ui.add_space(8.0);
    crossfade(app, ui);

    ui.add_space(10.0);
    output_device(app, ui);
}

/// How much one track overlaps into the next.
///
/// This is a single control because the two behaviours are two ends of one
/// setting: at zero the next track is queued behind the current one and starts
/// the instant it ends, and above zero the two are mixed for that many seconds
/// instead.
fn crossfade(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mut seconds = app.crossfade_seconds();
    let response = ui.add(
        Slider::new(&mut seconds, 0.0..=MAX_CROSSFADE_SECONDS)
            .text("Crossfade (s)")
            .fixed_decimals(1),
    );
    if response.changed() {
        app.set_crossfade_seconds(seconds);
    }

    let hint = if seconds <= 0.0 {
        "Off: tracks change over gaplessly, with no gap.".to_owned()
    } else {
        format!("The last {seconds:.1} s of each track is mixed into the next.")
    };
    ui.label(RichText::new(hint).small().weak());
}

/// Which device the music comes out of, and what it came out at.
///
/// The choice is stored by identifier and honoured on the next run, so a user
/// whose desktop default is the wrong one does not have to fix it twice. The
/// list is only read when this pane is drawn and when the user asks for it
/// again, because enumerating devices asks the audio backend.
fn output_device(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Output device").small().weak());
    ui.add_space(4.0);

    // The devices are copied out of the application before the combo box is
    // built: choosing one calls back into the application, which cannot happen
    // while its own device list is borrowed.
    let devices = app.output_devices().to_vec();
    let chosen = app.chosen_output_device().map(str::to_owned);
    let mut requested: Option<Option<String>> = None;

    ui.horizontal(|ui| {
        let selected = chosen
            .as_ref()
            .and_then(|id| {
                devices
                    .iter()
                    .find(|device| &device.id == id)
                    .map(|device| device.name.clone())
            })
            .unwrap_or_else(|| match &chosen {
                // A stored device that is not in the list: say so rather than
                // silently showing the default, which would look like the
                // preference had been forgotten.
                Some(id) => format!("{id} (not available)"),
                None => DEFAULT_OUTPUT_LABEL.to_owned(),
            });

        egui::ComboBox::from_id_salt("musiccraft_output_device")
            .selected_text(RichText::new(selected).small())
            .width(DEVICE_PICKER_WIDTH)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(chosen.is_none(), DEFAULT_OUTPUT_LABEL)
                    .on_hover_text("Follow the desktop's own output setting")
                    .clicked()
                {
                    requested = Some(None);
                }

                for device in &devices {
                    let label = if device.is_default {
                        format!("{} (system default)", device.name)
                    } else {
                        device.name.clone()
                    };

                    if ui
                        .selectable_label(chosen.as_deref() == Some(&device.id), label)
                        .on_hover_text(&device.id)
                        .clicked()
                    {
                        requested = Some(Some(device.id.clone()));
                    }
                }
            });

        if ui
            .small_button("Rescan")
            .on_hover_text("Read the list of output devices again")
            .clicked()
        {
            app.rescan_output_devices();
        }
    });

    if let Some(device) = requested {
        app.set_output_device(device);
    }

    ui.add_space(4.0);
    match app.playback().output_description() {
        Some(description) => ui.label(
            RichText::new(format!("Playing through {description}."))
                .small()
                .weak(),
        ),
        None => ui.label(
            RichText::new("No output device is open; the library and the queue still work.")
                .small()
                .color(theme::WARNING),
        ),
    };

    if let Some(error) = app.settings_error().map(str::to_owned) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(error).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_settings_error();
            }
        });
    }
}

/// What the device picker says when the system default is chosen.
const DEFAULT_OUTPUT_LABEL: &str = "System default";

/// Width of the device picker, in points.
const DEVICE_PICKER_WIDTH: f32 = 260.0;

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

    // Not a shortcut table entry: Tab and the arrow keys are egui moving
    // focus, and Enter is answered by whichever row wears the ring — the
    // Library table, the album grid, the artists, a playlist, the queue —
    // rather than by a command run over the whole application. It belongs in
    // the pane so the gesture is findable, and stated as what it is.
    ui.add_space(8.0);
    ui.label(
        RichText::new(
            "Tab and the arrow keys move the focus ring; Enter acts on the row \
             it rests on, as a double-click would.",
        )
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

/// Where the library lives and how music comes in — laid out the way a music
/// application's advanced preferences lay it out: the media folder's location
/// in a bordered box of its own, the buttons that move it stacked at its side,
/// and everything else stated plainly underneath.
fn library_management(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Library management").strong());
    ui.label(
        RichText::new(
            "The media home is where the library's own music lives. Change moves the \
             library's watch to a folder of your choice; Reset returns it to the folder \
             the application was born with. Music comes in from the gear at the corner \
             of the bar — Add files… or Add folder… — or by dragging it onto the window.",
        )
        .small()
        .weak(),
    );
    ui.add_space(6.0);

    // The location box. The path inside it is not editable: a library pointed
    // at a mistyped path is worse than one pointed at the wrong button, so the
    // two buttons at its side are the only ways this box ever changes.
    egui::Frame::new()
        .stroke(ui.visuals().window_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(12, 12))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("{} Media folder location", crate::APP_NAME))
                    .small()
                    .strong(),
            );

            let fixed = HOME_BUTTON_WIDTH + ui.spacing().item_spacing.x;
            let width = (ui.available_width() - fixed).max(140.0);

            ui.horizontal(|ui| {
                let mut path = match app.media_home() {
                    Some(home) => home.display().to_string(),
                    None => "no location — Reset picks one".to_owned(),
                };
                ui.add_sized(
                    [width, 24.0],
                    TextEdit::singleline(&mut path)
                        .interactive(false)
                        .font(egui::FontId::monospace(12.0)),
                );

                ui.vertical(|ui| {
                    if ui
                        .add_sized([HOME_BUTTON_WIDTH, 24.0], egui::Button::new("Change…"))
                        .on_hover_text(
                            "Move the library to a folder of your choice. The library follows \
                             the new folder; the files you have are not moved.",
                        )
                        .clicked()
                    {
                        app.change_media_home();
                    }

                    if ui
                        .add_sized([HOME_BUTTON_WIDTH, 24.0], egui::Button::new("Reset"))
                        .on_hover_text(format!("Return the media home to ~/{}", crate::APP_NAME))
                        .clicked()
                    {
                        app.reset_media_home();
                    }
                });
            });
        });

    ui.add_space(6.0);

    let scanning = app.is_scanning();
    let can_scan = !scanning && !app.library_folders().is_empty();
    ui.horizontal(|ui| {
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
    } else if app.is_copying() {
        let progress = app.copy_progress();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "Copying… {} of {} files",
                    progress.copied, progress.total
                ))
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

    // The list starts after the home: the home is the box above, and what is
    // named here is what the user added alongside it — each with the way to
    // stop watching it.
    let extra: Vec<std::path::PathBuf> = app.library_folders().iter().skip(1).cloned().collect();
    if extra.is_empty() {
        ui.label(
            RichText::new("No folders are watched besides the media home.")
                .small()
                .weak(),
        );
    } else {
        ui.label(RichText::new("Other watched folders").small().weak());

        // Copied out before the buttons are drawn: forgetting a folder edits
        // the list the loop is walking.
        for folder in extra {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(folder.display().to_string())
                        .small()
                        .monospace(),
                );

                if ui
                    .small_button("Forget")
                    .on_hover_text(
                        "Stop scanning this folder. The tracks it contributed stay in the \
                         library and their files stay on disk.",
                    )
                    .clicked()
                {
                    app.remove_library_folder(&folder);
                }
            });
        }
    }
}

/// Where the application keeps everything, and whether the database is sound.
///
/// This is the "something is wrong and I do not know what" pane: what the user
/// reads out when asking for help, and the one check the application can run on
/// its own library.
fn diagnostics(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Diagnostics").strong());
    ui.label(
        RichText::new(
            "What this copy of iCraftMusic is, and where it keeps your library, your settings \
             and its log.",
        )
        .small()
        .weak(),
    );
    ui.add_space(6.0);

    egui::Grid::new("musiccraft_diagnostics")
        .num_columns(2)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            for (label, value) in app.diagnostics().lines() {
                ui.label(RichText::new(label).small().weak());
                ui.label(RichText::new(value).small().monospace());
                ui.end_row();
            }
        });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui
            .button("Check the library database")
            .on_hover_text(
                "Reads every page of the database and reports anything wrong with it. \
                 Nothing is changed.",
            )
            .clicked()
        {
            app.check_library_integrity();
        }

        if let Some(folder) = app.diagnostics().data_directory.as_ref() {
            ui.label(
                RichText::new(format!("Data: {}", folder.display()))
                    .small()
                    .weak()
                    .monospace(),
            );
        }
    });

    if let Some(message) = app.library_integrity().map(str::to_owned) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(message).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_integrity();
            }
        });
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
    fn the_location_box_and_scan_buttons_fit_the_pane_they_are_drawn_in() {
        let mut app = app(None);
        app.open_settings();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        let texts = painted(&mut app, screen);

        for label in ["Change…", "Reset", "Rescan"] {
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
            painted.iter().any(|text| text.contains("Change…")),
            "the location box does not offer its chooser: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Media folder location")),
            "the location box does not name itself: {painted:?}"
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

    #[test]
    fn the_diagnostics_pane_reports_where_everything_lives() {
        let mut app = app(None);
        let painted = everything(&mut app);

        for label in [
            "Diagnostics",
            "Application",
            "Check the library database",
            "Audio output",
            "Indexed",
        ] {
            assert!(
                painted.iter().any(|text| text == label),
                "{label} is not on the pane: {painted:?}"
            );
        }

        assert!(
            painted
                .iter()
                .any(|text| text == &format!("{} {}", crate::APP_NAME, crate::APP_VERSION)),
            "the snapshot should name the build: {painted:?}"
        );
    }

    #[test]
    fn drawing_the_diagnostics_pane_does_not_run_the_check() {
        // The check reads the whole database, so it answers a press, never a
        // repaint — and a pane that ran it every frame would make Settings the
        // slowest screen in the application.
        let mut app = app(None);

        let _ = everything(&mut app);

        assert!(
            app.library_integrity().is_none(),
            "no check was asked for: {:?}",
            app.library_integrity()
        );
    }

    #[test]
    fn a_check_the_user_asked_for_is_reported_on_the_pane() {
        let mut app = app(None);
        app.check_library_integrity();

        let painted = everything(&mut app);

        let message = app.library_integrity().expect("the check said something");
        assert!(
            painted.iter().any(|text| text.contains(message)),
            "the answer is not on the pane: {painted:?}"
        );
    }

    #[test]
    fn a_watched_folder_can_be_forgotten_from_the_pane() {
        // Forgetting a folder is about the watched list, so the application
        // needs a library for the folder to be watched by. The first folder is
        // the media home — it lives in its own box — so the forgettable one is
        // the folder added alongside it.
        let mut app = watched();
        let home = std::env::temp_dir().join("musiccraft-settings-home");
        let folder = std::env::temp_dir().join("musiccraft-settings-forget");
        std::fs::create_dir_all(&home).expect("a folder to be the home");
        std::fs::create_dir_all(&folder).expect("a folder to watch");
        assert!(app.add_library_folder(&home));
        assert!(app.add_library_folder(&folder));

        let painted = everything(&mut app);
        assert!(
            painted.iter().any(|text| text == "Forget"),
            "the folder has no way to be dropped: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text == &folder.display().to_string()),
            "the extra folder is not listed: {painted:?}"
        );

        // Drawing the button is not pressing it.
        assert_eq!(
            app.library_folders().len(),
            2,
            "the folder is forgotten on a click, not on a repaint"
        );
    }

    /// Every line of text the whole Settings pane paints.
    ///
    /// The pane is drawn through the shell so the section is set the way the
    /// window sets it, and the existing `painted` helper collects the text. The
    /// window is drawn tall on purpose: the pane is longer than any real window
    /// and the part below the fold never reaches the painter, so a short screen
    /// would only test the sections that happen to come first.
    fn everything(app: &mut MusicCraftApp) -> Vec<String> {
        app.open_settings();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 2400.0));

        painted(app, screen)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
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

    /// An application that has a library, so folders can be watched by one.
    ///
    /// The pane's own `app()` deliberately has none; anything that adds or
    /// forgets a watched folder needs the library version, or it would be
    /// asserting against a failure ("there is no library to import into")
    /// rather than against the folder list.
    fn watched() -> MusicCraftApp {
        let library = crate::library::Library::in_memory(
            std::env::temp_dir().join("musiccraft-settings-artwork"),
        )
        .expect("an in-memory library");

        MusicCraftApp::with_library(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
            library,
        )
    }
}
