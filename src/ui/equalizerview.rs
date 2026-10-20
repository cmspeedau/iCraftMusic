//! The Equalizer window: ten band gains, a preamp and the preset buttons.
//!
//! The gear in the status bar opens it. It is a window rather than a pane
//! inside Settings so it can stay open beside the music while a band is
//! trimmed by ear — an equalizer you have to close, scroll and reopen to hear
//! the difference is one you stop adjusting by ear and start adjusting by
//! numbers, which is the wrong way round for something whose whole job is
//! what it sounds like.
//!
//! The settings are copied out of the application before anything is drawn and
//! written back once, at the end, if any control moved. That keeps the values
//! the sliders edit from being the ones on screen, and means a drag is one
//! settings change however many frames it spans.

use egui::{RichText, Slider, Window};

use crate::app::MusicCraftApp;
use crate::audio::{BAND_FREQUENCIES, MAX_GAIN_DB, MIN_GAIN_DB, PRESETS};

/// Height of each band's slider, in points.
const BAND_SLIDER_HEIGHT: f32 = 96.0;

/// Draw the Equalizer window, when it is open.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if !app.equalizer_open() {
        return;
    }

    // Opened beside the middle of a first-run screen rather than from a
    // corner: like Settings it belongs to the whole application, and after the
    // first opening the user's own drag is what places it. The window sizes to
    // its one panel, so there is nothing to scroll and nothing to resize.
    let mut open = true;
    Window::new("Equalizer")
        .collapsible(false)
        .resizable(false)
        .default_pos([430.0, 150.0])
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.set_max_width(WIDTH);
            body(app, ui);
        });

    // The ✕ closes it; the gear and the dismiss key do the same, and nothing
    // here needs saving first because every move is written straight through.
    if !open {
        app.close_equalizer();
    }
}

/// The panel's width, so the ten band sliders sit comfortably in one row.
const WIDTH: f32 = 320.0;

/// Draw the equalizer controls.
pub fn body(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mut settings = app.equalizer().clone();
    let mut changed = false;

    changed |= ui
        .checkbox(&mut settings.enabled, "Enable equalizer")
        .changed();

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for preset in PRESETS {
            if ui
                .selectable_label(false, preset.name)
                .on_hover_text("Copy this preset into the sliders")
                .clicked()
            {
                settings.bands = preset.gains;
                settings.enabled = true;
                changed = true;
            }
        }
    });

    ui.add_space(8.0);
    changed |= ui
        .add(
            Slider::new(&mut settings.preamp_db, MIN_GAIN_DB..=MAX_GAIN_DB)
                .text("Preamp (dB)")
                .fixed_decimals(0),
        )
        .changed();

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = BAND_SLIDER_HEIGHT;
        for (index, frequency) in BAND_FREQUENCIES.iter().enumerate() {
            ui.vertical(|ui| {
                let gain = &mut settings.bands[index];
                let response = ui.add(
                    Slider::new(gain, MIN_GAIN_DB..=MAX_GAIN_DB)
                        .vertical()
                        .show_value(false),
                );
                changed |= response.changed();
                ui.label(RichText::new(band_label(*frequency)).small().weak());
            });
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("Flat").clicked() {
            settings.bands = [0.0; crate::audio::BANDS];
            settings.preamp_db = 0.0;
            changed = true;
        }
        ui.label(
            RichText::new("Each band moves its own frequency; the preamp trims the whole signal.")
                .small()
                .weak(),
        );
    });

    if changed {
        app.set_equalizer(settings);
    }
}

/// A band's frequency as its slider label: `31`, `125`, `1k`, `16k`.
fn band_label(frequency: u32) -> String {
    if frequency >= 1_000 {
        format!("{}k", frequency / 1_000)
    } else {
        frequency.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;

    /// An application with a player that has no output device.
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

    /// Every line of text the window paints, given the app as it stands.
    fn lines(app: &mut MusicCraftApp) -> Vec<String> {
        let ctx = egui::Context::default();
        let mut output = None;
        for _ in 0..2 {
            let mut frame = ctx.run_ui(egui::RawInput::default(), |ui| window(app, ui));
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let mut texts = Vec::new();
        for clipped in &output.expect("`run_ui` ran").shapes {
            if let egui::Shape::Text(text) = &clipped.shape {
                texts.push(text.galley.text().to_owned());
            }
        }
        texts
    }

    /// The equalizer opens as its own window from the gear rather than only
    /// existing inside the Settings panes: with nothing asking for it, it draws
    /// nothing; once opened, it carries its own controls.
    #[test]
    fn the_equalizer_opens_as_its_own_window() {
        let mut app = app();
        assert!(
            lines(&mut app).iter().all(|text| text != "Preamp (dB)"),
            "the equalizer drew with nothing asking for its window"
        );

        app.toggle_equalizer();
        let painted = lines(&mut app);
        for want in ["Enable equalizer", "Preamp (dB)", "Flat", "16k"] {
            assert!(
                painted.iter().any(|text| text == want),
                "the equalizer window is missing {want:?}: {painted:?}"
            );
        }
    }

    /// Opening it again with `toggle_equalizer` closes it — the same control
    /// that raises it puts it away — and the equalizer setting it was asked to
    /// change is untouched by the closing.
    #[test]
    fn closing_the_window_leaves_the_equalizer_as_it_was() {
        let mut app = app();
        app.toggle_equalizer();
        app.set_equalizer({
            let mut settings = app.equalizer().clone();
            settings.enabled = true;
            settings
        });

        app.toggle_equalizer();
        assert!(lines(&mut app).iter().all(|text| text != "Preamp (dB)"));
        assert!(
            app.equalizer().enabled,
            "closing the window undid the equalizer's own setting"
        );
    }
}
