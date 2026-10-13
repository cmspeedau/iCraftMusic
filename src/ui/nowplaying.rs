//! The playback bar across the top of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.
//!
//! The bar has three parts. The transport is at the far LEFT, where the pointer
//! arrives first: previous · play · next, then the speaker and the volume
//! slider, all in white and centred on the same line as the box beside them. The middle is
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
/// The box's top row is a control row — cover, queue button and the two lines
/// of text between them — so the cover sets its height the way the queue
/// button would: 32 points of square, comfortable for a picture, without
/// flattening the row it heads.
const COVER_SIZE: f32 = 32.0;

/// Points the centre box leaves above and below its contents, in points.
const CENTRE_BOX_PAD_Y: f32 = 6.0;

/// Points of height the seek row is given around its rail, in points.
///
/// Slack, not layout: the slider asks for a taller row than the rail it draws —
/// [`egui::Spacing::interact_size`], about 15 points, so a pointer has something
/// to take hold of — and a layout centres its children in the row's height. The
/// row is placed so its MIDDLE is the rail's middle, which is where the box's
/// own constants put it; a row of the rail's own ten points would grow
/// downwards from its top and settle the rail low.
const SEEK_ROW_SLACK: f32 = 24.0;

/// Height of the seek row's rail, in points.
///
/// This is the request's "rectangle": on this box the seek bar is a squared-off
/// bar of a fixed thickness rather than egui's default rounded rail, and it is
/// thick enough to be a target for a pointer rather than a hair to aim at.
const SEEK_RAIL_HEIGHT: f32 = 10.0;

/// Height of the centre box, in points.
///
/// The cover, the seek row under it, and the points the box leaves above and
/// below the two: the box is as tall as what it holds plus its own padding, so
/// this has to be all of them together. Nothing about it is measured off a
/// child — the pill is placed by this constant, so a row that grew would hang
/// out of it rather than moving it.
const CENTRE_BOX_HEIGHT: f32 =
    COVER_SIZE + CENTRE_BOX_GAP + SEEK_RAIL_HEIGHT + 2.0 * CENTRE_BOX_PAD_Y;

/// Points between the cover, the track's details and the queue button, in
/// points.
const CENTRE_BOX_GAP: f32 = 10.0;

/// Points between the title and the state line under it, in points.
///
/// Tight, because the two are one block of text rather than two rows of a
/// layout: at the box's own spacing the pair measures taller than the cover
/// that sets the top row's height, and the seek row under it leaves the pill.
const DETAILS_LINE_GAP: f32 = 2.0;

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

/// Width of one of the seek row's two time labels, in points.
///
/// Fixed, so the rail between them is centred in the box and stays put as the
/// digits change; a time is never wider than an hour's, so nothing elides. The
/// label is laid out in a box this wide rather than sized to its text, which is
/// why the rail's length is a function of the box and not of how long `0:03`
/// happens to be.
const TIME_LABEL_WIDTH: f32 = 52.0;

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

                // The transport takes the bar's one row; the centre box is
                // placed over the room that row leaves rather than taking a row
                // of its own, and it draws the seek row inside itself.
                transport_row(app, ui);
                centre_box(app, ui);
            });
        });

    queue_popup(app, ui, panel.response.rect);
}

/// The bar's one row of controls, at its far left.
///
/// Returns the row's rectangle, which is what the centre box is centred
/// against: the box fills the whole bar under this row, so the row's own height
/// is not what places it.
///
/// The transport reads previous · play · next, then the speaker and the volume
/// slider. It is centred on the CENTRE BOX's own middle rather than on the bar's,
/// so the three buttons sit level with the box they play into — the box is the
/// thing the transport belongs to, and the bar's extra height is padding around
/// the two of them.
fn transport_row(app: &mut MusicCraftApp, ui: &mut egui::Ui) -> Rect {
    // The row is placed by its own rectangle so its MIDDLE is the centre box's,
    // which is the bar's own middle: a horizontal layout would put the row's
    // TOP where the cursor is and the controls would ride high in the bar.
    let row = Rect::from_center_size(
        ui.max_rect().center(),
        Vec2::new(ui.max_rect().width(), COVER_SIZE),
    );

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_transport")
            .max_rect(row)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            controls(app, ui);
            ui.add_space(TRANSPORT_GAP);
            volume(app, ui);
        },
    );

    row
}

/// The cream box on the window's centre: cover, track details, queue button,
/// and the seek row under them.
///
/// The pill is painted from the same constants its contents are placed by, so it
/// fits them exactly and its centre is the window's centre. Painting it here
/// rather than through a `Frame` is what makes that exact: a frame is only as
/// wide as what it holds, and what holds it — a child of the bar's own
/// top-down layout — is a quarter of the window wide.
///
/// The box fills the bar from top to bottom, so the seek row sits INSIDE the
/// pill rather than across the window under it: the two rows of the bar are now
/// the transport on the gray at its left and the seek row on the cream in the
/// middle. The box needs no inset from the bar for that — the frame's own
/// margin is the bars' inset, and a second one would only shrink the pill.
fn centre_box(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Owned before the box draws: it reads several pieces of the app and then
    // mutates it (dismissing a failure, opening the queue), and a borrow held
    // across that closure is a borrow conflict.
    let playing = app.now_playing();
    let title = playing
        .as_ref()
        .map_or_else(|| "Nothing playing".to_owned(), |track| track.title.clone());
    let artwork = playing.and_then(|track| track.artwork);
    let failure = app.playback_error();
    let state = failure.clone().unwrap_or_else(|| subtitle(app));

    let pill = centre_box_rect(ui.max_rect());

    // Under the contents, so the cover, the text and the rail sit on the cream
    // rather than the cream over them.
    ui.painter().rect_filled(
        pill,
        CornerRadius::same(theme::CENTRE_BOX_RADIUS),
        theme::CENTRE_BOX_FILL,
    );

    // The two rows are given their own rectangles rather than stacked by the
    // layout, so each one lands where the pill's own height puts it. A row that
    // grew — a title and a state line at the box's spacing measure 35 points
    // against the cover's 32 — would otherwise push the seek row out through
    // the box's foot, and the pill is placed from constants that know nothing
    // about how tall the text came out.
    let cover_row = top_row_rect(pill);
    let seek_row = seek_row_rect(pill);

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_centre_box")
            .max_rect(cover_row)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            theme::apply_centre_box_visuals(&mut ui.style_mut().visuals);
            ui.spacing_mut().item_spacing = Vec2::new(CENTRE_BOX_GAP, DETAILS_LINE_GAP);

            cover(app, ui, artwork.as_deref());
            details(ui, &title, &state, failure.is_some());

            // The button is at the row's trailing edge, not after the details:
            // the pill is sized to hold both and its own padding, so placing it
            // against the edge lands it there whatever the title came to.
            //
            // It is centred on the BOX's own middle, not on the top row's: the
            // box is the thing the button belongs to, and the row is only where
            // its own contents happen to sit. On the row's line it read as level
            // with the cover and high in the box; on the box's line it sits in
            // the box, and the two rows underneath it are what the eye reads the
            // height from.
            let button = Rect::from_center_size(
                Pos2::new(cover_row.right() - QUEUE_BUTTON_SIZE / 2.0, pill.center().y),
                Vec2::splat(QUEUE_BUTTON_SIZE),
            );
            ui.scope_builder(
                UiBuilder::new()
                    .id_salt("playback_queue_button")
                    .max_rect(button)
                    .layout(Layout::top_down(Align::Center)),
                |ui| queue_button(app, ui),
            );
        },
    );

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_seek_row")
            .max_rect(seek_row)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            theme::apply_centre_box_visuals(&mut ui.style_mut().visuals);
            progress(app, ui, seek_row);
        },
    );
}

/// The box's top row: the cover, the track's details and the queue button, one
/// row deep at the top of the pill.
///
/// Its height is the box's own padding plus the square the cover and the button
/// sit in — the same sum the pill's height is built from — so the seek row below
/// starts exactly where this row stops.
fn top_row_rect(pill: Rect) -> Rect {
    let inset = f32::from(theme::CENTRE_BOX_PAD_Y);
    let row = f32::from(theme::CENTRE_BOX_PAD);
    Rect::from_min_size(
        Pos2::new(pill.left() + row, pill.top() + inset),
        Vec2::new(pill.width() - 2.0 * row, COVER_SIZE),
    )
}

/// The box's seek row: the full inner width of the pill, centred on where the
/// rail's own middle belongs.
fn seek_row_rect(pill: Rect) -> Rect {
    let inset_x = f32::from(theme::CENTRE_BOX_PAD);
    Rect::from_center_size(
        Pos2::new(pill.center().x, rail_centre_y(pill)),
        Vec2::new(pill.width() - 2.0 * inset_x, SEEK_ROW_SLACK),
    )
}

/// The middle of where the box's rail belongs, measured down from the pill's
/// top: its own padding, the cover row it sets, the box's gap, and half the
/// rail.
fn rail_centre_y(pill: Rect) -> f32 {
    pill.top()
        + f32::from(theme::CENTRE_BOX_PAD_Y)
        + COVER_SIZE
        + CENTRE_BOX_GAP
        + SEEK_RAIL_HEIGHT / 2.0
}

/// Where the centre box sits on the bar: centred on the window, and filling the
/// bar's own inner height so its middle is the bar's middle — the line the
/// transport is centred on too.
fn centre_box_rect(bar: Rect) -> Rect {
    let width = 2.0 * f32::from(theme::CENTRE_BOX_PAD)
        + COVER_SIZE
        + CENTRE_BOX_GAP
        + CENTRE_BOX_TEXT_WIDTH
        + CENTRE_BOX_GAP
        + QUEUE_BUTTON_SIZE;

    Rect::from_center_size(bar.center(), Vec2::new(width, CENTRE_BOX_HEIGHT))
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
        // The box's own 10-point spacing is the width between its three
        // controls; between the title and the state under it a tenth of a line
        // is a gap rather than a paragraph break, and the pair has to stay
        // inside the cover's 32 points or the seek row is pushed out of the
        // pill that is sized to hold it.
        ui.spacing_mut().item_spacing.y = DETAILS_LINE_GAP;

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
    // The transport is the bar's own ink everywhere else, but these three are
    // painted white at the user's request; the helper keeps the colour in one
    // place so the glyph and the label cannot drift apart.
    fn white(glyph: &str) -> RichText {
        RichText::new(glyph).color(theme::PLAYBACK_BAR_CONTROL_INK)
    }
    let size = [38.0, 28.0];

    if ui
        .add_enabled(has_current, Button::new(white("⏮")).min_size(size.into()))
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
        .add_enabled(has_current, Button::new(white(play)).min_size(size.into()))
        .on_hover_text("Play or pause")
        .clicked()
    {
        app.toggle_playback();
    }

    if ui
        .add_enabled(has_current, Button::new(white("⏭")).min_size(size.into()))
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

/// Speaker icon and volume slider, at the tail of the transport: the buttons
/// come first and the slider follows them, so the pointer's first stop is the
/// control that starts the music.
fn volume(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.spacing_mut().slider_width = VOLUME_SLIDER_WIDTH;
    ui.label(
        RichText::new(VOLUME_GLYPH)
            .small()
            .color(theme::PLAYBACK_BAR_CONTROL_INK),
    );

    let mut value = app.playback().volume();

    // The rail and the handle are the one control the bar does not paint from a
    // widget fill: the slider asks for its rail and handle through the
    // interaction state's `bg_fill`, so the white it is drawn in has to be set
    // here for the two of them and restored afterwards, or the transport
    // buttons that follow in the same row would be left without the bar's own
    // fills.
    let resting = ui.style().visuals.widgets.inactive.bg_fill;
    let hovered = ui.style().visuals.widgets.hovered.bg_fill;
    let active = ui.style().visuals.widgets.active.bg_fill;
    {
        let widgets = &mut ui.style_mut().visuals.widgets;
        widgets.inactive.bg_fill = theme::PLAYBACK_BAR_CONTROL_INK;
        widgets.hovered.bg_fill = theme::PLAYBACK_BAR_CONTROL_INK;
        widgets.active.bg_fill = theme::PLAYBACK_BAR_CONTROL_INK;
    }

    let response = ui
        .add(Slider::new(&mut value, 0.0..=1.0).show_value(false))
        .on_hover_text("Volume");

    let widgets = &mut ui.style_mut().visuals.widgets;
    widgets.inactive.bg_fill = resting;
    widgets.hovered.bg_fill = hovered;
    widgets.active.bg_fill = active;

    if response.changed() {
        // Going through the app keeps the setting and the audible volume in
        // step: the slider is a setting as much as it is a control.
        app.set_volume(value);
    }
}

/// Elapsed time, a seek bar, and the track length, as one row.
///
/// The row's rectangle is given rather than asked for: the box places it inside
/// the pill, and taking it as an argument keeps the rail's geometry a function
/// of the box instead of of where the widgets before it happened to stop.
///
/// The bar is a solid, squared-off rectangle rather than egui's default pill —
/// [`SEEK_RAIL_HEIGHT`] thick, filled in the box's own control shade with the
/// played part in the accent — and it carries its full length whatever the
/// length of the track, so the row does not shift as tracks of different
/// lengths move through it.
fn progress(app: &mut MusicCraftApp, ui: &mut egui::Ui, row: Rect) {
    let duration = app.track_duration();
    let elapsed = format_duration(app.elapsed());
    let total = duration.map_or_else(|| "--:--".to_owned(), format_duration);

    // The three controls are placed in rectangles of their own rather than
    // stacked by the layout, so each one lands where the box's own width says it
    // does. A horizontal layout's cursor is advanced by what each child reports,
    // and a `0:03` reports its 17 points rather than the 52 its column is worth
    // -- measured on screen, that slid the rail 27 points left of the box's
    // centre. The rects make the geometry unarguable: the rail is what is left
    // between the two labels, and the labels are the same width.
    let label_box = |side: Side| {
        let x = match side {
            Side::Leading => row.left(),
            Side::Trailing => row.right() - TIME_LABEL_WIDTH,
        };
        Rect::from_min_size(
            Pos2::new(x, row.top()),
            Vec2::new(TIME_LABEL_WIDTH, row.height()),
        )
    };

    let rail_rect = Rect::from_min_max(
        Pos2::new(label_box(Side::Leading).right(), row.top()),
        Pos2::new(label_box(Side::Trailing).left(), row.bottom()),
    );
    let rail_width = rail_rect.width().max(0.0);
    ui.spacing_mut().slider_width = rail_width;
    ui.style_mut().spacing.slider_rail_height = SEEK_RAIL_HEIGHT;

    time_label(ui, label_box(Side::Leading), &elapsed);

    let mut fraction = app.progress();
    let seek = ui.scope_builder(UiBuilder::new().max_rect(rail_rect), |ui| {
        ui.add_enabled(
            // The track's length is what says there is anywhere to seek to:
            // with none reported the bar shows the elapsed part of nothing
            // and is drawn in egui's own disabled style, as the control it
            // is, and the action it returns is ignored either way.
            duration.is_some(),
            Slider::new(&mut fraction, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true)
                // Rectangular, not egui's default circular knob: the bar
                // reads as a bar, and the handle as the part of it held.
                .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.6 }),
        )
    });

    if seek.inner.changed() {
        app.seek_to_fraction(fraction);
    }

    time_label(ui, label_box(Side::Trailing), &total);
}

/// Which end of the seek row a time label sits at.
#[derive(Clone, Copy)]
enum Side {
    /// The elapsed time, at the row's head.
    Leading,
    /// The track's length, at the row's tail.
    Trailing,
}

/// One of the two times either side of the rail, in a box of its own width.
///
/// The box is exactly [`TIME_LABEL_WIDTH`] whatever the text comes to — the
/// rectangle is placed by the caller and this only puts the text in it — so the
/// rail between the two of them is a fixed length and centred in the box; the
/// text is centred in its box so the pair read as a frame around the bar rather
/// than as text hanging off each end.
fn time_label(ui: &mut egui::Ui, rect: Rect, text: &str) {
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.add(
                egui::Label::new(RichText::new(text).small().weak())
                    .halign(Align::Center)
                    .selectable(false),
            );
        },
    );
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

        /// The ink of the first line containing `needle` that is painted in
        /// `colour`. Colour as well as text, because the same glyph can be drawn
        /// twice in two inks and only one of them is the transport's.
        fn coloured(&self, needle: &str, colour: egui::Color32) -> Option<egui::Rect> {
            let text = self.texts.iter().find(|text| {
                text.galley.text().contains(needle)
                    && text
                        .galley
                        .job
                        .sections
                        .first()
                        .is_some_and(|section| section.format.color == colour)
            })?;
            let size = text.galley.size();
            let min = match text.galley.job.halign {
                egui::Align::Center => text.pos - egui::vec2(size.x / 2.0, 0.0),
                egui::Align::RIGHT => text.pos - egui::vec2(size.x, 0.0),
                egui::Align::LEFT => text.pos,
            };

            Some(egui::Rect::from_min_size(min, size))
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
    /// Found by its TRANSLUCENCY rather than its colour or its size: the rail
    /// and the played part of it over it are drawn with an alpha, where every
    /// opaque rectangle on this bar is a surface — the bar's own fill is 1180
    /// points wide and would otherwise be the widest rectangle in the frame. A
    /// slider with no known duration is drawn DISABLED, in egui's own disabled
    /// style rather than the box's control shade, so the colour is not a
    /// property these tests can lean on either.
    fn seek_rail(painted: &Painted) -> egui::Rect {
        painted
            .rects
            .iter()
            .filter(|(_, colour)| colour.a() < 255)
            .map(|(rect, _)| *rect)
            .filter(|rect| rect.width() >= 100.0)
            .max_by(|left, right| left.width().total_cmp(&right.width()))
            .expect("the seek row paints a rail")
    }

    /// The window the whole bar is painted into.
    const WINDOW_WIDTH: f32 = 1180.0;

    /// The width the box hands the seek row: the pill's INNER width, which is
    /// what `centre_box::seek_row_rect` gives it, not the pill's own width and
    /// not the window's. A fixture that handed it the outer width would measure
    /// a rail 24 points longer than the box ever draws.
    const SEEK_ROW_WIDTH: f32 =
        CENTRE_BOX_TEXT_WIDTH + COVER_SIZE + QUEUE_BUTTON_SIZE + 2.0 * CENTRE_BOX_GAP;

    /// The seek row's rectangle as the box places it, at the origin so a painted
    /// rail's coordinates read from the row's own corner.
    fn fixture_seek_row() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(SEEK_ROW_WIDTH, SEEK_ROW_SLACK))
    }

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
                |ui| super::progress(app, ui, fixture_seek_row()),
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

    /// The rail is only as long as the box's own inner width leaves it: the two
    /// time labels and the spacing around them come off the width the box gives
    /// the row, so the rail's length is a function of the box rather than of
    /// where the transport happened to stop.
    /// The rail is centred in the box, so the two time labels either side of it
    /// are the same width and the rail's middle is the pill's middle. The label
    /// boxes are allocated at their full width rather than sized to their text,
    /// and a label that did size to its text — a 17-point `0:03` — would hand
    /// the rail 35 points it does not have and slide it left.
    #[test]
    fn the_seek_row_is_centred_in_the_pill() {
        let mut app = app_playing_a_timed_track();
        let painted = paint_bar(&mut app);
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let rail = seek_rail(&painted);

        assert!(
            (rail.center().x - pill.center().x).abs() <= 1.0,
            "the seek bar is off centre in the box: {rail:?} in {pill:?}"
        );
        assert!(
            (rail.left() - pill.left() - (pill.right() - rail.right())).abs() <= 2.0,
            "the seek bar leaves more room on one side than the other: {rail:?} in {pill:?}"
        );
    }

    /// The rail is inside the pill, and inside it with room to spare: the box is
    /// sized to hold its two rows and its own padding, so a top row taller than
    /// the cover — a title and a state line at the box's 10-point spacing — is
    /// what pushes the rail out through the box's foot. Painted, not measured
    /// from constants, because the row's height is what the text comes to.
    #[test]
    fn the_seek_row_sits_inside_the_pill() {
        let mut app = app_playing_a_timed_track();
        let painted = paint_bar(&mut app);
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let rail = seek_rail(&painted);

        assert!(
            pill.contains_rect(rail),
            "the seek bar hangs out of the centre box: {rail:?} in {pill:?}"
        );
        assert!(
            pill.bottom() - rail.bottom() >= f32::from(theme::CENTRE_BOX_PAD_Y) - 1.0,
            "the seek bar is not inset from the box's foot: {rail:?} in {pill:?}"
        );
        assert!(
            rail.top() - pill.top() >= COVER_SIZE,
            "the seek bar starts inside the cover row rather than under it: \
             {rail:?} in {pill:?}"
        );
    }

    #[test]
    fn the_seek_bar_is_as_long_as_the_box_leaves_it() {
        let mut app = app_playing_a_timed_track();
        let painted = paint_seek_row(&mut app);
        let rail = seek_rail(&painted);

        let expected = SEEK_ROW_WIDTH - 2.0 * TIME_LABEL_WIDTH;

        assert!(
            (rail.width() - expected).abs() < 1.0,
            "the seek bar is {} wide, not the {expected} the box's {SEEK_ROW_WIDTH} leaves it",
            rail.width()
        );
        assert!(
            rail.width() < SEEK_ROW_WIDTH - TIME_LABEL_WIDTH,
            "the rail is not inset by its own time labels: {} against {SEEK_ROW_WIDTH}",
            rail.width()
        );
        assert!(
            rail.width() >= 160.0,
            "the box is too narrow for a usable seek bar: {} wide",
            rail.width()
        );
    }

    /// The rail is a rectangle, and a thick one: squared-off, as tall as
    /// [`SEEK_RAIL_HEIGHT`], rather than egui's default rounded hair.
    #[test]
    fn the_seek_bar_is_a_thick_rectangle() {
        let mut app = app_playing_a_timed_track();
        let painted = paint_seek_row(&mut app);
        let rail = seek_rail(&painted);

        assert!(
            (rail.height() - SEEK_RAIL_HEIGHT).abs() < 0.5,
            "the rail is {} tall, not the {SEEK_RAIL_HEIGHT} it is set to",
            rail.height()
        );
        assert!(
            rail.height() >= 8.0,
            "the rail is a hair rather than a bar: {} tall",
            rail.height()
        );

        // Squared off, not a pill: the radius the rail is drawn with is egui's
        // `corner_radius` for an inactive widget, which this interface sets to
        // 6 -- a sixth of the rail's length, and a third of its height, so the
        // ends stay square enough to read as a bar.
        let mut visuals = egui::Visuals::dark();
        theme::apply_centre_box_visuals(&mut visuals);
        assert!(
            f32::from(visuals.widgets.inactive.corner_radius.nw) <= SEEK_RAIL_HEIGHT / 2.0,
            "the rail's corners are rounder than it is tall: {:?} against {SEEK_RAIL_HEIGHT}",
            visuals.widgets.inactive.corner_radius
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
            total.right() <= SEEK_ROW_WIDTH,
            "the total time runs out of the box: {total:?} against a {SEEK_ROW_WIDTH} row"
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

        // The buttons head the row and the slider follows them: the change
        // swapped the two, so the pointer's first stop is the control that
        // starts the music rather than the one that sets how loud it is.
        let mut buttons = [
            painted.ink("⏮").expect("previous"),
            painted.ink("▶").expect("play"),
            painted.ink("⏭").expect("next"),
        ];
        buttons.sort_by(|a, b| a.left().total_cmp(&b.left()));

        assert!(
            buttons[0].left() < slider.left(),
            "the transport buttons are not ahead of the volume slider: {:?} vs {slider:?}",
            buttons[0]
        );
        assert!(
            buttons[0].left() < 30.0,
            "the transport is not at the window's left edge: {:?}",
            buttons[0]
        );
        assert!(
            speaker.left() < slider.left(),
            "the speaker does not head the volume slider: {speaker:?} vs {slider:?}"
        );
        assert!(
            speaker.left() > buttons[2].right(),
            "the volume slider is not after the transport buttons: {speaker:?} vs {:?}",
            buttons[2]
        );

        // The three buttons, left to right, each inside the bar's left quarter.
        for pair in buttons.windows(2) {
            assert!(
                pair[0].right() <= pair[1].left(),
                "the transport is out of order: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
        assert!(
            buttons[2].right() < WINDOW_WIDTH / 4.0,
            "the transport has drifted out of the bar's left end: {:?}",
            buttons[2]
        );
    }

    /// The transport is painted WHITE, at the user's request, where the rest of
    /// the bar's controls take the bar's own ink. The colour comes off the
    /// painted glyph, not off the constant, so a call site that stopped applying
    /// it fails here.
    #[test]
    fn the_transport_is_white() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let white = theme::PLAYBACK_BAR_CONTROL_INK;
        assert_eq!(
            white,
            egui::Color32::WHITE,
            "the transport's ink is no longer white"
        );

        for (glyph, what) in [("⏮", "previous"), ("▶", "play"), ("⏭", "next")] {
            let ink = painted
                .coloured(glyph, white)
                .unwrap_or_else(|| panic!("the {what} glyph is not painted white"));
            assert!(
                ink.width() > 1.0,
                "the {what} glyph is painted but carries no ink: {ink:?}"
            );
        }

        let speaker = painted
            .coloured(VOLUME_GLYPH, white)
            .expect("the speaker is not painted white");

        // And the slider's rail is white too, so the whole transport reads as one
        // colour rather than as white buttons beside a gray rail.
        let rail = painted
            .rects_with(white)
            .into_iter()
            .filter(|rect| rect.width() > 20.0)
            .max_by(|a, b| a.width().total_cmp(&b.width()));
        assert!(
            rail.is_some_and(|rail| rail.left() > speaker.left()),
            "the volume rail is not painted white after the speaker: {rail:?} vs {speaker:?}"
        );
    }

    /// The transport's middle is the centre box's middle: the two are drawn from
    /// the bar's own centre, so the buttons sit level with the box they play
    /// into rather than riding high in the bar's own padding.
    #[test]
    fn the_transport_is_centred_on_the_centre_box() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let play = painted.ink("▶").expect("the play button");

        assert!(
            (play.center().y - pill.center().y).abs() <= 1.0,
            "the transport is not centred on the box's line: {play:?} vs {pill:?}"
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

    /// The button is centred on the BOX's middle, not on the top row's: the box
    /// is the thing it belongs to, and sitting on the row's line it read as high
    /// in the box, level with a cover it is not the size of. Measured against
    /// the pill rather than against the cover, because the cover's line is
    /// exactly what it was moved off.
    #[test]
    fn the_queue_button_is_centred_in_the_box() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let button = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .find(|rect| (rect.width() - QUEUE_BUTTON_SIZE).abs() < 0.5)
            .expect("the queue button paints its own square");

        assert!(
            (button.center().y - pill.center().y).abs() <= 1.0,
            "the queue button is not centred in the box: {button:?} in {pill:?}"
        );
        assert!(
            (button.center().x
                - (pill.right() - f32::from(theme::CENTRE_BOX_PAD) - QUEUE_BUTTON_SIZE / 2.0))
                .abs()
                <= 1.0,
            "the queue button has left the box's right-hand edge: {button:?} in {pill:?}"
        );
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
