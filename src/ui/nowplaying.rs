//! The playback bar across the top of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.

use egui::{
    Align, Button, Color32, CornerRadius, Layout, Panel, Pos2, Rect, RichText, Sense, Slider,
    Stroke, Vec2,
};
use std::path::Path;

use crate::app::{MusicCraftApp, format_duration};
use crate::ui::{queueview, theme};

/// Side of the square cover drawn beside the track details, in points.
///
/// The bar is 92 points tall and its frame leaves 10 on each side, so the two
/// rows of text plus the seek bar come to about 65: a 40-point square marks the
/// head of the bar without pushing anything past the bottom of it.
const COVER_SIZE: f32 = 40.0;

/// Fraction of the seek row's length the slider gives up, in points per point.
///
/// Elapsed time, slider and total time share one row across the window, so the
/// slider's length is whatever they leave it — and at the full remainder it ran
/// span for span with the transport above it, reading as a second bar rather
/// than as a control. The slider now takes HALF of what that remainder would
/// give it: a share rather than a fixed number of points, so the user's "half
/// the size" holds at any window width instead of only at the one it was
/// measured on. The room the slider gives up stays where it was, at the row's
/// right-hand end: the total time keeps the window's edge and the gap opens
/// between it and the slider.
const SEEK_BAR_SHARE: f32 = 0.5;

/// Shortest the seek bar is allowed to get, in points.
///
/// A narrow window would otherwise squeeze it to nothing: the elapsed and total
/// times take a fixed width each, and the share above is taken from the whole
/// row. The slider keeps this much and the labels are left to share the rest.
const SEEK_BAR_MIN_WIDTH: f32 = 160.0;

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

/// Cover art, track title, and either where it sits in the queue or the last
/// failure.
fn summary(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let playing = app.now_playing();
    let title = playing
        .as_ref()
        .map_or_else(|| "Nothing playing".to_owned(), |track| track.title.clone());
    let artwork = playing.and_then(|track| track.artwork);
    let error = app.playback_error();

    ui.horizontal(|ui| {
        cover(app, ui, artwork.as_deref());

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
    });
}

/// The album cover beside the track details.
///
/// The picture belongs to the library row for the playing track, so the slot
/// takes whatever `now_playing` resolved and draws an empty frame when there is
/// none — the title then keeps its place as tracks change, and the bar says
/// "no cover" rather than jumping sideways.
fn cover(app: &mut MusicCraftApp, ui: &mut egui::Ui, artwork: Option<&Path>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(COVER_SIZE), Sense::hover());
    let texture = artwork.and_then(|path| app.artwork_mut().texture(ui.ctx(), path));
    let painter = ui.painter();

    match texture {
        Some(texture) => {
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            // The bar's own control gray, so the empty slot reads as part of the
            // bar rather than as a hole punched in it.
            painter.rect_filled(rect, CornerRadius::same(4), theme::PLAYBACK_BAR_CONTROL);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0, theme::PLAYBACK_BAR_EDGE),
                egui::StrokeKind::Inside,
            );
        }
    }
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
    let total = duration.map_or_else(|| "--:--".to_owned(), format_duration);

    ui.horizontal(|ui| {
        ui.label(RichText::new(format_duration(elapsed)).small().weak());

        // The slider takes half of what the two times leave the row. The 80
        // covers the labels' own widths and the spacing around them.
        let remaining = (ui.available_width() - 80.0).max(0.0);
        ui.spacing_mut().slider_width =
            (remaining * (1.0 - SEEK_BAR_SHARE)).max(SEEK_BAR_MIN_WIDTH);
        let mut fraction = app.progress();
        let seek = ui.add_enabled(
            duration.is_some(),
            Slider::new(&mut fraction, 0.0..=1.0).show_value(false),
        );

        if seek.changed() {
            app.seek_to_fraction(fraction);
        }

        // The track's length follows the seek bar's end rather than sitting at
        // the window's edge: the row reads elapsed · slider · length as one
        // line, and the length the slider gives up shows as the room to the
        // right of the pair.
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

    /// Title the fixture track carries, spelled out so the assertion reads.
    ///
    /// It is also the file's name: the queue labels an entry from its path, so
    /// the name it is written under is what the bar shows.
    const TITLE: &str = "Roygbiv";

    #[test]
    fn the_cover_is_painted_beside_the_title_at_the_head_of_the_bar() {
        let mut app = app_with_a_cover();
        let painted = paint_bar(&mut app);

        let art = painted
            .images
            .first()
            .copied()
            .expect("the playing track's cover is painted");
        assert!(
            (art.width() - COVER_SIZE).abs() < 0.5 && (art.height() - COVER_SIZE).abs() < 0.5,
            "the cover is a {COVER_SIZE}-point square, got {art:?}"
        );
        assert!(
            art.left() < art.right() && art.top() < art.bottom(),
            "the cover is a real rectangle, got {art:?}"
        );

        let title = painted
            .ink(TITLE)
            .expect("the playing track's title is painted");
        assert!(
            art.right() <= title.left(),
            "the cover sits to the left of the title: {art:?} against {title:?}"
        );
        assert!(
            art.left() < title.left() && art.top() >= 0.0,
            "the cover heads the row rather than following the title: {art:?} against {title:?}"
        );
        assert!(
            art.bottom() <= theme::PLAYBACK_BAR_HEIGHT,
            "the cover stays inside the bar: {art:?}"
        );
    }

    #[test]
    fn a_track_with_no_cover_still_marks_out_the_slot() {
        let mut app = app();
        app.open_path(std::path::Path::new("/music/song.mp3"));
        let painted = paint_bar(&mut app);

        assert!(
            painted.images.is_empty(),
            "a track the library has no cover for paints no picture"
        );

        let title = painted.ink("song").expect("the title is painted");
        let slot = painted
            .rects_with(theme::PLAYBACK_BAR_CONTROL)
            .into_iter()
            .find(|rect| {
                (rect.width() - COVER_SIZE).abs() < 0.5 && (rect.height() - COVER_SIZE).abs() < 0.5
            })
            .expect("the empty cover slot is marked out");
        assert!(
            slot.right() <= title.left() && slot.bottom() <= theme::PLAYBACK_BAR_HEIGHT,
            "the slot is where the cover would be: {slot:?} against {title:?}"
        );
    }

    /// A device-less application whose library knows one track, with a cover,
    /// and which is playing that track.
    fn app_with_a_cover() -> MusicCraftApp {
        use crate::library::{Library, Track, TrackId};

        let audio = crate::fixtures::scratch_file("Roygbiv.wav", &crate::fixtures::silent_wav(64));
        let cover = crate::fixtures::scratch_file("bar.png", &crate::fixtures::png());
        let library = Library::in_memory(crate::fixtures::scratch("bar-artwork"))
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

        let mut track = Track::from_path(&audio);
        track.id = Some(TrackId(1));
        track.title = TITLE.to_owned();
        track.artwork_path = Some(cover);
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        // The queue entry carries the path the library indexed, which is what
        // the bar looks the cover up by.
        app.open_path(&std::fs::canonicalize(&audio).unwrap_or(audio));

        app
    }

    /// Run the bar through real frames at a known window size and collect what
    /// it painted.
    ///
    /// Three frames, because the panel measures itself first and only paints on
    /// the frames after that.
    fn paint_bar(app: &mut MusicCraftApp) -> Painted {
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
                |ui| super::bar(app, ui),
            );
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let output = output.expect("`run_ui` ran at least once");
        let mut painted = Painted::default();
        for clipped in &output.shapes {
            painted.collect(&clipped.shape);
        }

        painted
    }

    /// What one frame of the bar drew: the rectangles, the images and the text.
    #[derive(Default)]
    struct Painted {
        rects: Vec<(egui::Rect, egui::Color32)>,
        images: Vec<egui::Rect>,
        texts: Vec<egui::epaint::TextShape>,
    }

    impl Painted {
        fn collect(&mut self, shape: &egui::Shape) {
            match shape {
                egui::Shape::Rect(rect) => self.rects.push((rect.rect, rect.fill)),
                egui::Shape::Mesh(mesh) => self.images.push(mesh.calc_bounds()),
                egui::Shape::Text(text) => self.texts.push(text.clone()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        self.collect(shape);
                    }
                }
                _ => {}
            }
        }

        /// Every filled rectangle painted in `fill`.
        fn rects_with(&self, fill: egui::Color32) -> Vec<egui::Rect> {
            self.rects
                .iter()
                .filter(|(_, colour)| *colour == fill)
                .map(|(rect, _)| *rect)
                .collect()
        }

        /// The ink of the first line containing `needle`.
        ///
        /// Read from the galley's own alignment: a centred line is painted from
        /// its centre, so its position is not its left edge.
        fn ink(&self, needle: &str) -> Option<egui::Rect> {
            let text = self
                .texts
                .iter()
                .find(|text| text.galley.text().contains(needle))?;
            let size = text.galley.size();
            let min = match text.galley.job.halign {
                egui::Align::Center => text.pos - egui::vec2(size.x / 2.0, 0.0),
                egui::Align::RIGHT => text.pos - egui::vec2(size.x, 0.0),
                egui::Align::LEFT => text.pos,
            };

            Some(egui::Rect::from_min_size(min, size))
        }
    }

    /// A track long enough for the seek row to have a real length, playing.
    ///
    /// The slider is drawn disabled with no duration and a disabled slider's
    /// rail is not the shade these assertions look for, so the fixture's track
    /// has to carry a length the way the decoder would report one.
    fn app_playing_a_timed_track() -> MusicCraftApp {
        use crate::library::{Library, Track, TrackId};

        let audio = crate::fixtures::scratch_file("Seek.wav", &crate::fixtures::silent_wav(64));
        let library =
            Library::in_memory(crate::fixtures::scratch("seek-bar")).expect("an in-memory library");

        let mut app = MusicCraftApp::with_library(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
            library,
        );

        let mut track = Track::from_path(&audio);
        track.id = Some(TrackId(1));
        track.title = TITLE.to_owned();
        track.duration = Some(std::time::Duration::from_secs(240));
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        app.open_path(&std::fs::canonicalize(&audio).unwrap_or(audio));

        app
    }

    /// The seek bar's rail, from one frame of the seek row.
    ///
    /// The row's widest rectangle is the rail; the only other rectangle beside
    /// it is the slider's handle, a fraction of that width. Found by size
    /// rather than by colour because a slider with no known duration is drawn
    /// DISABLED, in egui's own disabled style rather than the bar's control
    /// gray — the device-less fixture has no duration to offer, and the row's
    /// geometry is what these tests are about.
    fn seek_rail(painted: &Painted) -> egui::Rect {
        painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| rect.width() >= SEEK_BAR_MIN_WIDTH)
            .max_by(|left, right| left.width().total_cmp(&right.width()))
            .expect("the seek row paints a rail")
    }

    /// The window the seek row is painted into, matching `paint_seek_row`.
    const WINDOW_WIDTH: f32 = 1180.0;

    /// Run the seek row alone through real frames at a known window size.
    ///
    /// Alone rather than under the bar: one slider in the frame makes the rail
    /// unambiguous, where the bar also paints the volume slider's. The bar's
    /// own 16-point frame margin only makes the rail shorter still.
    fn paint_seek_row(app: &mut MusicCraftApp) -> Painted {
        let ctx = egui::Context::default();
        let mut output = None;
        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(WINDOW_WIDTH, 808.0),
                    )),
                    ..Default::default()
                },
                |ui| super::progress(app, ui),
            );
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let output = output.expect("`run_ui` ran at least once");
        let mut painted = Painted::default();
        for clipped in &output.shapes {
            painted.collect(&clipped.shape);
        }

        painted
    }

    #[test]
    fn the_seek_bar_takes_no_more_than_its_half_of_the_row() {
        let mut app = app_playing_a_timed_track();
        let rail = seek_rail(&paint_seek_row(&mut app));

        // The room the row leaves the slider, measured from the slider's own
        // left edge: "half the size" is a share of what there is to take, not a
        // number of points that only holds at one window width.
        let row = WINDOW_WIDTH - rail.left();

        assert!(
            rail.width() <= row * (SEEK_BAR_SHARE + 0.01),
            "the seek bar is {} wide, more than the {} half of the row it may take",
            rail.width(),
            row * SEEK_BAR_SHARE
        );
        assert!(
            rail.width() >= SEEK_BAR_MIN_WIDTH,
            "the seek bar is still usable at {} wide",
            rail.width()
        );
        assert!(
            (rail.width() - row * SEEK_BAR_SHARE).abs() < row * 0.05,
            "the seek bar is {} wide, not the {} half of the row the size it had leaves",
            rail.width(),
            row * SEEK_BAR_SHARE
        );
    }

    #[test]
    fn the_seek_bar_stops_well_short_of_the_row_it_sits_in() {
        let mut app = app_playing_a_timed_track();
        let rail = seek_rail(&paint_seek_row(&mut app));

        assert!(
            rail.width() < WINDOW_WIDTH - 100.0,
            "the seek bar is {} wide, not the {WINDOW_WIDTH} of the row it sits in",
            rail.width()
        );
        assert!(
            rail.right() <= WINDOW_WIDTH / 2.0,
            "the seek bar ends at {}, inside the right half of the row",
            rail.right()
        );
    }

    #[test]
    fn the_total_time_sits_at_the_end_of_the_seek_bar() {
        let mut app = app_playing_a_timed_track();
        let painted = paint_seek_row(&mut app);
        let rail = seek_rail(&painted);

        let total = painted
            .ink("--:--")
            .expect("the total time is painted even with no track length");

        assert!(
            total.left() >= rail.right(),
            "the total time follows the seek bar rather than sitting away from it: \
             {total:?} against {rail:?}"
        );
        assert!(
            total.left() - rail.right() < 40.0,
            "the total time is the bar's own trailing label, not a label parked \
             across the row: {total:?} against {rail:?}"
        );
        assert!(
            total.right() < WINDOW_WIDTH - SEEK_BAR_MIN_WIDTH,
            "the total time travels with the bar instead of holding the window's \
             edge: {total:?}"
        );
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
