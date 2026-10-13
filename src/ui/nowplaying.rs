//! The playback bar across the top of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.
//!
//! The bar has three parts. The transport is at the far LEFT, where the pointer
//! arrives first: the volume slider, then previous · play · next. The middle is
//! the centre box — a pale cream pill centred on the window, holding the album
//! cover at its left, the track's title and state beside it, and the queue
//! button at its right. The seek row runs under both.
//!
//! Shuffle and repeat are not here. Both moved down to the status bar, where
//! they sit with the controls that are about the session rather than about the
//! track: on this bar each took a button's width out of the transport row to
//! say a state that outlives the track.

use egui::{
    Align, Button, Color32, CornerRadius, Layout, Panel, Pos2, Rect, RichText, Sense, Slider,
    UiBuilder, Vec2,
};
use std::path::Path;

use crate::app::{MusicCraftApp, format_duration};
use crate::ui::{queueview, theme};

/// Side of the square cover drawn inside the centre box, in points.
///
/// The box leaves 6 points of itself above and below its contents, so a 40-point
/// square makes the box 52 tall: as tall as the transport row's controls and
/// their own box around them, which is what the bar has room for.
const COVER_SIZE: f32 = 40.0;

/// Points the centre box leaves above and below its contents, in points.
const CENTRE_BOX_PAD_Y: f32 = 6.0;

/// Height of the centre box, in points.
///
/// The cover plus the points the box leaves above and below it: the box is as
/// tall as what it holds plus its own padding, so this has to be the two
/// together.
const CENTRE_BOX_HEIGHT: f32 = COVER_SIZE + 2.0 * CENTRE_BOX_PAD_Y;

/// Points between the cover, the track's details and the queue button, in
/// points.
const CENTRE_BOX_GAP: f32 = 10.0;

/// Room the centre box gives the track's title and state, in points.
///
/// The box is sized by its contents, so without this a long title would carry
/// the pill out past both edges of a narrow window. Fixed, the box is the same
/// width for every track and a title too long for it is elided.
const CENTRE_BOX_TEXT_WIDTH: f32 = 200.0;

/// Points between the transport's controls.
const TRANSPORT_GAP: f32 = 8.0;

/// Length of the volume slider, in points.
const VOLUME_SLIDER_WIDTH: f32 = 90.0;

/// Side of the queue button in the centre box, in points.
///
/// Square, and as tall as the box's contents: it reads as one of the bar's
/// controls rather than as the glyph it draws.
const QUEUE_BUTTON_SIZE: f32 = 28.0;

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

/// The burger glyph drawn on the queue button.
const QUEUE_GLYPH: &str = "☰";

/// The speaker drawn at the head of the volume slider.
const VOLUME_GLYPH: &str = "🔈";

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

                // The bar is only as tall as what it holds, and every control
                // on it is placed in a rectangle of its own: a button's own
                // padding would add to that and push the bar's fill past the
                // foot of the window.
                ui.style_mut().spacing.button_padding = Vec2::ZERO;

                // The rows first: they are the bar's own layout, and the centre
                // box is placed over the room the transport row leaves rather
                // than taking a row of its own.
                let transport = rows(app, ui);
                centre_box(app, ui, transport);
            });
        });

    queue_popup(app, ui, panel.response.rect);
}

/// The bar's two rows: the transport at the left, then the seek bar.
///
/// Returns the transport row's rectangle, which is what the centre box is
/// centred against.
fn rows(app: &mut MusicCraftApp, ui: &mut egui::Ui) -> Rect {
    // The transport comes first, at the far left, reading speaker · slider ·
    // previous · play · next: the controls you reach for most are the ones the
    // pointer is already nearest.
    let transport = ui
        .horizontal(|ui| {
            volume(app, ui);
            ui.add_space(TRANSPORT_GAP);
            controls(app, ui);
        })
        .response
        .rect;

    ui.add_space(8.0);
    progress(app, ui);

    transport
}

/// The cream box on the window's centre: cover, track details, queue button.
///
/// The pill is painted from the same constants its contents are placed by, so it
/// fits them exactly and its centre is the window's centre: the box is a
/// surface the track sits on, and the bar's middle says what is playing without
/// the transport and the seek row having to move aside for it. Painting it here
/// rather than through a `Frame` is what makes that exact: a frame is only as
/// wide as what it holds, and what holds it — a child of the bar's own
/// top-down layout — is a quarter of the window wide.
fn centre_box(app: &mut MusicCraftApp, ui: &mut egui::Ui, transport: Rect) {
    // Owned before the box draws: it reads several pieces of the app and then
    // mutates it (opening the queue), and a borrow held across that closure is
    // a borrow conflict.
    let playing = app.now_playing();
    let title = playing
        .as_ref()
        .map_or_else(|| "Nothing playing".to_owned(), |track| track.title.clone());
    let artwork = playing.and_then(|track| track.artwork);
    let failure = app.playback_error();
    let state = failure.clone().unwrap_or_else(|| subtitle(app));

    let pill = centre_box_rect(ui.max_rect(), transport);

    // Under the contents, so the cover and the title sit on the cream rather
    // than the cream over them.
    ui.painter().rect_filled(
        pill,
        CornerRadius::same(theme::CENTRE_BOX_RADIUS),
        theme::CENTRE_BOX_FILL,
    );

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_centre_box")
            .max_rect(pill.shrink2(Vec2::new(theme::CENTRE_BOX_PAD as f32, CENTRE_BOX_PAD_Y)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            theme::apply_centre_box_visuals(&mut ui.style_mut().visuals);

            cover(app, ui, artwork.as_deref());
            ui.add_space(CENTRE_BOX_GAP);
            details(ui, &title, &state, failure.is_some());

            // The button follows the details, and the pill is sized to hold
            // both and its own padding, so it lands on the box's right-hand
            // edge whatever the title is.
            ui.add_space(CENTRE_BOX_GAP);
            queue_button(app, ui);
        },
    );
}

/// Where the centre box sits on the bar: centred on the window, its top level
/// with the transport's controls and as tall as the cover it holds.
fn centre_box_rect(bar: Rect, transport: Rect) -> Rect {
    let width = 2.0 * theme::CENTRE_BOX_PAD as f32
        + COVER_SIZE
        + CENTRE_BOX_GAP
        + CENTRE_BOX_TEXT_WIDTH
        + CENTRE_BOX_GAP
        + QUEUE_BUTTON_SIZE;

    Rect::from_min_size(
        Pos2::new(bar.center().x - width / 2.0, transport.top()),
        Vec2::new(width, CENTRE_BOX_HEIGHT),
    )
}

/// The track's title and, under it, either what the transport is doing or the
/// last failure.
///
/// Bounded to [`CENTRE_BOX_TEXT_WIDTH`] so the box does not grow with the
/// title: a name too long for it is elided rather than carrying the pill across
/// the bar.
fn details(ui: &mut egui::Ui, title: &str, state: &str, failure: bool) {
    ui.vertical(|ui| {
        ui.set_min_width(CENTRE_BOX_TEXT_WIDTH);
        ui.set_max_width(CENTRE_BOX_TEXT_WIDTH);

        ui.add(
            egui::Label::new(RichText::new(title).strong())
                .truncate()
                .selectable(false),
        );

        let state = RichText::new(state).small().weak();
        let state = if failure {
            // The failure takes the place of the usual subtitle: the box has a
            // fixed height, and a message the user must read outranks where the
            // track sits in the queue.
            state.color(theme::WARNING)
        } else {
            state
        };
        ui.add(egui::Label::new(state).truncate().selectable(false));
    });
}

/// The album cover at the left of the centre box.
///
/// The picture belongs to the library row for the playing track, so the slot
/// takes whatever `now_playing` resolved and draws an empty frame when there is
/// none — the title then keeps its place as tracks change, and the box says
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
            // A step of the box's own cream, so the empty slot reads as part of
            // the box rather than as the bar's gray punched through it.
            painter.rect_filled(rect, CornerRadius::same(4), theme::CENTRE_BOX_CONTROL);
        }
    }
}

/// `Track 2 of 3 · Playing`, or what the transport is when nothing is playing.
///
/// With no track on the bar there is no queue position to name, so the line is
/// the transport's own state. It used to carry a sentence pointing at Settings,
/// which read as a second title under "Nothing playing" rather than as the
/// state of the transport, and said the same thing twice: the Library's own
/// empty state is where a user is told what to do about it, and every section
/// already spells out how to bring music in.
fn subtitle(app: &MusicCraftApp) -> String {
    let Some(track) = app.now_playing() else {
        return app.transport().label().to_owned();
    };

    let mut parts = Vec::new();
    if let Some(position) = track.position {
        parts.push(position);
    }
    parts.push(app.transport().label().to_owned());

    parts.join(" · ")
}

/// Previous, play/pause and next, in that on-screen order at the bar's left.
///
/// Shuffle and repeat are not here: each took a button's width in this row —
/// `Repeat: All` took more than a button's — to say a state that belongs to
/// the session rather than to the track, and both moved to the status bar,
/// where they sit with the other controls that are about the application.
fn controls(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let has_current = app.playback().current().is_some();
    let size = [38.0, 28.0];

    if ui
        .add_enabled(has_current, Button::new("⏮").min_size(size.into()))
        .on_hover_text("Previous")
        .clicked()
    {
        app.previous_track();
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
        .add_enabled(has_current, Button::new("⏭").min_size(size.into()))
        .on_hover_text("Next")
        .clicked()
    {
        app.next_track();
    }
}

/// The burger button at the right of the centre box, which opens the queue.
fn queue_button(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let queued = app.playback().queue().len();
    let open = app.queue_open();

    if ui
        .add_sized(
            Vec2::splat(QUEUE_BUTTON_SIZE),
            Button::selectable(open, RichText::new(QUEUE_GLYPH).small()),
        )
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

/// Speaker icon and volume slider, at the head of the transport.
fn volume(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.spacing_mut().slider_width = VOLUME_SLIDER_WIDTH;
    ui.label(RichText::new(VOLUME_GLYPH).small().weak());

    let mut value = app.playback().volume();
    let response = ui
        .add(Slider::new(&mut value, 0.0..=1.0).show_value(false))
        .on_hover_text("Volume");

    if response.changed() {
        // Going through the app keeps the setting and the audible volume in
        // step: the slider is a setting as much as it is a control.
        app.set_volume(value);
    }
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
    fn an_empty_queue_says_nothing_but_the_transport_state() {
        let app = app();

        assert_eq!(app.transport(), TransportState::Unavailable);
        assert_eq!(app.now_playing(), None);
        assert_eq!(
            subtitle(&app),
            "Not connected",
            "the empty bar says more than what the transport is"
        );

        // The line the user asked to be rid of, in the exact form it was
        // painted in: it is what `subtitle` used to return here.
        assert!(
            !subtitle(&app).contains("Import a folder"),
            "the how-to-start sentence is back on the bar: {}",
            subtitle(&app)
        );
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

    /// The transport no longer has a repeat button; the mode is cycled from the
    /// status bar's switch now, and that helper is what it calls.
    #[test]
    fn cycling_the_repeat_mode_remembers_it() {
        let mut app = app();
        assert_eq!(app.playback().repeat(), RepeatMode::Off);

        assert_eq!(app.cycle_repeat_mode(), RepeatMode::All);
        assert_eq!(
            app.configured_repeat(),
            RepeatMode::All,
            "the mode moved but the setting did not follow"
        );

        assert_eq!(app.cycle_repeat_mode(), RepeatMode::One);
        assert_eq!(app.configured_repeat(), RepeatMode::One);
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
    fn the_cover_is_painted_beside_the_title_at_the_head_of_the_centre_box() {
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

        // Inside the box, not on the bar's gray: the pill is what the track
        // sits on, and a cover left outside it would read as a second control
        // on the transport.
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        assert!(
            pill.contains_rect(art),
            "the cover is not inside the centre box: {art:?} against {pill:?}"
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
            .rects_with(theme::CENTRE_BOX_CONTROL)
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

    /// The centre box is centred on the window, not parked beside the transport
    /// that now lives at the bar's left: its two edges are the same distance
    /// from the window's two edges.
    #[test]
    fn the_centre_box_is_centred_on_the_window() {
        let mut app = app();
        let painted = paint_bar(&mut app);
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        assert!(
            (pill.center().x - WINDOW_WIDTH / 2.0).abs() <= 1.0,
            "the centre box is not centred: {pill:?} on a {WINDOW_WIDTH}-wide window"
        );
        assert!(
            (pill.left() - (WINDOW_WIDTH - pill.right())).abs() <= 1.0,
            "the centre box is a different distance from each edge: {pill:?}"
        );
    }

    /// The pale cream the user asked for, read off the frame that ships.
    #[test]
    fn the_centre_box_is_cream() {
        let [r, g, b, a] = theme::CENTRE_BOX_FILL.to_array();
        assert_eq!(a, 255, "the box's surface must be opaque");
        assert!(
            r > g && g > b,
            "cream leads with red, then green, then blue"
        );
        assert!(r >= 230, "the box has to stay a pale cream: {r} red");

        let mut app = app();
        let painted = paint_bar(&mut app);
        assert!(
            !painted.rects_with(theme::CENTRE_BOX_FILL).is_empty(),
            "the box never painted the cream it is defined by"
        );
    }

    /// The transport reads from the window's left edge: the volume first, then
    /// previous · play · next, all inside the left quarter of the bar.
    #[test]
    fn the_transport_sits_at_the_far_left_of_the_bar() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let speaker = painted.ink(VOLUME_GLYPH).expect("the volume icon");
        let slider = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| (rect.width() - VOLUME_SLIDER_WIDTH).abs() < 2.0)
            .min_by(|a, b| a.left().total_cmp(&b.left()))
            .expect("the volume slider's rail");

        assert!(
            speaker.left() < slider.left(),
            "the speaker does not head the volume slider: {speaker:?} vs {slider:?}"
        );
        assert!(
            speaker.left() < 30.0,
            "the transport is not at the window's left edge: {speaker:?}"
        );

        // The three transport controls, left to right, and each inside the
        // left quarter of the bar.
        let mut controls = [
            painted.ink("⏮").expect("previous"),
            painted.ink("▶").expect("play"),
            painted.ink("⏭").expect("next"),
        ];
        controls.sort_by(|a, b| a.left().total_cmp(&b.left()));
        for pair in controls.windows(2) {
            assert!(
                pair[0].right() <= pair[1].left(),
                "the transport is out of order: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
        assert!(
            controls[2].right() < WINDOW_WIDTH / 4.0,
            "the transport has drifted out of the bar's left end: {:?}",
            controls[2]
        );
        assert!(
            controls[0].left() > slider.right(),
            "the transport is not to the right of the volume slider: {slider:?} vs {:?}",
            controls[0]
        );
    }

    /// The queue button is at the RIGHT of the box, and the box holds it: a
    /// button left outside the cream would be a control on the bar instead.
    #[test]
    fn the_queue_button_sits_at_the_right_of_the_centre_box() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let burger = painted.ink(QUEUE_GLYPH).expect("the queue button");
        let title = painted.ink("Nothing playing").expect("the title");

        assert!(
            burger.left() > title.right(),
            "the queue button is not to the right of the track details: \
             {title:?} vs {burger:?}"
        );
        assert!(
            pill.contains_rect(burger),
            "the queue button is outside the centre box: {burger:?} against {pill:?}"
        );

        // The box's right edge is where the button is, within its own padding.
        assert!(
            pill.right() - burger.right() < f32::from(theme::CENTRE_BOX_PAD) + QUEUE_BUTTON_SIZE,
            "the queue button is not at the box's right-hand edge: {burger:?} vs {pill:?}"
        );
    }

    /// The pill is as tall as the cover it holds plus its own padding, and no
    /// part of it reaches the bottom of the window — the bar is as tall as what
    /// it draws, and the box must not drag its fill past the window's foot.
    #[test]
    fn the_centre_box_fits_between_the_bar_and_its_own_contents() {
        let mut app = app();
        let painted = paint_bar(&mut app);
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        assert!(
            (pill.height() - CENTRE_BOX_HEIGHT).abs() <= 1.0,
            "the box is {} tall, not the {CENTRE_BOX_HEIGHT} it is defined as",
            pill.height()
        );
        assert!(
            pill.bottom() <= theme::PLAYBACK_BAR_HEIGHT - COVER_SIZE / 2.0,
            "the box reaches the foot of the bar: {pill:?}"
        );

        // The cover, centred in it, leaves the same cream above as below.
        let cover = painted
            .rects_with(theme::CENTRE_BOX_CONTROL)
            .into_iter()
            .find(|rect| (rect.width() - COVER_SIZE).abs() < 0.5);
        if let Some(cover) = cover {
            assert!(
                (cover.center().y - pill.center().y).abs() <= 1.0,
                "the cover is not centred in the box: {cover:?} in {pill:?}"
            );
        }
    }

    /// The queue button still opens the queue where it landed: the box moved it
    /// to the middle of the bar, and a control that no longer answers is not a
    /// control that moved.
    #[test]
    fn a_click_on_the_queue_button_opens_the_queue() {
        let mut app = app();
        assert!(!app.queue_open(), "the queue starts shut");

        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_WIDTH, 808.0));

        let mut frame = |events: Vec<egui::Event>| {
            let mut run = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ui| super::bar(&mut app, ui),
            );
            let mut painted = Painted::default();
            for clipped in &run.shapes {
                painted.collect(&clipped.shape);
            }
            run.textures_delta.clear();
            painted
        };

        // One pass so egui learns the button's rectangle: a hit is decided
        // against the previous frame.
        let painted = frame(Vec::new());
        let pos = painted
            .ink(QUEUE_GLYPH)
            .expect("the queue button was drawn")
            .center();

        frame(vec![egui::Event::PointerMoved(pos)]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }]);

        assert!(
            app.queue_open(),
            "clicking the queue button at {pos:?} did not open the queue"
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
