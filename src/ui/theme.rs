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
/// Measured off the reference: its source list is 203 pixels across, the
/// last of them the dark rule, and the page takes the pixel after that.
pub const SIDEBAR_WIDTH: f32 = 203.0;

/// Height of the playback bar across the top of the window, in points.
pub const PLAYBACK_BAR_HEIGHT: f32 = 92.0;

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
pub const STATUS_BAR_HEIGHT: f32 = 30.0;

/// Side of a corner button on the status bar, in points.
///
/// Smaller than [`STATUS_BAR_HEIGHT`] so a hair of the gray shows above and
/// below it: the button then reads as a control sitting on the bar rather than
/// as a block filling its corner.
///
/// Both corner buttons are this size, so the bar's two ends read as a pair
/// rather than as two independently sized controls.
pub const STATUS_BAR_BUTTON_SIZE: f32 = 24.0;

/// Size of the glyph drawn in a corner button, in points.
///
/// A glyph draws a box taller than its own size -- some 1.16 times it, so this
/// stands the glyph at 25.5 of the bar's 29 points, most of the bar's height and
/// as much as it has room for: the bar is only as tall as what it holds, and at
/// 23 the glyph pushes the bar's fill past the foot of the window. At 19 it read
/// as a speck in the corner rather than as the one control at that end of the
/// bar.
pub const STATUS_BAR_BUTTON_GLYPH_SIZE: f32 = 22.0;

/// Gap between a corner button and its end of the status bar, in points.
pub const STATUS_BAR_BUTTON_INSET: f32 = 3.0;

/// Gap between two buttons sharing one end of the status bar, in points.
///
/// The left end holds three — the `+` that names a playlist, the shuffle
/// switch and the repeat switch — which read as separate controls rather than
/// as one wide button because of this hair of the bar showing between them.
pub const STATUS_BAR_BUTTON_GAP: f32 = 2.0;

/// Fill of a status bar corner button while the pointer is over it, or held
/// down.
///
/// The bar paints its own gray, so the themes' hover fill — near-black in the
/// dark theme — would put the bar's dark ink on a dark square. This is the
/// bar's gray a shade deeper, which the ink still reads on.
pub const STATUS_BAR_HOVER: Color32 = Color32::from_rgb(158, 160, 166);

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
/// flat bar across the panel's whole width, deepening from a pale slate at
/// its top to a full shade at its bottom; the two ends here are the reference's
/// own measured pixels, top (161, 176, 207) and bottom (113, 133, 171). The
/// white label sits across the whole ramp, and the ramp is wide enough that
/// the label reads — see the tests below for the contrast each end gives it.
pub const SIDEBAR_SELECTED_TOP: Color32 = Color32::from_rgb(161, 176, 207);

/// The bottom end of the selected row's bar.
pub const SIDEBAR_SELECTED_BOTTOM: Color32 = Color32::from_rgb(113, 133, 171);

/// The single darker line the reference lays along the bar's top edge —
/// (145, 160, 192), a shade below the gradient's own first row, so the bar
/// sits in the panel instead of floating over it.
pub const SIDEBAR_SELECTED_EDGE: Color32 = Color32::from_rgb(145, 160, 192);

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
/// A pale cream, so the box reads as a window onto the track rather than as
/// another panel of the bar's gray. Like the bar's own fill it is deliberately
/// not taken from the active theme: the box is a cream surface in the light
/// theme and in the dark one.
pub const CENTRE_BOX_FILL: Color32 = Color32::from_rgb(243, 234, 205);

/// Label colour inside the centre box.
///
/// The box is a light surface, so it carries its own dark ink; the theme's
/// foreground is near-white in the dark theme and would vanish on the cream.
pub const CENTRE_BOX_INK: Color32 = Color32::from_rgb(38, 34, 24);

/// Muted [`CENTRE_BOX_INK`], for the line under the title.
pub const CENTRE_BOX_INK_MUTED: Color32 = Color32::from_rgb(92, 84, 62);

/// Fill of a control inside the centre box, and of the empty cover slot.
///
/// A step of the box's own cream rather than the bar's gray, so a control on
/// the box stays part of the box.
pub const CENTRE_BOX_CONTROL: Color32 = Color32::from_rgb(226, 214, 180);

/// The centre box's rim: a warm dark cream, darker than the bar's gray and
/// far darker than the box's own fill.
///
/// The player drew everything as an object — no edge-less fields — and on a
/// gray bar a cream field with no edge dissolves into it. One hairline is
/// what makes the box a thing sitting ON the bar rather than a hole in it.
pub const CENTRE_BOX_EDGE: Color32 = Color32::from_rgb(150, 140, 114);

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
    painter.rect_filled(rect, radius, CENTRE_BOX_FILL);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, CENTRE_BOX_EDGE),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(1.0),
        CornerRadius::same(CENTRE_BOX_RADIUS - 1),
        Stroke::new(1.0, CENTRE_BOX_GLOSS),
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
pub const STATUS_BAR_FILL: Color32 = Color32::from_rgb(176, 178, 182);

/// Label colour of the status bar.
///
/// The bar is a light surface in both themes, so it carries its own dark ink:
/// the theme's foreground is near-white in the dark theme and would vanish on
/// [`STATUS_BAR_FILL`].
pub const STATUS_BAR_INK: Color32 = Color32::from_rgb(32, 34, 38);

/// Muted [`STATUS_BAR_INK`], for the status bar's single line.
pub const STATUS_BAR_INK_MUTED: Color32 = Color32::from_rgb(58, 60, 66);

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
        .inner_margin(Margin::symmetric(16, 10))
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
/// Half the box's height, which is what makes it the pill the bar is built
/// around rather than a rectangle sitting on it.
pub const CENTRE_BOX_RADIUS: u8 = 26;

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

/// The status bar's ink, and the fill a button on the bar rests in.
///
/// A resting button takes the bar's own fill, so the `+` reads as the bar
/// rather than as a pale tile sitting on it: the bar is a surface, and a
/// control only shows itself when the pointer is over it or its state is on.
pub const STATUS_BAR_BUTTON_FILL: Color32 = STATUS_BAR_FILL;

/// Point a style at the status bar's own palette.
///
/// Call this inside a `Ui::scope` around the bar's body. Like the sidebar and
/// the playback bar, the status bar paints a fill of its own in every theme, so
/// its line takes the bar's ink rather than the theme's foreground.
pub fn apply_status_bar_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, STATUS_BAR_INK, STATUS_BAR_INK_MUTED);

    // A button on the bar is painted with `weak_bg_fill`, so in any theme it
    // rests in a fill of its own -- the light theme's `#E6E6E6` on the bar's
    // gray `#B0B2B6`, a pale tile that is visible at rest and that shifts when
    // the theme changes. It takes the bar's fill instead, and shows itself only
    // where it has something to say: under the pointer, or while it is on.
    visuals.widgets.inactive.weak_bg_fill = STATUS_BAR_BUTTON_FILL;
    visuals.widgets.noninteractive.weak_bg_fill = STATUS_BAR_BUTTON_FILL;

    // `RichText::strong()` needs no colour of its own here: egui resolves it
    // through `Visuals::strong_text_color()`, which is `widgets.active`'s text
    // colour, and `ink_surface` above has just set that to the bar's ink. So
    // the bar's glyphs can say `strong()` and mean it, without a second place
    // to keep in step.

    // The hover fill is the bar's own step-darker shade rather than the themes':
    // theirs is near-black in the dark theme, which would put the bar's dark ink
    // on a dark square.
    for widget in [&mut visuals.widgets.hovered, &mut visuals.widgets.active] {
        widget.bg_fill = STATUS_BAR_HOVER;
        widget.weak_bg_fill = STATUS_BAR_HOVER;
    }
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

    /// The centre box is a pale cream, and its ink reads on it in both themes:
    /// the box paints its own fill whatever the theme, so the theme's
    /// foreground — near-white in the dark theme — would vanish on it.
    #[test]
    fn the_centre_box_is_cream_and_its_ink_reads_on_it() {
        let [r, g, b, a] = CENTRE_BOX_FILL.to_array();
        assert_eq!(a, 255, "the box's surface must be opaque");
        assert!(
            r > g && g > b,
            "cream leads with red, then green, then blue"
        );
        assert!(
            luminance(CENTRE_BOX_FILL) > 0.7,
            "the box has to stay a pale cream, not a tan: {CENTRE_BOX_FILL:?}"
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
        for ink in [STATUS_BAR_INK, STATUS_BAR_INK_MUTED] {
            assert!(
                contrast(ink, STATUS_BAR_FILL) >= 4.5,
                "bar ink {ink:?} is too faint on {STATUS_BAR_FILL:?}"
            );
        }
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

    #[test]
    fn the_gear_hovers_in_the_bars_own_shade() {
        let mut visuals = Visuals::dark();
        apply_status_bar_visuals(&mut visuals);

        for widget in [&visuals.widgets.hovered, &visuals.widgets.active] {
            assert_eq!(widget.bg_fill, STATUS_BAR_HOVER);
            assert_eq!(widget.weak_bg_fill, STATUS_BAR_HOVER);
        }

        // The hover frame has to be a step away from the bar, or the pointer
        // gives no sign it is over the control.
        assert!(
            contrast(STATUS_BAR_HOVER, STATUS_BAR_FILL) > 1.2,
            "a hovered corner button must show: {STATUS_BAR_HOVER:?} on {STATUS_BAR_FILL:?}"
        );
        // And the bar's ink must stay readable on it, which is why the hover
        // fill is a darker gray rather than the themes' own.
        assert!(
            contrast(STATUS_BAR_INK, STATUS_BAR_HOVER) >= 4.5,
            "the bar's ink is too faint on {STATUS_BAR_HOVER:?}"
        );
    }

    /// A bar button at rest is the bar: the `+` is the bar's own gray, not a
    /// tile sitting on it in whatever fill the active theme happens to give an
    /// inactive button.
    #[test]
    fn a_button_on_the_bar_rests_in_the_bars_own_fill_in_both_themes() {
        for theme in [Visuals::light(), Visuals::dark()] {
            let stock = theme.widgets.inactive.weak_bg_fill;

            let mut visuals = theme;
            apply_status_bar_visuals(&mut visuals);

            assert_eq!(
                visuals.widgets.inactive.weak_bg_fill, STATUS_BAR_FILL,
                "a resting bar button is painted in the bar's own fill"
            );
            assert_eq!(
                visuals.widgets.noninteractive.weak_bg_fill, STATUS_BAR_FILL,
                "and so is one egui draws as non-interactive"
            );
            assert_ne!(
                visuals.widgets.inactive.weak_bg_fill, stock,
                "the button is still taking the theme's own resting fill"
            );
        }

        // The ink has to stay readable on that fill, or the glyph vanishes into
        // the bar it was just made to match.
        assert!(
            contrast(STATUS_BAR_INK, STATUS_BAR_BUTTON_FILL) >= 4.5,
            "the bar's ink is too faint on its own fill"
        );

        // The hover still has to show: matching at rest must not have matched
        // the hover too, or pointing at the control does nothing.
        assert_ne!(
            STATUS_BAR_HOVER, STATUS_BAR_BUTTON_FILL,
            "the hover was flattened into the resting fill"
        );
        assert!(
            contrast(STATUS_BAR_HOVER, STATUS_BAR_BUTTON_FILL) > 1.2,
            "a hovered bar button must still show against its resting fill"
        );
    }

    #[test]
    fn the_corner_button_glyph_fills_its_button_within_the_bar() {
        // Sized here and not at the call site: the button has to leave gray
        // above and below it on the bar, and the glyph has to fit the button.
        const {
            assert!(
                STATUS_BAR_BUTTON_SIZE < STATUS_BAR_HEIGHT,
                "a corner button must leave gray above and below it on the bar"
            )
        };
        const {
            assert!(
                STATUS_BAR_BUTTON_GLYPH_SIZE <= STATUS_BAR_BUTTON_SIZE,
                "a glyph wider than its button would be clipped by it"
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

    /// The centre box's surface is three bands, and all three are painted:
    /// the fill that every caller looks the box up by, the rim that stands
    /// it off the bar, and the lighter line just inside the rim that is the
    /// raised edge catching the light.
    #[test]
    fn the_centre_box_surface_paints_its_three_bands() {
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
        assert!(
            rims.contains(&(rect.shrink(1.0), CENTRE_BOX_GLOSS)),
            "no light along the inner edge: {rims:?}"
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
