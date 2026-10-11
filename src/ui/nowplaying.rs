//! The playback bar along the bottom of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.

use egui::{Align, Button, Layout, Panel, RichText, Slider};

use crate::app::{MusicCraftApp, format_duration};
use crate::ui::theme;

/// Draw the playback bar.
pub fn bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let style = ui.style().clone();

    Panel::bottom("musiccraft_playback_bar")
        .resizable(false)
        .exact_size(theme::PLAYBACK_BAR_HEIGHT)
        .frame(theme::playback_bar_frame(&style))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                summary(app, ui);

                // Laid out right to left so the reading order stays
                // shuffle · previous · play · next · repeat, volume last.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    volume(app, ui);
                    ui.add_space(18.0);
                    controls(app, ui);
                });
            });

            ui.add_space(8.0);
            progress(app, ui);
        });
}

/// Track title, and either where it sits in the queue or the last failure.
fn summary(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let title = app
        .now_playing()
        .map_or_else(|| "Nothing playing".to_owned(), |track| track.title);
    let error = app.playback_error();

    ui.vertical(|ui| {
        ui.label(RichText::new(title).strong());

        match error {
            // The failure takes the place of the usual subtitle: the bar has a
            // fixed height, and a message the user must read outranks the
            // queue position.
            Some(message) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(message).small().color(theme::WARNING));
                    if ui
                        .small_button("Dismiss")
                        .on_hover_text("Hide this message")
                        .clicked()
                    {
                        app.dismiss_playback_error();
                    }
                });
            }
            None => {
                ui.label(RichText::new(subtitle(app)).small().weak());
            }
        }
    });
}

/// `Track 2 of 3 · Playing`, or how to get started when the queue is empty.
fn subtitle(app: &MusicCraftApp) -> String {
    let Some(track) = app.now_playing() else {
        return "Load a file or folder from the Library to begin.".to_owned();
    };

    let mut parts = Vec::new();
    if let Some(position) = track.position {
        parts.push(position);
    }
    parts.push(app.transport().label().to_owned());

    parts.join(" · ")
}

/// Shuffle, previous, play/pause, next and repeat, in that on-screen order.
fn controls(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let has_current = app.playback().current().is_some();
    let size = [38.0, 28.0];

    if ui
        .add(Button::new(repeat_label(app)))
        .on_hover_text("Repeat")
        .clicked()
    {
        app.playback_mut().cycle_repeat();
    }

    if ui
        .add_enabled(has_current, Button::new("⏭").min_size(size.into()))
        .on_hover_text("Next")
        .clicked()
    {
        app.next_track();
    }

    let play = if app.playback().is_playing() {
        "⏸"
    } else {
        "▶"
    };
    if ui
        .add_enabled(has_current, Button::new(play).min_size(size.into()))
        .on_hover_text("Play or pause")
        .clicked()
    {
        app.toggle_playback();
    }

    if ui
        .add_enabled(has_current, Button::new("⏮").min_size(size.into()))
        .on_hover_text("Previous")
        .clicked()
    {
        app.previous_track();
    }

    let shuffle = app.playback().shuffle();
    if ui
        .add(Button::selectable(
            shuffle,
            RichText::new("Shuffle").small(),
        ))
        .on_hover_text("Shuffle the queue")
        .clicked()
    {
        app.toggle_shuffle();
    }
}

/// The repeat button's text, e.g. `Repeat: All`.
fn repeat_label(app: &MusicCraftApp) -> String {
    format!("Repeat: {}", app.playback().repeat().label())
}

/// Speaker icon and volume slider, at the right-hand end of the top row.
fn volume(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.spacing_mut().slider_width = 90.0;

    let mut value = app.playback().volume();
    let response = ui
        .add(Slider::new(&mut value, 0.0..=1.0).show_value(false))
        .on_hover_text("Volume");

    if response.changed() {
        // Going through the app keeps the setting and the audible volume in
        // step: the slider is a setting as much as it is a control.
        app.set_volume(value);
    }

    ui.label(RichText::new("🔈").small().weak());
}

/// Elapsed time, a seek bar, and the track length.
fn progress(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let duration = app.track_duration();
    let elapsed = app.elapsed();

    ui.horizontal(|ui| {
        ui.label(RichText::new(format_duration(elapsed)).small().weak());

        ui.spacing_mut().slider_width = (ui.available_width() - 80.0).max(80.0);
        let mut fraction = app.progress();
        let seek = ui.add_enabled(
            duration.is_some(),
            Slider::new(&mut fraction, 0.0..=1.0).show_value(false),
        );

        if seek.changed() {
            app.seek_to_fraction(fraction);
        }

        let total = duration.map_or_else(|| "--:--".to_owned(), format_duration);
        ui.label(RichText::new(total).small().weak());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::{Startup, TransportState};
    use crate::audio::{AudioEngine, Playback, RepeatMode};
    use crate::config::AppConfig;

    #[test]
    fn an_empty_queue_explains_how_to_start() {
        let app = app();

        assert!(subtitle(&app).contains("Library"), "{}", subtitle(&app));
        assert_eq!(app.transport(), TransportState::Unavailable);
        assert_eq!(app.now_playing(), None);
    }

    #[test]
    fn the_subtitle_names_the_queue_position_and_the_transport_state() {
        let mut app = app();
        app.open_path(std::path::Path::new("/music/song.mp3"));
        app.stop_playback();

        let subtitle = subtitle(&app);

        assert!(subtitle.contains("Track 1 of 1"), "{subtitle}");
        assert!(
            subtitle.contains(app.transport().label()),
            "{subtitle} should name the transport state"
        );
    }

    #[test]
    fn the_repeat_button_shows_the_current_mode() {
        let mut app = app();
        let initial = repeat_label(&app);

        app.set_repeat_mode(RepeatMode::One);
        let after = repeat_label(&app);

        assert_ne!(initial, after);
        assert!(after.ends_with(RepeatMode::One.label()), "{after}");
    }

    #[test]
    fn drawing_the_bar_without_a_device_is_harmless() {
        let mut app = app();

        egui::__run_test_ui(|ui| super::bar(&mut app, ui));
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
