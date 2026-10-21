//! Visual constants and style setup for the iCraftMusic shell.
//!
//! Stage 1 only needs a small, mostly flat look: a sidebar, a content area and
//! a playback bar across the top. Everything here is shared by the views so the
//! shell stays visually consistent and the spacing is defined in exactly one
//! place.

use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Frame, Margin, Painter, Pos2, Rect,
    Response, Stroke, StrokeKind, Style, TextStyle, ThemePreference, Ui, Vec2, Visuals, vec2,
};

use crate::config::ThemeMode;

/// Width of the left-hand navigation sidebar, in points — divider included.
///
/// Measured off the reference: its source list is 204 pixels across, the
/// last of them the dark rule, and the page takes the pixel after that.
pub const SIDEBAR_WIDTH: f32 = 204.0;

/// Pixel-measured reference window geometry and chrome metrics.
pub const MENU_BAND_HEIGHT: f32 = 16.0;
pub const TOOLBAR_HEIGHT: f32 = 60.0;
pub const TRACK_ROW_HEIGHT: f32 = 17.0;
pub const CELL_PADDING: f32 = 6.0;
pub const WINDOW_WIDTH: f32 = 1180.0;
pub const WINDOW_HEIGHT: f32 = 780.0;
pub const DARK_RULE: Color32 = Color32::from_rgb(64, 64, 64);
pub const TABLE_HEADER_TOP: Color32 = Color32::from_rgb(219, 219, 219);
pub const TABLE_HEADER_BOTTOM: Color32 = Color32::from_rgb(187, 187, 187);
pub const TABLE_HEADER_RULE: Color32 = Color32::from_rgb(85, 85, 85);
pub const TABLE_STRIPE: Color32 = Color32::from_rgb(241, 245, 250);
pub const FLAT_SELECTION: Color32 = Color32::from_rgb(93, 148, 214);
pub const CENTRE_DISC_TOP: Color32 = Color32::from_rgb(254, 255, 232);
pub const CENTRE_DISC_BOTTOM: Color32 = Color32::from_rgb(220, 224, 197);

/// Height of the playback bar across the top of the window, in points.
pub const PLAYBACK_BAR_HEIGHT: f32 = TOOLBAR_HEIGHT;

/// The top end of the menu band's gradient, in points.
///
/// The band across the very top of the window carries the five menu labels;
/// the two colours here are the measured pixel rows of the reference's band,
/// light at its top and deepening down its sixteen points.
pub const MENU_BAND_TOP: Color32 = Color32::from_rgb(226, 226, 226);

/// The bottom end of the menu band's gradient.
pub const MENU_BAND_BOTTOM: Color32 = Color32::from_rgb(190, 190, 190);

/// The ink the five menu labels are set in.
pub const MENU_BAND_INK: Color32 = Color32::from_rgb(43, 43, 43);

/// Paint the menu band's measured gradient across its strip.
///
/// Square corners, edge to edge: the band is the window's own first rows, not
/// a control sitting on them. Call it inside the band's panel before the
/// labels are laid out, so the labels stand on the gradient rather than under
/// it.
pub fn menu_band_surface(painter: &Painter, rect: Rect) {
    painter.add(egui::epaint::Shape::gradient_rect(
        rect,
        egui::epaint::Direction::TopDown,
        [MENU_BAND_TOP, MENU_BAND_BOTTOM],
    ));
}

/// Height of a single sidebar entry, in points.
///
/// Measured off the reference: its rows are 20 pixels tall, and every row of
/// the source list — the Library's and the playlists' alike — wears the same
/// height, so the bands line up across the whole panel.
pub const NAV_ITEM_HEIGHT: f32 = 20.0;

/// Where a row's painted icon begins, from the sidebar's left edge.
///
/// Measured off the reference: its icons begin 28 pixels in, leaving room at
/// the left for the disclosure triangle's column, and its labels begin 16
/// pixels further on — exactly one icon wide.
pub const SIDEBAR_ICON_INSET: f32 = 28.0;

/// Where a row's label begins, from the sidebar's left edge.
///
/// Past the icon column — the source list hangs every label off the right
/// edge of its icons, so the labels start on one line however wide each icon
/// happens to be.
pub const SIDEBAR_LABEL_INSET: f32 = 44.0;

/// Height of one playlist in the sidebar's list, in points.
///
/// The reference gives a playlist row the same 20 pixels it gives a Library
/// row — the list reads as one grid, not two.
pub const PLAYLIST_ITEM_HEIGHT: f32 = 20.0;

/// Size of body-level small text across the interface, in points.
///
/// egui's default for its "small" style is 9.0, which is too small to read
/// comfortably on a desktop: this raises every `.small()` label in one place,
/// in both themes, rather than at each of the call sites. 12.0 is also what the
/// hand-painted rows already use, so the drawn text and the laid-out text agree.
pub const SMALL_TEXT_SIZE: f32 = 12.0;

/// How many playlists the sidebar lists before it starts scrolling, in points.
///
/// The list is inside the sidebar, which does not scroll: without a cap it
/// would simply run out of the window at the bottom and take the rest of the
/// sidebar with it. The room it takes has to leave the browsing entries above
/// it alone and still fit the shortest window worth supporting, which is still
/// deep enough for a long list the user scrolls in place.
pub const PLAYLIST_LIST_MAX_HEIGHT: f32 = 260.0;

/// Height of the status bar across the bottom of the window, in points.
///
/// One line of text with the room to centre it vertically, plus the squares
/// along it: the `+` that names a playlist and the shuffle and repeat switches
/// at the left, and the gear that opens Settings at the right.
pub const STATUS_BAR_HEIGHT: f32 = 28.0;

/// Width of a corner button on the status bar, in points.
///
/// The measured bevel button: wider than tall, so the bar's ends read as
/// controls set into it rather than as squares punched out of it.
pub const STATUS_BAR_BUTTON_WIDTH: f32 = 43.0;

/// Height of a corner button on the status bar, in points.
///
/// Seventeen of the bar's twenty-eight: the bevel and its borders, with the
/// bar's gradient showing above and below.
pub const STATUS_BAR_BUTTON_HEIGHT: f32 = 17.0;

/// Corner radius of the measured bevel buttons, in points.
pub const STATUS_BAR_BUTTON_RADIUS: u8 = 3;

/// Size of the glyph drawn in a corner button, in points.
///
/// The measured bevel's own caption size: twelve points of sign set in
/// seventeen points of metal, the button reading as a keycap with a mark on
/// it rather than as a glyph with a hair of margin.
pub const STATUS_BAR_BUTTON_GLYPH_SIZE: f32 = 12.0;

/// Gap between a corner button and its end of the status bar, in points.
///
/// Fifteen, as measured: the buttons sit clear of the window's corners, the
/// way a real toolbar's controls sit in their panel rather than in it.
pub const STATUS_BAR_BUTTON_INSET: f32 = 15.0;

/// Gap between two buttons sharing one end of the status bar, in points.
///
/// The left end holds three — the `+` that names a playlist, the shuffle
/// switch and the repeat switch — a measured stride of 47 apart, which is
/// the button's 43 plus this.
pub const STATUS_BAR_BUTTON_GAP: f32 = 4.0;

/// The face of a resting bevel button, at its top.
pub const STATUS_BUTTON_TOP: Color32 = Color32::from_rgb(252, 252, 252);

/// The face of a resting bevel button, at its bottom.
pub const STATUS_BUTTON_BOTTOM: Color32 = Color32::from_rgb(181, 181, 181);

/// The face of a bevel button under the pointer: the same fall, lit.
pub const STATUS_BUTTON_HOVER_TOP: Color32 = Color32::from_rgb(255, 255, 255);

/// …and its bottom.
pub const STATUS_BUTTON_HOVER_BOTTOM: Color32 = Color32::from_rgb(210, 210, 210);

/// The rule round a bevel button.
pub const STATUS_BUTTON_BORDER: Color32 = Color32::from_rgb(118, 118, 118);

/// The line of light along a bevel button's inside top, under its border:
/// the chamfer catching the room light, measured off the capture.
pub const STATUS_BUTTON_GLOSS: Color32 = Color32::from_rgb(255, 255, 255);

/// Paint one of the status bar's bevel buttons.
///
/// White at its top falling to silver at its foot, a grey rule around it and
/// a white line under that rule; `top` and `bottom` are the caller's choice
/// of the two falls — rest, hover, or the turned-over face of a switch that
/// is on. Square corners would poke out of the measured 3-point rounding by
/// a pixel's worth at each corner, which is what the stroked rule on top
/// covers.
pub fn status_button_surface(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    painter.add(egui::epaint::Shape::gradient_rect(
        rect,
        egui::epaint::Direction::TopDown,
        [top, bottom],
    ));
    painter.rect_stroke(
        rect,
        CornerRadius::same(STATUS_BAR_BUTTON_RADIUS),
        Stroke::new(1.0, STATUS_BUTTON_BORDER),
        StrokeKind::Inside,
    );
    painter.hline(
        rect.x_range().shrink(3.0),
        rect.top() + 1.5,
        Stroke::new(1.0, STATUS_BUTTON_GLOSS),
    );
}

/// Accent colour used for selections and links in both themes.
pub const ACCENT: Color32 = Color32::from_rgb(10, 132, 255);

/// The inner band of the keyboard focus ring.
///
/// egui draws no focus ring of its own — a focused button and an unfocused one
/// paint identically — so the interface draws this itself wherever the keyboard
/// can rest: the hand-painted rows, the sidebar entries, and the glyph
/// controls. It is [`ACCENT`], the colour every selection in the interface
/// already speaks.
pub const FOCUS_RING: Color32 = ACCENT;

/// The hairline outside the focus ring's accent band.
///
/// ACCENT alone is under the 3:1 a focus indicator owes its surroundings on
/// the light surfaces — the bars' mid gray and the sidebar's blue especially —
/// so a dark hairline rides just outside it. On the dark theme's panel the
/// hairline sinks into the panel and the accent band carries the ring; on the
/// light surfaces the hairline is what makes the edge. One of the two bands
/// clears 3:1 on every surface the ring is ever drawn on, which is what the
/// test below pins.
pub const FOCUS_RING_EDGE: Color32 = Color32::from_rgb(24, 24, 28);

/// Width of the focus ring's accent band, in points.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// Width of the focus ring's hairline, in points.
pub const FOCUS_RING_EDGE_WIDTH: f32 = 1.0;

/// Draw the keyboard focus ring inside `rect`.
///
/// Two bands, both measured inwards from `rect`, so the ring never paints
/// outside the control it marks — the rows are painted through a clipped
/// painter and a ring standing outside the row would be shaved by the clip.
pub fn focus_ring(painter: &Painter, rect: Rect) {
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(FOCUS_RING_EDGE_WIDTH, FOCUS_RING_EDGE),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(FOCUS_RING_EDGE_WIDTH),
        CornerRadius::same(4),
        Stroke::new(FOCUS_RING_WIDTH, FOCUS_RING),
        StrokeKind::Inside,
    );
}

/// Draw the focus ring over `response` when the keyboard is on it.
///
/// egui 0.36 moves focus only under the keyboard — Tab, the arrow keys, an
/// AccessKit request — and never on click, so the ring belongs to the
/// keyboard alone: the pointer has hover and selection to speak with, and the
/// ring says exactly one thing, which is where Enter will land.
pub fn focus_ring_if_focused(ui: &Ui, response: &Response) {
    if response.has_focus() {
        focus_ring(ui.painter(), response.rect);
    }
}

/// Name `response`'s control for a screen reader.
///
/// egui hands AccessKit the widget's own text as its name, which is fine for a
/// button reading `Clear` and useless for one reading `⏵`. This overwrites the
/// name with words — the same string the control already shows as hover text,
/// so pointer and keyboard users are told the same thing. It is cheap and
/// inert when AccessKit is not active for the frame (screen readers, tests):
/// egui returns `None` and the closure never runs.
pub fn accessible_name(ui: &Ui, response: &Response, name: &str) {
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(name));
}

/// Background colour of the navigation sidebar.
///
/// The blue-gray of the reference's source list: measured (209, 215, 226),
/// light enough that the white card of a playlist's icon still stands on it,
/// and quiet enough that the selection bar is the loudest thing in the panel.
/// It is deliberately not taken from the active theme: the sidebar is a light
/// surface in both the light and the dark theme.
pub const SIDEBAR_FILL: Color32 = Color32::from_rgb(209, 215, 226);

/// The top end of the selected row's bar.
///
/// The source list selected a row not with a rounded button fill but with a
/// flat bar across the panel's whole width, deepening from a bright blue at
/// its top to a deep blue at its bottom. The two ends here are the
/// reference's own measured pixels from a focused window — top (93, 148,
/// 214), the same blue [`FLAT_SELECTION`] is, and bottom (28, 88, 174). An
/// earlier pinning read an unfocused window and came out a washed slate. The
/// white label sits across the whole ramp, and the ramp is deep enough at
/// its foot that the label reads — see the tests below for the contrast
/// each end gives it.
pub const SIDEBAR_SELECTED_TOP: Color32 = Color32::from_rgb(93, 148, 214);

/// The bottom end of the selected row's bar.
pub const SIDEBAR_SELECTED_BOTTOM: Color32 = Color32::from_rgb(28, 88, 174);

/// The single darker line the reference lays along the bar's top edge —
/// (62, 120, 198), a shade below the gradient's own first row, so the bar
/// sits in the panel instead of floating over it.
pub const SIDEBAR_SELECTED_EDGE: Color32 = Color32::from_rgb(62, 120, 198);

/// The top end of the hover bar: the same idea a step quieter, because the
/// pointer's claim has to stay under the selection's. (The reference shows no
/// hover in its still frame; this ramp is the selection's own blue washed
/// nearly back into the fill.)
pub const SIDEBAR_HOVER_TOP: Color32 = Color32::from_rgb(222, 229, 243);

/// The bottom end of the hover bar.
pub const SIDEBAR_HOVER_BOTTOM: Color32 = Color32::from_rgb(181, 195, 222);

/// Row label colour while the selection bar lies under it.
///
/// White, because the bar deepens under the label exactly as the reference's
/// did — the label steps onto white when the blue arrives.
pub const SIDEBAR_INK_SELECTED: Color32 = Color32::WHITE;

/// Colour of the sidebar's group headings.
///
/// The reference set its source-list groups ("LIBRARY", "PLAYLISTS") in
/// slate capitals rather than in the list's own ink, measured (80, 93, 109),
/// so the heading reads as a name for what follows instead of as another row
/// waiting to be clicked.
pub const SIDEBAR_HEADING: Color32 = Color32::from_rgb(80, 93, 109);

/// Colour of the row icons — the note's head, stem and flag, and the card's
/// edge and ruling. The reference's icon blue measured (51, 112, 178) at the
/// glyph's edge; this is that blue with its core kept dark enough to hold
/// contrast on the pale panel.
pub const SIDEBAR_ICON: Color32 = Color32::from_rgb(36, 92, 162);

/// The same icon as it reads ON the selection bar.
///
/// The reference does not turn its icon white when the bar arrives — the
/// icon's own navy simply deepens against the paler blue, measured (7, 45,
/// 103). The card's paper stays paper either way.
pub const SIDEBAR_ICON_SELECTED: Color32 = Color32::from_rgb(7, 45, 103);

/// The hairline that separates the sidebar from the page beside it.
///
/// Measured (64, 64, 64) — a near-black rule, not the soft seam one expects:
/// the reference's source list and its white page end on a hard edge, and
/// anything paler would let the two surfaces melt together at the join.
pub const SIDEBAR_EDGE: Color32 = Color32::from_rgb(64, 64, 64);

/// Label colour of the navigation sidebar.
///
/// The sidebar is a light surface, but the theme's own foreground is near-white
/// in the dark theme and would vanish on [`SIDEBAR_FILL`], so the sidebar
/// carries its own dark ink in every theme — the near-black the reference's
/// row labels measure at their cores.
pub const SIDEBAR_INK: Color32 = Color32::from_rgb(28, 30, 38);

/// Muted [`SIDEBAR_INK`], for the placeholder line the list shows while it
/// has nothing to list. The group headings took this colour too, until the
/// sidebar learned iTunes 7's grey capitals — [`SIDEBAR_HEADING`].
pub const SIDEBAR_INK_MUTED: Color32 = Color32::from_rgb(70, 92, 116);

/// Background colour of the playback bar.
///
/// The status bar's own gray, so the two bars that frame the window stand as
/// one surface rather than as two shades of one colour. Like [`SIDEBAR_FILL`]
/// it is deliberately not taken from the active theme, so the bar's surface
/// does not shift with the theme while its contents are inked for it.
pub const PLAYBACK_BAR_FILL: Color32 = Color32::from_rgb(176, 178, 182);

/// Label colour of the playback bar.
///
/// The bar is a mid-gray surface in every theme, so it carries its own dark
/// ink; the theme's foreground is near-white in the dark theme.
pub const PLAYBACK_BAR_INK: Color32 = Color32::from_rgb(32, 32, 36);

/// Muted [`PLAYBACK_BAR_INK`], for the elapsed and total times.
pub const PLAYBACK_BAR_INK_MUTED: Color32 = Color32::from_rgb(58, 60, 66);

/// Fill of an idle control on the playback bar.
///
/// WHITE, at the user's request, where it was a step of the bar's own gray: the
/// bar's controls and the search field are the one surface on the bar that is
/// not its gray, so a button reads as a button rather than as a tile of the bar.
/// The bar still supplies everything drawn ON them — [`PLAYBACK_BAR_INK`] reads
/// on white at about 13:1 — and the ramp under this steps so a hover and a held
/// control still show themselves.
pub const PLAYBACK_BAR_CONTROL: Color32 = Color32::WHITE;

/// Fill of a control on the playback bar the pointer is over.
///
/// A light step of [`PLAYBACK_BAR_CONTROL`] rather than a darker shade: the
/// control is a white tile on the bar, so its states step AWAY from the white
/// and settle back toward the bar's gray, and the pointer still has a sign.
pub const PLAYBACK_BAR_CONTROL_HOVER: Color32 = Color32::from_rgb(224, 224, 229);

/// Fill of a control on the playback bar that is held or selected.
///
/// A mid-gray, and the one step of the ramp that overshoots: it is also what
/// the seek bar's UNGROOVED rail is filled with, and that rail is drawn on the
/// centre box's cream rather than on the bar — a tile that had stepped all the
/// way down from the white would be a hairline of nothing there. Held controls
/// take it too, so a press is plainly visible.
pub const PLAYBACK_BAR_CONTROL_ACTIVE: Color32 = Color32::from_rgb(156, 158, 163);

/// Ink of the playback bar's controls that have no surface of their own: the
/// speaker at the head of the volume slider and the slider's rail, both drawn
/// straight onto the bar's gray. White, at the user's request.
///
/// The three transport glyphs took this colour too until the user asked for the
/// buttons to be white as well: their button is now a white surface, so they
/// take [`PLAYBACK_BAR_INK`], which is what reads on it. What is left here is
/// the pair drawn ON the bar rather than on a control — white on the bar's gray
/// is about 2.1:1, which is the price of the request, and it is the direction of
/// that ratio the test below pins.
pub const PLAYBACK_BAR_CONTROL_INK: Color32 = Color32::WHITE;

/// Outline of the playback bar's controls, so they stay distinct from the bar
/// itself without needing a shadow. The same outline the seek bar's rail is
/// drawn with.
pub const PLAYBACK_BAR_CONTROL_EDGE: Color32 = Color32::from_rgb(84, 86, 92);

/// Fill of the playback bar's centre box, in points.
///
/// A pale yellow-green — [`CENTRE_DISC_TOP`], the measured top of the
/// reference's disc panel — so the box reads as a window onto the track
/// rather than as another panel of the bar's gray. Like the bar's own fill
/// it is deliberately not taken from the active theme: the box is this pale
/// surface in the light theme and in the dark one.
pub const CENTRE_BOX_FILL: Color32 = CENTRE_DISC_TOP;

/// Label colour inside the centre box.
///
/// The box is a light surface, so it carries its own dark ink; the theme's
/// foreground is near-white in the dark theme and would vanish on the pale fill.
pub const CENTRE_BOX_INK: Color32 = Color32::from_rgb(38, 34, 24);

/// Muted [`CENTRE_BOX_INK`], for the line under the title.
pub const CENTRE_BOX_INK_MUTED: Color32 = Color32::from_rgb(92, 84, 62);

/// Fill of a control inside the centre box, and of the empty cover slot.
///
/// A step of the box's own cream rather than the bar's gray, so a control on
/// the box stays part of the box.
pub const CENTRE_BOX_CONTROL: Color32 = Color32::from_rgb(226, 214, 180);

/// The centre box's rim: the measured border of the reference's disc panel,
/// (86, 88, 76) — an olive so dark it reads as an outline rather than a colour.
///
/// The player drew everything as an object — no edge-less fields — and on a
/// gray bar a pale field with no edge dissolves into it. One hairline is
/// what makes the box a thing sitting ON the bar rather than a hole in it.
pub const CENTRE_BOX_EDGE: Color32 = Color32::from_rgb(86, 88, 76);

/// The light that gathers on raised edges: the inner line just inside the
/// rim, and the lit top half of a push button.
pub const CENTRE_BOX_GLOSS: Color32 = Color32::from_rgb(252, 247, 231);

/// A control pushed in: the box's cream sunk a step below its resting
/// [`CENTRE_BOX_CONTROL`], the way a button goes dark where it has been
/// pressed and stays dark while it is held.
pub const CENTRE_BOX_PRESSED: Color32 = Color32::from_rgb(213, 199, 163);

/// Paint the centre box's surface the way those panels were drawn: the fill,
/// a hairline rim so it stands off the bar, and a lighter line just inside
/// the rim — the edge catching the light, which is what makes a flat shape
/// read as raised.
///
/// The fill is painted first and alone in its colour, so whatever looks the
/// box up by its fill — tests and callers alike — still finds it.
pub fn centre_box_surface(painter: &Painter, rect: Rect) {
    let radius = CornerRadius::same(CENTRE_BOX_RADIUS);
    // The measured panel is a pale so even across forty-four points that a
    // flat fill reads alike with it; the rim is the measured border, one
    // hairline, and the box's controls carry their own glosses on top.
    painter.rect_filled(rect, radius, CENTRE_BOX_FILL);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, CENTRE_BOX_EDGE),
        StrokeKind::Inside,
    );
}

/// Height of the Indexing bar drawn inside the centre box, in points.
///
/// The old iTunes encoding sheet's bar was slim — a line of work, not a
/// panel — and the centre box's cover row has exactly this much to spare
/// under the wording.
pub const CANDY_BAR_HEIGHT: f32 = 12.0;

/// The groove the Indexing bar fills: a step darker than the cream, so the
/// empty part of the bar reads as cut into the box rather than printed on it.
pub const CANDY_GROOVE_FILL: Color32 = Color32::from_rgb(206, 199, 185);

/// The groove's hairline border — iTunes drew its tracks rimmed, the way a
/// real inset one is.
pub const CANDY_GROOVE_EDGE: Color32 = Color32::from_rgb(164, 156, 141);

/// The fill's top, and its bottom: one gradient from pale blue to the app's
/// own accent, which is how the glossy blue of a Aqua-era progress bar got
/// its depth without any actual glass.
pub const CANDY_TOP: Color32 = Color32::from_rgb(126, 180, 255);
pub const CANDY_BOTTOM: Color32 = Color32::from_rgb(10, 132, 255);

/// The single light line along the fill's top edge — the reflection.
pub const CANDY_HIGHLIGHT: Color32 = Color32::from_rgb(235, 245, 255);

/// Draw a progress bar the way the iTunes 7 encoding sheet drew its one: a
/// slim inset groove with a glossy gradient fill.
///
/// `fraction` is how much of the groove belongs to the work done, or `None`
/// while the total is not yet known — then the gloss becomes a short stripe
/// that slides across the groove, the way those sheets said "working" without
/// promising a number it did not have. The stripe's position reads the clock
/// off the frame input, which is why the frame loop repaints while a job runs.
pub fn candy_progress(ui: &Ui, rect: Rect, fraction: Option<f32>) {
    let radius = CornerRadius::same(3);
    let stroke = Stroke::new(1.0, CANDY_GROOVE_EDGE);
    ui.painter()
        .rect(rect, radius, CANDY_GROOVE_FILL, stroke, StrokeKind::Inside);

    let inner = rect.shrink(1.0);

    let stripe: Rect = match fraction {
        Some(fraction) => {
            let fraction = fraction.clamp(0.0, 1.0);
            if fraction <= 0.0 {
                return;
            }
            Rect::from_min_max(
                inner.left_center(),
                Pos2::new(inner.left() + inner.width() * fraction, inner.bottom()),
            )
        }
        None => {
            let width = 26.0;
            let span = inner.width() + 2.0 * width;
            let travelled = (ui.input(|input| input.time) as f32 * 90.0) % span;
            Rect::from_min_size(
                Pos2::new(inner.left() - width + travelled, inner.top()),
                Vec2::new(width, inner.height()),
            )
            .intersect(inner)
        }
    };

    if stripe.width() <= 0.0 {
        return;
    }

    // The gloss is a mesh gradient, so it is square where the groove is
    // rounded; the groove's rim is stroked again over it, which is what keeps
    // the fill inside the rounded ends no matter how far the stripe has run.
    ui.painter().add(egui::epaint::Shape::gradient_rect(
        stripe,
        egui::epaint::Direction::TopDown,
        [CANDY_TOP, CANDY_BOTTOM],
    ));

    // The reflection only reads as one on a stretch wide enough to hold a
    // line; on a sliver it would just be the sliver's top edge twice.
    if stripe.width() > 6.0 {
        ui.painter().hline(
            stripe.x_range().shrink(2.0),
            stripe.top() + 1.0,
            Stroke::new(1.0, CANDY_HIGHLIGHT),
        );
    }

    ui.painter()
        .rect_stroke(rect, radius, stroke, StrokeKind::Inside);
}

/// Color used for non-fatal warnings, such as a settings file that failed to
/// load.
pub const WARNING: Color32 = Color32::from_rgb(255, 159, 10);

/// Background colour of the status bar across the bottom of the window.
///
/// A mid gray, well darker than the content between the bars, so the two that
/// frame the window read as a frame. It is also what [`PLAYBACK_BAR_FILL`] is
/// set to: the top bar and the status bar are one colour, and this is the
/// constant both read.
///
/// The status bar itself wears the measured gradient below rather than this
/// flat fill — see [`status_bar_surface`]; this constant is what its frame
/// falls back to and what the playback bar matches.
pub const STATUS_BAR_FILL: Color32 = Color32::from_rgb(176, 178, 182);

/// The status bar's gradient, at its top and its foot.
///
/// Measured off the capture: the bar is lit from above, `224` at the divider
/// deepening to `149` at the bottom border. The frame keeps the flat
/// [`STATUS_BAR_FILL`] underneath so the two bars can still be looked up by
/// one colour, and the gradient is painted over it.
pub const STATUS_GRADIENT_TOP: Color32 = Color32::from_rgb(224, 224, 224);

/// …and at its foot.
pub const STATUS_GRADIENT_BOTTOM: Color32 = Color32::from_rgb(149, 149, 149);

/// The hairline that separates the status bar from the page above it.
pub const STATUS_DIVIDER: Color32 = Color32::from_rgb(85, 85, 85);

/// The bar's bottom border, the last row of pixels of the window.
pub const STATUS_BOTTOM_BORDER: Color32 = Color32::from_rgb(71, 71, 71);

/// Paint the status bar's measured face across its strip: the top-to-foot
/// gradient, the divider along its top and the border along its foot. Call it
/// inside the bar's panel so its controls sit on the gradient.
pub fn status_bar_surface(painter: &Painter, rect: Rect) {
    painter.add(egui::epaint::Shape::gradient_rect(
        rect,
        egui::epaint::Direction::TopDown,
        [STATUS_GRADIENT_TOP, STATUS_GRADIENT_BOTTOM],
    ));
    // A stroke stands centred on the line it is given, so each rule is set
    // half a point inside the strip: at the very edge, half of it would fall
    // outside the panel's clip and the measured hairline would come out bald.
    painter.hline(
        rect.x_range(),
        rect.top() + 0.5,
        Stroke::new(1.0, STATUS_DIVIDER),
    );
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, STATUS_BOTTOM_BORDER),
    );
}

/// Label colour of the status bar.
///
/// The bar is a light surface in both themes, so it carries its own dark ink:
/// the theme's foreground is near-white in the dark theme and would vanish on
/// [`STATUS_BAR_FILL`]. This is the bevel buttons' ink; their face is pale
/// enough that the measured dark grey reads on it.
pub const STATUS_BAR_INK: Color32 = Color32::from_rgb(60, 60, 60);

/// Muted [`STATUS_BAR_INK`], for the status bar's single counts line.
pub const STATUS_BAR_INK_MUTED: Color32 = Color32::from_rgb(85, 85, 85);

// ————— Settings dialog —————
//
// The reference's Preferences sheet is light in both themes — a dark dialog
// over a light sheet is the same contradiction the content page refused — so
// the whole dialog takes these measured colours rather than the theme's, and
// [`page_visuals`] for its widgets.

/// The dialog's border, one point all round the sheet.
pub const SETTINGS_EDGE: Color32 = Color32::from_rgb(83, 86, 89);

/// The dialog's title bar, the one dark strip on a light sheet.
pub const SETTINGS_BAR: Color32 = Color32::from_rgb(41, 44, 48);

/// The title's ink on [`SETTINGS_BAR`].
pub const SETTINGS_BAR_INK: Color32 = Color32::from_rgb(232, 232, 232);

/// The close mark's ink on [`SETTINGS_BAR`] — the title's, set back a touch.
pub const SETTINGS_BAR_CLOSE: Color32 = Color32::from_rgb(207, 207, 207);

/// The dialog's face: the strip under the tabs and the footer.
pub const SETTINGS_FACE: Color32 = Color32::from_rgb(245, 245, 245);

/// The hairlines that rule the sheet: the pane's border and the tabs' edges.
pub const SETTINGS_LINE: Color32 = Color32::from_rgb(174, 174, 174);

/// The hairlines inside a pane, between one group of settings and the next.
///
/// A shade darker than [`SETTINGS_LINE`], measured: the inner rule has to
/// show on white where the border only has to show on the face.
pub const SETTINGS_HAIR: Color32 = Color32::from_rgb(166, 166, 166);

/// The dialog's own ink: everything the sheet says.
pub const SETTINGS_INK: Color32 = Color32::from_rgb(17, 17, 17);

/// The unchosen tabs' gradient, at its top…
pub const SETTINGS_TAB_TOP: Color32 = Color32::from_rgb(251, 251, 251);

/// …and at its foot. The chosen tab is the sheet's own white instead, and
/// taller by three points — the height is how the strip says which one is on.
pub const SETTINGS_TAB_BOTTOM: Color32 = Color32::from_rgb(233, 233, 233);

/// The footer buttons' edge.
pub const SETTINGS_BUTTON_EDGE: Color32 = Color32::from_rgb(185, 185, 185);

/// The footer buttons' fall, at top…
pub const SETTINGS_BUTTON_TOP: Color32 = Color32::from_rgb(255, 255, 255);

/// …and bottom.
pub const SETTINGS_BUTTON_BOTTOM: Color32 = Color32::from_rgb(237, 237, 237);

/// The default button's ring: two points of it, and the hairline behind the
/// ring. The rest of the face runs pale blue rather than the Cancel's gray.
pub const SETTINGS_DEFAULT_RING: Color32 = Color32::from_rgb(55, 150, 246);

/// The hairline that stands off the default button's ring.
pub const SETTINGS_DEFAULT_HALO: Color32 = Color32::from_rgb(156, 196, 238);

/// The default button's fall, at top…
pub const SETTINGS_DEFAULT_TOP: Color32 = Color32::from_rgb(255, 255, 255);

/// …and bottom — the blue of the ring, thinned to almost nothing.
pub const SETTINGS_DEFAULT_BOTTOM: Color32 = Color32::from_rgb(244, 250, 255);

/// Install the iCraftMusic style for `mode`.
///
/// This sets the egui theme preference and then adjusts both the dark and the
/// light style, so switching themes later never loses the customisations.
pub fn apply(ctx: &Context, mode: ThemeMode) {
    ctx.set_theme(preference(mode));
    ctx.all_styles_mut(customise);
}

/// Translate a stored preference into egui's own theme preference.
fn preference(mode: ThemeMode) -> ThemePreference {
    match mode {
        ThemeMode::Dark => ThemePreference::Dark,
        ThemeMode::Light => ThemePreference::Light,
        ThemeMode::System => ThemePreference::System,
    }
}

/// Apply the iCraftMusic spacing and rounding to a single style.
fn customise(style: &mut Style) {
    style.spacing.item_spacing = vec2(8.0, 6.0);
    style.spacing.button_padding = vec2(10.0, 6.0);

    // Nothing is set in egui's "small" size: its 9.0 is too small to read on a
    // desktop at arm's length, and every one of this interface's labels is
    // either a heading, a control, or — where a distinction is wanted — a
    // *muted* line, which is what carries the hierarchy instead of the size.
    // Raising it here covers the whole interface at once, both themes included,
    // and leaves every `.small()` call site saying what it means.
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(SMALL_TEXT_SIZE, FontFamily::Proportional),
    );

    style.visuals.hyperlink_color = ACCENT;
    style.visuals.selection.bg_fill = ACCENT;
    style.visuals.window_corner_radius = CornerRadius::same(10);

    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(6);
    }
}

/// The content page's sheet: white in every theme.
///
/// iTunes 7's source pages painted their lists on a white sheet and left the
/// grey to the chrome around them, and this window follows it: the page is
/// the one surface that does not change with the theme, which is also what
/// keeps the fixed pale chrome — the source list's wash, the bar's gray, the
/// centre box's cream — from having a dark square punched through it when
/// the system asks for a dark theme. The widgets that sit on it come from
/// [`page_visuals`], not from the theme.
pub const PAGE_FILL: Color32 = Color32::WHITE;

/// Frame used by the content page.
///
/// It fills with [`PAGE_FILL`] rather than the theme's panel fill, so the
/// page's ink has to come from [`page_visuals`] rather than from the theme
/// either. The margin is the one a default central panel carries, so nothing
/// on the page shifts when the sheet arrives.
pub fn page_frame() -> Frame {
    Frame::new().inner_margin(Margin::same(8)).fill(PAGE_FILL)
}

/// [`page_frame`] with its top inset gone.
///
/// The Library's track page runs its table header up to the chrome: in the
/// reference the header band sits directly under the bar's dark rule, the
/// two sharing an edge with no strip of the sheet's white between them.
/// Every other page starts with words, which want the inset — this frame
/// is for the one page whose first row is a band meant to touch.
pub fn page_frame_flush() -> Frame {
    Frame::new()
        .inner_margin(Margin {
            left: 8,
            right: 8,
            top: 0,
            bottom: 8,
        })
        .fill(PAGE_FILL)
}

/// The visuals for the page area, the same in every theme.
///
/// Because the page paints [`PAGE_FILL`] in both themes, it takes the light
/// visuals whole rather than ink alone: a page's buttons, combo boxes,
/// check boxes and sliders are surfaces as much as its labels are, and a
/// dark widget floating on a white sheet is neither theme's. The ink that
/// comes with the light visuals is already the dark ink a white sheet
/// needs. The accent — selection and links — matches what `customise` gives
/// both themes; spacing and rounding stay as the shared style set them,
/// since this replaces only the visuals.
pub fn page_visuals() -> Visuals {
    let mut visuals = Visuals::light();
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT;
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals
}

/// The tint a marked album tile or artist row wears: [`ACCENT`] at a
/// transparency that lets the cover beneath it stay the picture.
///
/// A click on the group pages marks rather than searches, so the mark has to
/// read as "this one" over artwork rather than as a row highlight — a flat
/// opaque bar would hide the sleeve it is pointing at.
pub const GROUP_SELECTED_FILL: Color32 = Color32::from_rgba_premultiplied(10, 132, 255, 90);

/// Frame used by the navigation sidebar.
///
/// It fills with [`SIDEBAR_FILL`] rather than the theme's panel fill, so it
/// takes no style. The side margins are zero on purpose: a frame margin on a
/// side panel grows the panel's rect outward rather than padding it inward,
/// which slid the whole source list eight pixels past the window's edge.
/// With none, the rows are already full-width bands, and the few pixels of
/// room the headings and the placeholder want are added where they are drawn.
pub fn sidebar_frame() -> Frame {
    Frame::new()
        .inner_margin(Margin::symmetric(0, 4))
        .fill(SIDEBAR_FILL)
}

/// Give a light surface its own ink.
///
/// The sidebar and the playback bar both paint a fill of their own in every
/// theme, so both have to supply their own ink: the theme's foreground is
/// near-white in the dark theme and would vanish on a light fill.
fn ink_surface(visuals: &mut Visuals, ink: Color32, ink_muted: Color32) {
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.fg_stroke.color = ink;
    }

    // The selected entry is filled with ACCENT, which is dark enough for white
    // text in both themes; the themes' own selection strokes are not.
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.weak_text_color = Some(ink_muted);
}

/// Point a style at the sidebar's own palette.
///
/// Call this inside a `Ui::scope` around the sidebar body. The sidebar paints
/// [`SIDEBAR_FILL`], so it supplies its own ink rather than inheriting the
/// theme's foreground, which is unreadable on a light fill in the dark theme.
///
/// Ink is all the style is asked for now. A row's hover and selection bars are
/// painted by the sidebar itself — the source list's ramps are flat, panel-wide
/// gradients, not the rounded button fill a widget style hands out — so the
/// style no longer dyes any widget fill: the only button in the panel is a
/// row's own, and it strips its fill off entirely.
pub fn apply_sidebar_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, SIDEBAR_INK, SIDEBAR_INK_MUTED);
}

/// Frame used by the playback bar.
///
/// It fills with [`PLAYBACK_BAR_FILL`] rather than the theme's panel fill, so it
/// takes no style.
pub fn playback_bar_frame() -> Frame {
    Frame::new()
        // The inner height is the measured disc panel's own forty-four points:
        // the box fills the bar's insides exactly, as it fills the reference's
        // band, and the transport and the volume ride centred in the same strip.
        .inner_margin(Margin::symmetric(16, 8))
        .fill(PLAYBACK_BAR_FILL)
}

/// Point a style at the playback bar's own palette.
///
/// Call this inside a `Ui::scope` around the bar's body. Like the sidebar, the
/// bar paints a fill of its own in every theme, so its controls take the bar's
/// grays instead of the theme's.
pub fn apply_playback_bar_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, PLAYBACK_BAR_INK, PLAYBACK_BAR_INK_MUTED);

    // A button is painted with `weak_bg_fill`; a slider takes its rail from
    // `bg_fill` and its handle from the same `bg_fill` through the interaction
    // state, so the rail is the darker of the two.
    //
    // `noninteractive` carries the resting fill too, because the transport is
    // DISABLED while nothing is queued and egui draws a disabled button from
    // that state: without it the three buttons would be white tiles with a track
    // playing and theme-coloured ones without, which is the same control
    // changing its surface to say something it is not.
    visuals.widgets.inactive.weak_bg_fill = PLAYBACK_BAR_CONTROL;
    visuals.widgets.noninteractive.weak_bg_fill = PLAYBACK_BAR_CONTROL;
    visuals.widgets.hovered.weak_bg_fill = PLAYBACK_BAR_CONTROL_HOVER;
    visuals.widgets.active.weak_bg_fill = PLAYBACK_BAR_CONTROL_ACTIVE;
    visuals.widgets.inactive.bg_fill = PLAYBACK_BAR_CONTROL_ACTIVE;
    visuals.widgets.hovered.bg_fill = PLAYBACK_BAR_CONTROL_HOVER;
    visuals.widgets.active.bg_fill = PLAYBACK_BAR_CONTROL_ACTIVE;

    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.bg_stroke.color = PLAYBACK_BAR_CONTROL_EDGE;
    }

    // The search field's caret has to show on the control shade the field is
    // filled with, the way its text does: egui's own caret is the theme's, which
    // is near-white in the dark theme and would vanish on the bar.
    visuals.text_cursor.stroke = Stroke::new(2.0, PLAYBACK_BAR_INK);
}

/// Points the playback bar's centre box leaves between its edge and its
/// contents, on its left and right.
///
/// The box is sized from this, what it holds and its own height, so the pill it
/// paints fits its contents exactly and cannot disagree with them.
pub const CENTRE_BOX_PAD: i8 = 12;

/// Points the centre box leaves above its contents and below them, in points.
///
/// Its top row and the seek row under it are placed from this and the bar's own
/// height, so the two rows sit inside the pill rather than being measured off
/// whatever a child happened to report.
pub const CENTRE_BOX_PAD_Y: i8 = 6;

/// Corner radius of the centre box, in points.
///
/// The measured radius of the reference's disc panel: a soft corner on a
/// rectangle, not the full round a pill would wear. Half the old box's height
/// is what it used to be, back when the box was as tall as two stacked rows.
pub const CENTRE_BOX_RADIUS: u8 = 6;

/// Point a style at the centre box's own palette.
///
/// Call this inside a `Ui::scope` around the box's contents. Like the bar it
/// sits on, the box paints a fill of its own in every theme, so its controls
/// take the box's cream and its ink rather than the bar's gray.
pub fn apply_centre_box_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, CENTRE_BOX_INK, CENTRE_BOX_INK_MUTED);

    // A resting control on the box is the box, and only shows itself under the
    // pointer or while it is on, exactly as a control on the bar does.
    visuals.widgets.inactive.weak_bg_fill = CENTRE_BOX_FILL;
    visuals.widgets.noninteractive.weak_bg_fill = CENTRE_BOX_FILL;
    visuals.widgets.hovered.weak_bg_fill = CENTRE_BOX_CONTROL;
    visuals.widgets.active.weak_bg_fill = CENTRE_BOX_CONTROL;
    visuals.widgets.inactive.bg_fill = CENTRE_BOX_CONTROL;
    visuals.widgets.hovered.bg_fill = CENTRE_BOX_CONTROL;
}

/// Frame used by the status bar.
///
/// It fills with [`STATUS_BAR_FILL`] rather than the theme's panel fill, so it
/// takes no style. The bar's one line centres itself rather than being inset by
/// the frame, so the body gets the whole bar and the fill reaches every edge.
pub fn status_bar_frame() -> Frame {
    Frame::new().fill(STATUS_BAR_FILL)
}

/// Point a style at the status bar's own palette.
///
/// Call this inside a `Ui::scope` around the bar's body. Like the sidebar and
/// the playback bar, the status bar paints a fill of its own in every theme, so
/// its line takes the bar's ink rather than the theme's foreground.
pub fn apply_status_bar_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, STATUS_BAR_INK, STATUS_BAR_INK_MUTED);

    // The bar's buttons are hand-bevelled — the measured keycap is painted
    // under each glyph by `status_button_surface`, not by egui — so every one
    // of their own fills goes transparent and the widget is only its text and
    // its click area. That is true at rest and under the pointer alike: the
    // bevel does the resting face, the lit face and the turned-over face, so
    // egui must not lay a tile of its own over any of them.
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.noninteractive,
    ] {
        widget.bg_fill = Color32::TRANSPARENT;
        widget.weak_bg_fill = Color32::TRANSPARENT;
        widget.bg_stroke = Stroke::NONE;
    }

    // `RichText::strong()` needs no colour of its own here: egui resolves it
    // through `Visuals::strong_text_color()`, which is `widgets.active`'s text
    // colour, and `ink_surface` above has just set that to the bar's ink. So
    // the bar's glyphs can say `strong()` and mean it, without a second place
    // to keep in step.
}

/// Relative luminance of a colour, as WCAG defines it.
#[cfg(test)]
fn luminance(colour: Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    let [r, g, b, _] = colour.to_array();
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contrast ratio between two colours, as WCAG defines it.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let (lighter, darker) = {
            let (a, b) = (luminance(a), luminance(b));
            if a >= b { (a, b) } else { (b, a) }
        };
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn the_sidebar_is_light_blue() {
        let [r, g, b, a] = SIDEBAR_FILL.to_array();
        assert_eq!(a, 255, "the sidebar surface must be opaque");
        assert!(b > g && g > r, "blue must lead the sidebar's channels");
        assert!(
            luminance(SIDEBAR_FILL) > 0.6,
            "the sidebar must stay light — a source-list pale, not a mid-tone"
        );
    }

    #[test]
    fn the_sidebar_labels_stay_readable_on_the_sidebar() {
        for ink in [
            SIDEBAR_INK,
            SIDEBAR_INK_MUTED,
            SIDEBAR_HEADING,
            SIDEBAR_ICON,
        ] {
            assert!(
                contrast(ink, SIDEBAR_FILL) >= 4.5,
                "sidebar ink {ink:?} is too faint on {SIDEBAR_FILL:?}"
            );
        }
    }

    /// The selected row's bar is the source list's own shape: a flat ramp that
    /// darkens downward, standing clear of the panel's pale surface, with the
    /// white label reading on it at BOTH ends — the top being the hard case,
    /// since this build copies the ramp without the player's text shadow.
    #[test]
    fn the_selection_bar_darkens_downwards_under_white_ink() {
        assert!(
            luminance(SIDEBAR_SELECTED_TOP) > luminance(SIDEBAR_SELECTED_BOTTOM),
            "the selection bar must be lit along its top, not its bottom"
        );
        assert!(
            contrast(SIDEBAR_SELECTED_TOP, SIDEBAR_FILL) >= 1.5,
            "the bar's lighter end must still stand on the pale panel"
        );
        assert!(
            contrast(SIDEBAR_INK_SELECTED, SIDEBAR_SELECTED_BOTTOM) >= 3.0,
            "the white label cannot read on the bar's deep end, where its \
             lower half of pixels sit"
        );
        // The bar's pale end is the reference's own measured first row, and
        // white on it contrasts 2.2 — the reference's own number, taken at
        // 1:1 with it. WCAG's floors were not what 2007's source lists were
        // designed against; this records where the fidelity lands rather
        // than pretending the original was different.
        assert!(
            contrast(SIDEBAR_INK_SELECTED, SIDEBAR_SELECTED_TOP) >= 2.0,
            "the bar's pale end has drifted lighter than the reference's"
        );
    }

    /// The pointer's bar is the selection bar's ramp a step quieter: it has
    /// to show on the panel, and it must never outshine the bar that says a
    /// row is the open one.
    #[test]
    fn the_hover_bar_shows_on_the_panel_and_defers_to_the_selection() {
        assert!(
            contrast(SIDEBAR_HOVER_BOTTOM, SIDEBAR_FILL) > 1.2,
            "the hover bar's deeper end must be visible on the panel"
        );
        assert!(
            luminance(SIDEBAR_HOVER_TOP) > luminance(SIDEBAR_SELECTED_TOP),
            "the hover bar is darker than the selection bar, so they read alike"
        );
    }

    /// The panel's one divider against the page: a hairline, and unmistakably
    /// one — neither the panel's fill nor anything close to the row inks.
    #[test]
    fn the_sidebar_edge_is_a_darker_hairline_than_the_panel() {
        assert!(
            contrast(SIDEBAR_EDGE, SIDEBAR_FILL) >= 1.5,
            "the divider is invisible against the panel"
        );
        assert!(
            luminance(SIDEBAR_EDGE) > luminance(SIDEBAR_INK),
            "the divider must read as a line, not as text weight"
        );
    }

    #[test]
    fn the_sidebar_frame_paints_the_sidebar_colour() {
        assert_eq!(sidebar_frame().fill, SIDEBAR_FILL);
    }

    #[test]
    fn applying_the_sidebar_visuals_inks_the_entries() {
        let mut visuals = Visuals::light();
        apply_sidebar_visuals(&mut visuals);

        for widget in [
            &visuals.widgets.inactive,
            &visuals.widgets.hovered,
            &visuals.widgets.active,
        ] {
            assert_eq!(widget.fg_stroke.color, SIDEBAR_INK);
        }

        assert_eq!(visuals.weak_text_color, Some(SIDEBAR_INK_MUTED));
    }

    /// The style is asked for ink only: whatever fill a hovered or selected
    /// widget would reach for, the style must not dye it the panel's colour —
    /// the row's painted bars are the panel's whole width and a widget fill
    /// underneath them would fight their ramp.
    #[test]
    fn the_sidebar_style_dyes_no_widget_fill() {
        for mut visuals in [Visuals::light(), Visuals::dark()] {
            apply_sidebar_visuals(&mut visuals);

            for widget in [
                &visuals.widgets.hovered,
                &visuals.widgets.active,
                &visuals.widgets.noninteractive,
            ] {
                assert_ne!(widget.bg_fill, SIDEBAR_FILL);
            }
        }
    }

    #[test]
    fn the_playback_bar_is_the_status_bars_gray() {
        assert_eq!(
            PLAYBACK_BAR_FILL, STATUS_BAR_FILL,
            "the top bar and the status bar are not the same colour"
        );

        // And it still has to stand out from the content area between the two
        // bars, or neither would read as part of a frame around the window.
        assert!(
            luminance(PLAYBACK_BAR_FILL) < 0.6,
            "the bar is still the light gray it no longer is"
        );
    }

    /// A control on the playback bar has to stay visible where it is now a WHITE
    /// tile on the bar's gray: at rest it is the white itself, under the pointer
    /// a step down from it, and held a further step and the outline deeper
    /// again. The floors are the ratios this palette actually produces, so a
    /// later nudge toward the bar's own colour fails rather than sliding under
    /// the eye.
    #[test]
    fn the_playback_bars_controls_still_show_against_its_own_fill() {
        // The resting fill is the white the user asked for, and the bar is not.
        assert_eq!(
            PLAYBACK_BAR_CONTROL,
            Color32::WHITE,
            "a button on the bar no longer rests in the white it was asked for"
        );
        assert_ne!(
            PLAYBACK_BAR_CONTROL, PLAYBACK_BAR_FILL,
            "the controls were flattened into the bar"
        );

        // The resting white and the states it steps through: a hovered or held
        // control has to show against the white it rests in, which is what the
        // pointer reads — the bar's gray is behind all of them.
        assert!(
            contrast(PLAYBACK_BAR_CONTROL, PLAYBACK_BAR_FILL) >= 2.0,
            "the white control tile does not show on the bar: {:.2}",
            contrast(PLAYBACK_BAR_CONTROL, PLAYBACK_BAR_FILL)
        );
        let ramp = [
            ("hovered", PLAYBACK_BAR_CONTROL_HOVER, 1.2),
            ("held", PLAYBACK_BAR_CONTROL_ACTIVE, 1.5),
            ("edge", PLAYBACK_BAR_CONTROL_EDGE, 2.5),
        ];
        for (state, fill, at_least) in ramp {
            assert!(
                contrast(fill, PLAYBACK_BAR_CONTROL) >= at_least,
                "the {state} fill {fill:?} does not show against the resting white: \
                 {:.2} against {at_least}",
                contrast(fill, PLAYBACK_BAR_CONTROL)
            );
        }

        // The white ink is now only on the two controls with no surface of their
        // own — the speaker and the volume rail, drawn straight onto the bar's
        // gray. Its floor is a LOW one: white on a mid-gray bar is about 2.1:1,
        // which is the price of the user's request. What the assertion is for is
        // the direction — a later nudge of the bar's fill toward white would take
        // it to nothing, and that fails here.
        assert!(
            contrast(PLAYBACK_BAR_CONTROL_INK, PLAYBACK_BAR_FILL) >= 2.0,
            "the speaker's white ink is too faint on the bar: {:.2}",
            contrast(PLAYBACK_BAR_CONTROL_INK, PLAYBACK_BAR_FILL)
        );

        // And the bar's ink still reads on every control fill, since the
        // transport's glyphs, the Clear label and the seek bar are drawn in it
        // over them — and the field's comment in `nowplaying` says the query
        // reads at well over 10:1 on the white one.
        for fill in [
            PLAYBACK_BAR_CONTROL,
            PLAYBACK_BAR_CONTROL_HOVER,
            PLAYBACK_BAR_CONTROL_ACTIVE,
        ] {
            assert!(
                contrast(PLAYBACK_BAR_INK, fill) >= 4.5,
                "the bar's ink is too faint on the control fill {fill:?}: {:.2}",
                contrast(PLAYBACK_BAR_INK, fill)
            );
        }
    }

    /// The search field is a control on the bar rather than a field on a page,
    /// and it is filled with one of the bar's own control shades — the white —
    /// with the bar's ink for the query and the caret. The theme's text-edit
    /// fill is near-black in the dark theme, which would put the bar's dark ink
    /// on a black tile, and egui's own caret is the theme's, which would vanish
    /// on the field's light fill.
    #[test]
    fn the_search_field_wears_the_playback_bars_own_controls() {
        let mut visuals = Visuals::dark();
        apply_playback_bar_visuals(&mut visuals);

        assert_eq!(
            visuals.text_cursor.stroke.color, PLAYBACK_BAR_INK,
            "the caret would not show on the fill the field is drawn on"
        );
        assert_eq!(
            PLAYBACK_BAR_CONTROL,
            Color32::WHITE,
            "the bar's control fill is no longer the white the field is drawn in"
        );

        for what in ["the query", "the caret"] {
            assert!(
                contrast(PLAYBACK_BAR_INK, PLAYBACK_BAR_CONTROL) >= 4.5,
                "{what} is too faint on the field's fill: {:.2}",
                contrast(PLAYBACK_BAR_INK, PLAYBACK_BAR_CONTROL)
            );
        }
    }

    /// The centre box is the measured pale of the reference's disc panel, and
    /// its ink reads on it in both themes: the box paints its own fill
    /// whatever the theme, so the theme's foreground — near-white in the dark
    /// theme — would vanish on it.
    #[test]
    fn the_centre_box_is_the_measured_pale_and_its_ink_reads_on_it() {
        let [r, g, b, a] = CENTRE_BOX_FILL.to_array();
        assert_eq!(a, 255, "the box's surface must be opaque");
        assert_eq!(
            CENTRE_BOX_FILL, CENTRE_DISC_TOP,
            "the box wears the measured top of the reference's disc panel"
        );
        assert!(
            r > b && g > b,
            "the measured pale is a yellow-green: red and green lead blue"
        );
        assert!(
            luminance(CENTRE_BOX_FILL) > 0.7,
            "the box has to stay pale, not a tan: {CENTRE_BOX_FILL:?}"
        );

        for ink in [CENTRE_BOX_INK, CENTRE_BOX_INK_MUTED] {
            assert!(
                contrast(ink, CENTRE_BOX_FILL) >= 4.5,
                "box ink {ink:?} is too faint on {CENTRE_BOX_FILL:?}"
            );
        }
    }

    /// A control on the box stays part of the box: at rest it is the cream
    /// itself, and under the pointer a step of it, not the bar's gray.
    #[test]
    fn the_centre_box_holds_its_own_controls() {
        for theme in [Visuals::light(), Visuals::dark()] {
            let stock_resting = theme.widgets.inactive.weak_bg_fill;

            let mut visuals = theme;
            apply_centre_box_visuals(&mut visuals);

            assert_eq!(
                visuals.widgets.inactive.weak_bg_fill, CENTRE_BOX_FILL,
                "a resting control on the box is painted in the box's own cream"
            );
            assert_ne!(
                visuals.widgets.inactive.weak_bg_fill, stock_resting,
                "the control is still taking the theme's own resting fill"
            );
            assert_eq!(
                visuals.weak_text_color,
                Some(CENTRE_BOX_INK_MUTED),
                "the box's own muted ink did not reach its labels"
            );

            // The hover has to show, and the box's ink has to read on it.
            assert!(
                contrast(visuals.widgets.hovered.weak_bg_fill, CENTRE_BOX_FILL) > 1.1,
                "a hovered control must show against the box"
            );
            assert!(
                contrast(CENTRE_BOX_INK, CENTRE_BOX_CONTROL) >= 4.5,
                "the box's ink is too faint on a hovered control"
            );
        }
    }

    #[test]
    fn the_playback_bar_labels_stay_readable_on_the_bar() {
        for ink in [PLAYBACK_BAR_INK, PLAYBACK_BAR_INK_MUTED] {
            assert!(
                contrast(ink, PLAYBACK_BAR_FILL) >= 4.5,
                "bar ink {ink:?} is too faint on {PLAYBACK_BAR_FILL:?}"
            );
        }
    }

    #[test]
    fn the_playback_bar_frame_paints_the_bar_colour() {
        assert_eq!(playback_bar_frame().fill, PLAYBACK_BAR_FILL);
    }

    #[test]
    fn applying_the_playback_bar_visuals_lights_its_controls() {
        // The dark theme is the interesting case: left alone, its button fill
        // and its foreground are both wrong on the bar's own gray.
        let mut visuals = Visuals::dark();
        apply_playback_bar_visuals(&mut visuals);

        for widget in [
            &visuals.widgets.inactive,
            &visuals.widgets.hovered,
            &visuals.widgets.active,
        ] {
            assert_eq!(widget.fg_stroke.color, PLAYBACK_BAR_INK);
            assert!(
                contrast(widget.weak_bg_fill, PLAYBACK_BAR_FILL) > 1.1,
                "a control fill {:#?} would read as a hole in the bar",
                widget.weak_bg_fill
            );
        }

        assert_eq!(visuals.widgets.inactive.weak_bg_fill, PLAYBACK_BAR_CONTROL);
        assert_eq!(visuals.weak_text_color, Some(PLAYBACK_BAR_INK_MUTED));

        // The seek bar's rail and its handle share `inactive.bg_fill`, and both
        // have to stay visible against the bar itself.
        assert!(
            contrast(visuals.widgets.inactive.bg_fill, PLAYBACK_BAR_FILL) > 1.2,
            "the rail must be distinguishable from the bar"
        );
    }

    #[test]
    fn the_status_bar_is_a_gray_that_frames_the_window() {
        let [r, g, b, a] = STATUS_BAR_FILL.to_array();
        assert_eq!(a, 255, "the bar surface must be opaque");
        assert!(
            r.max(g).max(b) - r.min(g).min(b) <= 8,
            "the bar must read as a gray: {STATUS_BAR_FILL:?} leans on a channel"
        );
        assert!(
            luminance(STATUS_BAR_FILL) < 0.6,
            "the bar must be dark enough to read as a frame around the content"
        );
    }

    #[test]
    fn the_status_bar_label_stays_readable_on_the_bar() {
        // The glyphs are the measured (60,60,60): that clears the bar's gray
        // and both bevel faces at the 4.5 an interface holds itself to.
        assert!(
            contrast(STATUS_BAR_INK, STATUS_BAR_FILL) >= 4.5,
            "bar ink {STATUS_BAR_INK:?} is too faint on {STATUS_BAR_FILL:?}"
        );
        // The counts line is the measured (85,85,85) on a gradient that runs
        // (224,224,224) to (149,149,149): a reproduction inherits the
        // reference's own contrast, and the reference's counts sit at a soft
        // 3-to-odd — deliberately recessive, the one line on the bar that is
        // read by asking rather than by glancing. The floor here is 3.0, the
        // bar an indicator owes its surface, measured where the line lands:
        // centred in the band, on the gradient's middle.
        let mid = Color32::from_rgb(
            ((u16::from(STATUS_GRADIENT_TOP.r()) + u16::from(STATUS_GRADIENT_BOTTOM.r())) / 2)
                as u8,
            ((u16::from(STATUS_GRADIENT_TOP.g()) + u16::from(STATUS_GRADIENT_BOTTOM.g())) / 2)
                as u8,
            ((u16::from(STATUS_GRADIENT_TOP.b()) + u16::from(STATUS_GRADIENT_BOTTOM.b())) / 2)
                as u8,
        );
        assert!(
            contrast(STATUS_BAR_INK_MUTED, mid) >= 3.0,
            "the counts line {STATUS_BAR_INK_MUTED:?} vanishes on the bar's middle {mid:?}"
        );
    }

    #[test]
    fn the_status_bar_frame_paints_the_bar_colour() {
        let frame = status_bar_frame();
        assert_eq!(frame.fill, STATUS_BAR_FILL);
        assert_eq!(
            frame.inner_margin,
            Margin::ZERO,
            "the bar centres its own line, so the frame must leave it the whole height"
        );
    }

    #[test]
    fn applying_the_status_bar_visuals_inks_its_line() {
        // The dark theme is the interesting case: left alone, its foreground is
        // near-white and would vanish on the bar's gray.
        let mut visuals = Visuals::dark();
        apply_status_bar_visuals(&mut visuals);

        for widget in [
            &visuals.widgets.noninteractive,
            &visuals.widgets.inactive,
            &visuals.widgets.hovered,
            &visuals.widgets.active,
        ] {
            assert_eq!(widget.fg_stroke.color, STATUS_BAR_INK);
        }

        assert_eq!(visuals.weak_text_color, Some(STATUS_BAR_INK_MUTED));
    }

    /// A bar button is hand-bevelled: egui lays no tile of its own at rest,
    /// under the pointer, or pressed — the measured keycap under each glyph
    /// IS the button's face, and an egui fill laid over it would hide the
    /// bevel. What remains to hold is that the widget fills all went
    /// transparent in both themes, whatever the theme's own button fills
    /// happen to be.
    #[test]
    fn the_bar_buttons_wear_the_hand_bevel_and_no_egui_tile() {
        for theme in [Visuals::light(), Visuals::dark()] {
            let mut visuals = theme;
            apply_status_bar_visuals(&mut visuals);

            for widget in [
                &visuals.widgets.inactive,
                &visuals.widgets.hovered,
                &visuals.widgets.active,
                &visuals.widgets.noninteractive,
            ] {
                assert_eq!(widget.bg_fill, Color32::TRANSPARENT);
                assert_eq!(widget.weak_bg_fill, Color32::TRANSPARENT);
                assert_eq!(widget.bg_stroke, Stroke::NONE);
            }
        }

        // The three faces of the hand-painted bevel: the resting fall must
        // differ from the lit one, the lit one from the turned-over face of a
        // switch that is on, and every fall must carry the ink.
        assert_ne!(STATUS_BUTTON_TOP, STATUS_BUTTON_HOVER_TOP);
        assert!(
            contrast(STATUS_BAR_INK, STATUS_BUTTON_TOP) >= 4.5
                && contrast(STATUS_BAR_INK, STATUS_BUTTON_BOTTOM) >= 4.5,
            "the bar's glyph ink is too faint somewhere on the resting bevel"
        );
        assert!(
            contrast(STATUS_BAR_INK, STATUS_BUTTON_HOVER_BOTTOM) >= 4.5,
            "the bar's glyph ink is too faint on the lit bevel"
        );
        // And the rule around the button has to read against both the bar
        // under it and the pale face inside it.
        assert!(
            contrast(STATUS_BUTTON_BORDER, STATUS_BAR_FILL) > 1.2
                && contrast(STATUS_BUTTON_BORDER, STATUS_BUTTON_BOTTOM) > 1.2,
            "the bevel's rule is invisible against the bar or its own face"
        );
    }

    /// The bar's own face is the measured gradient between its two rules; the
    /// frame underneath stays the shared gray so the two bars of the window
    /// can still be looked up by one colour.
    #[test]
    fn the_status_bar_paints_its_gradient_between_its_two_rules() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1180.0, STATUS_BAR_HEIGHT));
        let ctx = Context::default();

        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(rect.expand(8.0)),
                ..Default::default()
            },
            |ui| status_bar_surface(ui.painter(), rect),
        );

        let gradients = output
            .shapes
            .iter()
            .filter(|clipped| matches!(clipped.shape, egui::Shape::Mesh(_)))
            .count();
        let lines: Vec<Color32> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::LineSegment { stroke, .. } => Some(stroke.color),
                _ => None,
            })
            .collect();
        output.textures_delta.clear();

        assert_eq!(gradients, 1, "the bar's gradient is not the one mesh it is");
        assert!(
            lines.contains(&STATUS_DIVIDER) && lines.contains(&STATUS_BOTTOM_BORDER),
            "the bar lost one of its two rules: {lines:?}"
        );
    }

    #[test]
    fn the_corner_button_is_a_measured_keycap_within_the_bar() {
        // Sized here and not at the call site: the keycap leaves the bar's
        // gradient above and below it, fits its glyph, and the gradient's
        // fall still reads across its height.
        const {
            assert!(
                STATUS_BAR_BUTTON_HEIGHT < STATUS_BAR_HEIGHT,
                "a corner button must leave the bar's face above and below it"
            )
        };
        const {
            assert!(
                STATUS_BAR_BUTTON_GLYPH_SIZE <= STATUS_BAR_BUTTON_HEIGHT,
                "a glyph taller than its keycap would be clipped by it"
            )
        };
        const {
            assert!(
                STATUS_BAR_BUTTON_WIDTH > STATUS_BAR_BUTTON_HEIGHT,
                "the measured bevel is wider than tall — a keycap, not a square"
            )
        };
    }

    /// The focus ring is the only sign the keyboard is on something, so it has
    /// to clear the 3:1 an indicator owes the surface it sits on. It does that
    /// with two bands because no single colour clears it on all six: the
    /// accent band shows on the dark panel, the hairline shows on the light
    /// surfaces where the accent sinks in. The test asks of each surface only
    /// that ONE band stands out from it.
    #[test]
    fn the_focus_ring_shows_on_every_surface_it_lands_on() {
        let light_panel = Visuals::light().panel_fill;
        let dark_panel = Visuals::dark().panel_fill;

        for surface in [
            SIDEBAR_FILL,
            STATUS_BAR_FILL,
            CENTRE_BOX_FILL,
            PAGE_FILL,
            light_panel,
            dark_panel,
        ] {
            let best = contrast(FOCUS_RING, surface).max(contrast(FOCUS_RING_EDGE, surface));
            assert!(
                best >= 3.0,
                "the focus ring is invisible on {surface:?}: its better band only reaches {best:.2}:1"
            );
        }
    }

    /// The ring belongs to the control: both bands are painted inwards from
    /// the rect, so a row drawn through a clipped painter cannot shave it.
    #[test]
    fn the_focus_ring_paints_its_two_bands_inside_the_rect() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(120.0, 24.0));
        let ctx = Context::default();

        let mut bands = Vec::new();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |ui| focus_ring(ui.painter(), rect),
        );
        for clipped in &output.shapes {
            if let egui::Shape::Rect(boxed) = &clipped.shape {
                bands.push((boxed.rect, boxed.stroke.color, boxed.stroke.width));
            }
        }
        output.textures_delta.clear();

        assert!(
            bands.contains(&(rect, FOCUS_RING_EDGE, FOCUS_RING_EDGE_WIDTH)),
            "no hairline band: {bands:?}"
        );
        assert!(
            bands.contains(&(
                rect.shrink(FOCUS_RING_EDGE_WIDTH),
                FOCUS_RING,
                FOCUS_RING_WIDTH
            )),
            "no accent band just inside it: {bands:?}"
        );
        for (band, _, _) in &bands {
            assert!(
                rect.contains_rect(*band),
                "a band painted outside the control it marks: {band:?} vs {rect:?}"
            );
        }
    }

    /// Everything one frame of [`candy_progress`] painted, sorted into the
    /// shapes the assertions read: the groove's rectangle, the gloss's mesh,
    /// and the reflection's line.
    fn paint_candy(
        fraction: Option<f32>,
    ) -> (Rect, Vec<Rect>, Vec<egui::epaint::Mesh>, Vec<Color32>) {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(200.0, CANDY_BAR_HEIGHT));
        let ctx = Context::default();

        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 200.0))),
                ..Default::default()
            },
            |ui| candy_progress(ui, rect, fraction),
        );

        let mut grooves = Vec::new();
        let mut glosses = Vec::new();
        let mut reflections = Vec::new();
        for clipped in &output.shapes {
            match &clipped.shape {
                egui::Shape::Rect(boxed) if boxed.fill == CANDY_GROOVE_FILL => {
                    grooves.push(boxed.rect);
                }
                egui::Shape::Mesh(mesh) => glosses.push((**mesh).clone()),
                egui::Shape::LineSegment { stroke, .. } => reflections.push(stroke.color),
                _ => {}
            }
        }
        output.textures_delta.clear();

        (rect, grooves, glosses, reflections)
    }

    /// The groove is the bar's promise and it holds whatever the job's state:
    /// rimmed in its own darker shade, drawn even when there is nothing to
    /// fill it with — and with nothing to fill it, nothing is painted in it.
    #[test]
    fn the_candy_bar_carries_its_groove_whatever_the_fraction() {
        for fraction in [Some(0.0), None] {
            let (rect, grooves, glosses, _) = paint_candy(fraction);

            assert!(
                grooves.contains(&rect),
                "no groove the width of the bar: {grooves:?}"
            );

            if fraction == Some(0.0) {
                assert!(
                    glosses.is_empty(),
                    "an empty job paints no gloss: {glosses:?}"
                );
            }
        }

        // And the rim is its own hairline, drawn over the fill so a gloss run
        // to the ends stays inside the rounded groove.
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(200.0, CANDY_BAR_HEIGHT));
        let ctx = Context::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 200.0))),
                ..Default::default()
            },
            |ui| candy_progress(ui, rect, Some(1.0)),
        );
        let rims = output
            .shapes
            .iter()
            .filter(|clipped| {
                matches!(&clipped.shape, egui::Shape::Rect(boxed)
                    if boxed.stroke.color == CANDY_GROOVE_EDGE && boxed.rect == rect)
            })
            .count();
        output.textures_delta.clear();

        assert!(
            rims >= 2,
            "the rim should be drawn under the gloss and again over it, got {rims}"
        );
    }

    /// A known total fills the groove from the left with the gradient — pale
    /// at the top, the accent's own blue at the bottom — and the proportion
    /// drawn is the proportion reported, not a decoration.
    #[test]
    fn the_candy_bar_fills_its_groove_in_proportion_and_top_down() {
        let (rect, _, glosses, reflections) = paint_candy(Some(0.5));

        let gloss = glosses.first().expect("a half-done job paints its fill");
        let bounds = gloss.calc_bounds();
        let inner = rect.shrink(1.0);

        assert!(
            (bounds.width() - inner.width() * 0.5).abs() <= 1.0,
            "half done drew {:?} of a {:?} groove",
            bounds.width(),
            inner.width()
        );
        assert!(
            inner.contains_rect(bounds),
            "the fill ran outside the groove: {bounds:?} in {inner:?}"
        );

        let first = gloss
            .vertices
            .first()
            .expect("a gradient is at least one vertex");
        let last = gloss
            .vertices
            .last()
            .expect("a gradient is at least two vertices");
        assert_eq!(first.color, CANDY_TOP, "the fill's top is not the pale end");
        assert_eq!(
            last.color, CANDY_BOTTOM,
            "the fill's bottom is not the accent end"
        );

        assert!(
            reflections.contains(&CANDY_HIGHLIGHT),
            "no reflection on a fill this wide: {reflections:?}"
        );
    }

    /// While the walk is still counting, the bar says "working" and nothing
    /// more: one short stripe inside the groove, never a full band — a full
    /// band would read as finished.
    #[test]
    fn the_candy_bar_slides_a_stripe_while_the_total_is_unknown() {
        let (rect, _, glosses, _) = paint_candy(None);

        let stripe = glosses.first().expect("an unknown job still shows work");
        let bounds = stripe.calc_bounds();
        let inner = rect.shrink(1.0);

        assert!(
            inner.contains_rect(bounds),
            "the stripe left the groove: {bounds:?} in {inner:?}"
        );
        assert!(
            bounds.width() < inner.width() / 2.0,
            "a stripe filling {}/{:?} of the groove is not a stripe",
            bounds.width(),
            inner.width()
        );
    }

    /// The stripe is a clock, not a counter: it has to be somewhere else when
    /// the clock has moved on, or the barber pole would hang and the sheet
    /// would look finished while it worked.
    #[test]
    fn the_unknown_stripe_moves_with_the_clock() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(200.0, CANDY_BAR_HEIGHT));

        let at = |time: f64| {
            let ctx = Context::default();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 200.0))),
                    time: Some(time),
                    ..Default::default()
                },
                |ui| candy_progress(ui, rect, None),
            );
            let bounds = output
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    egui::Shape::Mesh(mesh) => Some(mesh.calc_bounds()),
                    _ => None,
                })
                .expect("the stripe is painted");
            output.textures_delta.clear();
            bounds
        };

        let start = at(0.01).max.x;
        let later = at(0.35).max.x;

        assert!(
            (start - later).abs() > 0.5,
            "the stripe sat still between two frames: {start} then {later}"
        );
    }

    /// The centre box's surface is a pale field with the one rim the reference
    /// measured around it. The raised-panel gloss the box used to wear is gone:
    /// the measured disc panel is a flat pale, and it is the controls inside it
    /// — the seek rail, the queue button — that carry the glosses, not the box.
    #[test]
    fn the_centre_box_surface_paints_the_measured_pale_and_rim() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(304.0, 64.0));
        let ctx = Context::default();

        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(rect.expand(8.0)),
                ..Default::default()
            },
            |ui| centre_box_surface(ui.painter(), rect),
        );

        let mut fills = Vec::new();
        let mut rims = Vec::new();
        for clipped in &output.shapes {
            if let egui::Shape::Rect(boxed) = &clipped.shape {
                fills.push((boxed.rect, boxed.fill));
                rims.push((boxed.rect, boxed.stroke.color));
            }
        }
        output.textures_delta.clear();

        assert!(
            fills.contains(&(rect, CENTRE_BOX_FILL)),
            "the box's fill is not painted, or not first-class: {fills:?}"
        );
        assert!(
            rims.contains(&(rect, CENTRE_BOX_EDGE)),
            "no rim on the box's own edge: {rims:?}"
        );
        // The inner gloss line must not have come back. Its colour being all
        // but identical to the fill is why it was never seen, and the measured
        // panel does without it: nothing on the surface is stroked in the gloss.
        assert!(
            !rims.iter().any(|&(_r, c)| c == CENTRE_BOX_GLOSS),
            "the box stroked its inner gloss, which the measured panel has no line for: {rims:?}"
        );
    }

    /// Naming a control is the cheap half of accessibility and must never be
    /// the reason a frame dies: with no AccessKit active — every in-process
    /// test, and every session without an assistive technology attached —
    /// egui hands back `None` and the name is simply not set.
    #[test]
    fn naming_a_control_without_accesskit_active_is_harmless() {
        let ctx = Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let response = ui.button("⏵");
            accessible_name(ui, &response, "Play or pause");
        });
        output.textures_delta.clear();
    }
}

#[cfg(test)]
mod reference_geometry_tests {
    use super::*;
    #[test]
    fn measured_geometry_is_pinned() {
        assert_eq!(MENU_BAND_HEIGHT, 16.0);
        assert_eq!(TOOLBAR_HEIGHT, 60.0);
        assert_eq!(TRACK_ROW_HEIGHT, 17.0);
        assert_eq!(CELL_PADDING, 6.0);
        assert_eq!(STATUS_BAR_HEIGHT, 28.0);
        assert_eq!(SIDEBAR_WIDTH, 204.0);
        assert_eq!(WINDOW_WIDTH, 1180.0);
        assert_eq!(WINDOW_HEIGHT, 780.0);
    }
    #[test]
    fn measured_palette_is_pinned() {
        assert_eq!(DARK_RULE, Color32::from_rgb(64, 64, 64));
        assert_eq!(TABLE_HEADER_TOP, Color32::from_rgb(219, 219, 219));
        assert_eq!(TABLE_HEADER_BOTTOM, Color32::from_rgb(187, 187, 187));
        assert_eq!(TABLE_STRIPE, Color32::from_rgb(241, 245, 250));
        assert_eq!(FLAT_SELECTION, Color32::from_rgb(93, 148, 214));
        assert_eq!(CENTRE_DISC_TOP, Color32::from_rgb(254, 255, 232));
        assert_eq!(CENTRE_DISC_BOTTOM, Color32::from_rgb(220, 224, 197));
    }
}
