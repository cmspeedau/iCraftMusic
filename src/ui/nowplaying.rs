//! The playback bar across the top of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.

use egui::{Align, Button, Layout, Panel, Rect, RichText, Slider};

use crate::app::{MusicCraftApp, format_duration};
use crate::ui::{queueview, theme};

/// Draw the playback bar.
pub fn bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let panel = Panel::top("musiccraft_playback_bar")
        .resizable(false)
        .exact_size(theme::PLAYBACK_BAR_HEIGHT)
        .frame(theme::playback_bar_frame())
        .show(ui, |ui| {
            ui.scope(|ui| {
                // The bar paints a light fill in every theme, so its controls
                // take the bar's ink and fills rather than the theme's.
                theme::apply_playback_bar_visuals(&mut ui.style_mut().visuals);
                rows(app, ui);
            });
        });

    queue_popup(app, ui, panel.response.rect);
}

/// The bar's two rows: the track and controls, then the seek bar.
fn rows(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        summary(app, ui);

        // Laid out right to left so the reading order stays
        // queue · shuffle · previous · play · next · repeat, volume last.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            volume(app, ui);
            ui.add_space(18.0);
            controls(app, ui);
            ui.add_space(12.0);
            queue_button(app, ui);
        });
    });

    ui.add_space(8.0);
    progress(app, ui);
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
        return "Import a folder under Settings → Library management to begin.".to_owned();
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

    shuffle_button(app, ui);
}

/// The repeat button's text, e.g. `Repeat: All`.
fn repeat_label(app: &MusicCraftApp) -> String {
    format!("Repeat: {}", app.playback().repeat().label())
}

/// The shuffle button's glyph. Struck through by [`strike_out`] while shuffle
/// is off.
const SHUFFLE_GLYPH: &str = "🔀";

/// The shuffle button: 🔀 while shuffle is on, struck through while it is off.
///
/// An icon rather than the word, because it sits in a row of glyph buttons —
/// and the slash is what makes it read as "no shuffle" rather than as an icon
/// that has merely gone grey.
fn shuffle_button(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let shuffle = app.playback().shuffle();
    let response = ui
        .add(Button::selectable(
            shuffle,
            RichText::new(SHUFFLE_GLYPH).small(),
        ))
        .on_hover_text(shuffle_hover(shuffle));

    if !shuffle {
        strike_out(ui, response.rect);
    }

    if response.clicked() {
        app.toggle_shuffle();
    }
}

/// What the shuffle button says when it is pointed at. The word lives here
/// because the button itself no longer carries it.
fn shuffle_hover(shuffle: bool) -> &'static str {
    if shuffle {
        "Shuffle: on"
    } else {
        "Shuffle: off"
    }
}

/// Cross the shuffle glyph out.
///
/// Painted over the button rather than typed as a combining slash: none of the
/// fonts egui bundles carry one, so a struck-through glyph would silently drop
/// the stroke and read as plain shuffle.
fn strike_out(ui: &egui::Ui, rect: egui::Rect) {
    let inset = egui::vec2(rect.height() * 0.30, rect.height() * 0.26);

    ui.painter().line_segment(
        [
            rect.left_bottom() + egui::vec2(inset.x, -inset.y),
            rect.right_top() + egui::vec2(-inset.x, inset.y),
        ],
        egui::Stroke::new(1.5, ui.visuals().text_color()),
    );
}

/// The burger button that opens the queue, just left of Shuffle.
fn queue_button(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let queued = app.playback().queue().len();

    if ui
        .add(Button::selectable(
            app.queue_open(),
            RichText::new("☰").small(),
        ))
        .on_hover_text(format!("Queue ({queued})"))
        .clicked()
    {
        app.toggle_queue();
    }
}

/// The queue popover, pinned under the playback bar.
///
/// It is a free-floating window rather than a panel child: the bar is already
/// drawn, so the queue can only be an overlay below it. Its ✕ writes the same
/// flag the burger button reads, so the button's highlight stays truthful
/// whichever way the popover was closed.
fn queue_popup(app: &mut MusicCraftApp, ui: &mut egui::Ui, bar: Rect) {
    if !app.queue_open() {
        return;
    }

    let mut open = true;
    egui::Window::new("Queue")
        .fixed_pos(bar.left_bottom() + egui::vec2(16.0, 12.0))
        .default_size([420.0, 300.0])
        .resizable(true)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .show(ui, |ui| queueview::show(app, ui));
        });

    if !open {
        app.close_queue();
    }
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

    #[test]
    fn drawing_the_queue_button_does_not_open_the_queue() {
        let mut app = app();

        egui::__run_test_ui(|ui| super::queue_button(&mut app, ui));

        assert!(!app.queue_open(), "drawing is not clicking");
    }

    #[test]
    fn the_queue_popover_draws_over_the_bar_while_it_is_open() {
        let mut app = app();
        app.toggle_queue();

        egui::__run_test_ui(|ui| super::bar(&mut app, ui));

        assert!(app.queue_open(), "the popover stays up until it is closed");
    }

    #[test]
    fn the_shuffle_button_carries_the_icon_and_not_the_word() {
        assert_eq!(super::SHUFFLE_GLYPH, "🔀");
        assert_eq!(super::shuffle_hover(true), "Shuffle: on");
        assert_eq!(super::shuffle_hover(false), "Shuffle: off");
    }

    #[test]
    fn drawing_the_shuffle_button_either_way_is_harmless() {
        let mut app = app();

        for _ in 0..2 {
            egui::__run_test_ui(|ui| super::shuffle_button(&mut app, ui));
            app.toggle_shuffle();
        }

        assert!(
            !app.playback().shuffle(),
            "drawing is not clicking, on or off"
        );
    }

    #[test]
    fn the_shuffle_glyph_is_struck_out_while_shuffle_is_off() {
        let mut app = app();

        let drawn = draw_shuffle(&mut app);
        assert!(drawn.shows(SHUFFLE), "the icon is drawn: {:?}", drawn.text);
        assert_eq!(drawn.lines.len(), 1, "and crossed out while shuffle is off");

        app.toggle_shuffle();
        let drawn = draw_shuffle(&mut app);
        assert!(drawn.shows(SHUFFLE), "the icon is still drawn");
        assert!(drawn.lines.is_empty(), "with no slash once shuffle is on");
    }

    /// The glyph the button is expected to draw, spelled out rather than read
    /// from `super::SHUFFLE_GLYPH` so a change to the icon has to be meant.
    const SHUFFLE: &str = "🔀";

    /// Run the button through a real egui pass and collect what it painted.
    fn draw_shuffle(app: &mut MusicCraftApp) -> Drawn {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            super::shuffle_button(app, ui)
        });

        let mut drawn = Drawn::default();
        for clipped in &output.shapes {
            drawn.collect(&clipped.shape);
        }

        // `FullOutput` insists its texture deltas are dealt with before it is
        // dropped; a test has no renderer to apply them to.
        output.textures_delta.clear();
        drawn
    }

    /// The text and line segments a frame produced.
    #[derive(Default)]
    struct Drawn {
        text: Vec<String>,
        lines: Vec<[egui::Pos2; 2]>,
    }

    impl Drawn {
        fn collect(&mut self, shape: &egui::Shape) {
            match shape {
                egui::Shape::Text(text) => self.text.push(text.galley.text().to_owned()),
                egui::Shape::LineSegment { points, .. } => self.lines.push(*points),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        self.collect(shape);
                    }
                }
                _ => {}
            }
        }

        fn shows(&self, glyph: &str) -> bool {
            self.text.iter().any(|text| text.contains(glyph))
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
}
