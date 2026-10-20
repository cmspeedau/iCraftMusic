//! The playback bar across the top of the window.
//!
//! Everything here mirrors the audio layer: what is playing, how far along it
//! is, and the controls that drive it. The bar is always drawn, even when no
//! output device could be opened — the queue is still real work, so the
//! controls stay live and the failure is shown in place of the track details.
//!
//! The centre box is the one part whose contents come and go. Its cream pill is
//! always drawn, but the "now playing" widget inside it — the cover, the
//! track's title and state, the seek row and the queue button — is painted only
//! while there is a track on the bar: with nothing playing the box keeps its
//! surface and holds nothing rather than saying "Nothing playing" in the middle
//! of the cream. The transport and the Library's own controls stay where they
//! are.
//!
//! The bar has three parts. The transport is at the far LEFT, where the pointer
//! arrives first: previous · play · next, then the speaker and the volume
//! slider, all in white. The buttons sit against the bar's left edge and the
//! volume hangs in the middle of the room between them and the box — the two
//! controls that start the music are where the pointer is, and the one that
//! sets how loud it is is centred on the gap it belongs to rather than pushed
//! up against the buttons. The middle is the centre box — a pale cream pill
//! centred on the window, holding the album cover at its left, the track's
//! title and state beside it, and the queue button at its right. The seek row
//! runs under both.
//!
//! The Library's search field rides at the bar's right end. It used to sit on
//! the Library's own heading, where the query filtered the page it was typed
//! on; the bar is the one line that is on screen in every section, so the field
//! and the query it holds are now reachable from all of them, and the heading
//! line goes back to naming the section.
//!
//! The Library's three pages — Tracks, Albums and Artists — are buttons here
//! too, centred on the room between the centre box and the search field: they
//! left the sidebar, where they were three indented rows under the Library
//! entry with nothing to tell them apart, and each carries the icon the request
//! named for it — a musical note, a compact disc, and a person's head and
//! shoulders. Like the field, they are drawn only while the Library is open:
//! they are the Library's own pages, and the section is what they page through.
//!
//! Shuffle and repeat are not here. Both moved down to the status bar, where
//! they sit with the controls that are about the session rather than about the
//! track: on this bar each took a button's width out of the transport row to
//! say a state that outlives the track.

use egui::{
    Align, Button, Color32, CornerRadius, Layout, Panel, Pos2, Rect, RichText, Sense, Slider,
    Stroke, StrokeKind, TextEdit, UiBuilder, Vec2,
};
use std::path::Path;

use crate::app::{IndexingStatus, MusicCraftApp, UpdateStatus, format_duration};
use crate::library::browse::BrowseKind;
use crate::ui::navigation::NavSection;
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

/// Width of one of the transport's three buttons, in points.
///
/// Named so the row the buttons are placed in is their own sum rather than a
/// number that has to be kept in step with `controls` by hand.
const TRANSPORT_BUTTON_WIDTH: f32 = 38.0;

/// Height of one of the transport's three buttons, in points.
const TRANSPORT_BUTTON_HEIGHT: f32 = 28.0;

/// Length of the volume slider, in points.
const VOLUME_SLIDER_WIDTH: f32 = 90.0;

/// Width of the box the volume speaker is drawn in, in points.
///
/// Fixed, so the group the volume is centred by is a sum of constants rather
/// than whatever the glyph happened to measure: the speaker is narrower than
/// its box and centres inside it, and a label that reported the glyph's own 11
/// points would leave the rail half the difference off the centre it was put
/// on.
const VOLUME_GLYPH_WIDTH: f32 = 16.0;

/// Side of the queue button in the centre box, in points.
///
/// Square, and as tall as the box's contents: it reads as one of the bar's
/// controls rather than as the glyph it draws.
const QUEUE_BUTTON_SIZE: f32 = 28.0;

/// The Cancel button on the Indexing bar's line, in points.
///
/// Fixed so the bar takes what is left of the row's width rather than trading
/// width with a label that changes every file: the groove never jitters.
const CANCEL_BUTTON_WIDTH: f32 = 54.0;

/// Size the queue's burger glyph is drawn at, in points.
///
/// Set here rather than taken from a text style because the burger is a narrow
/// glyph: at the bar's Button size (13) it inked about 11×9 points, and at the
/// smaller size it was drawn at before that, 8×7 — a speck beside the
/// transport's glyphs. Its button is 28 across and the burger is thin, so there
/// is room to draw it well larger than the line it sits in; this is the size
/// that fills the button without the glyph touching its edges.
const QUEUE_GLYPH_SIZE: f32 = 20.0;

/// Room a page button is given, in points.
///
/// Square, and the size the transport's buttons are drawn at, so the row of
/// pages reads as part of the bar's own controls rather than as a second kind.
const PAGE_BUTTON_SIZE: f32 = 28.0;

/// Width of a page button's ring, in points.
///
/// The same on all three at the user's request, so the closed pages are framed
/// like the open one and only the accent says which page you are on. It is also
/// the width the Clear button's label is set off by, so the button's reserved
/// room in `SEARCH_CLEAR_WIDTH` is not disturbed by it.
const PAGE_BUTTON_EDGE: f32 = 2.0;

/// The music note that stands for Tracks, on the Library's first page button.
///
/// A note is what a track is; the request named it, and the bundled fonts carry
/// it at the size it is drawn (the quarter note and the beamed pair ink the
/// same width, so either would do — this is the plainer of the two).
const TRACKS_GLYPH: &str = "♪";

/// The jewel case that stands for Albums.
const ALBUMS_GLYPH: &str = "💿";

/// The head and shoulders that stand for Artists.
const ARTISTS_GLYPH: &str = "👤";

/// The Library's three pages, in the order the buttons are drawn: each page
/// with the icon that tells it apart and the words the pointer reads.
///
/// One list rather than three hand-placed buttons, so a page cannot arrive
/// without its icon, its hover text and a place in the row.
const PAGES: [(BrowseKind, &str, &str); 3] = [
    (BrowseKind::Tracks, TRACKS_GLYPH, "Tracks"),
    (BrowseKind::Albums, ALBUMS_GLYPH, "Albums"),
    (BrowseKind::Artists, ARTISTS_GLYPH, "Artists"),
];

/// Points between two of the Library's page buttons.
///
/// Tighter than the bar's own item spacing: the three carry no words, so a
/// 10-point gap between 28-point squares read as three separate controls rather
/// than as the one row they are.
const PAGE_BUTTON_GAP: f32 = 4.0;

/// Length of the search field at the bar's right end, in points.
///
/// The width the query was typed into when the field lived on the Library's
/// heading, so the same query reads the same way on the bar.
const SEARCH_WIDTH: f32 = 220.0;

/// Fill of the search field and of the buttons beside it, in points.
///
/// White, at the user's request: on a bar whose own fill is a mid gray, the
/// field and the buttons are the one surface that is not, so the query reads as
/// a field rather than as another control tile. Like the transport's glyphs
/// were, it is a surface of its own, so everything drawn on it — the query, its
/// hint text, the caret and the Clear button's label — takes the bar's dark ink
/// and reads on it.
const SEARCH_FILL: Color32 = Color32::WHITE;

/// Height of the search field, in points.
const SEARCH_HEIGHT: f32 = 22.0;

/// Room the Clear button takes, to the left of the field, in points.
///
/// The button's width is its text's; this is the room reserved for it so the
/// field it belongs to stays against the bar's right edge whether or not a
/// query is on.
///
/// Kept small on purpose. The room is set off the field's leading edge, and the
/// page group is centred on the room between the centre box and that edge, so
/// the room has to fit in the gap the group's trailing end leaves over. A wide
/// room would reach back under the last page button once the window is narrow
/// enough for the field to meet the middle of the bar. 44 holds the label at the
/// bar's text size with padding to spare, and leaves that gap the larger of the
/// two on the narrowest window the app opens at.
const SEARCH_CLEAR_WIDTH: f32 = 44.0;

/// Points between the search field and what is beside it — the Clear button on
/// its left, and the centre box beyond that when the window is too narrow for
/// both.
const SEARCH_GAP: f32 = 8.0;

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
                // of its own, and it draws the seek row inside itself. The
                // search field is placed at the bar's right end, off the same
                // line, so the three are on one line across the bar.
                // The centre box's pill is always drawn — it is the widget's
                // own surface and keeps the bar's middle — but with nothing
                // playing [`centre_box`] leaves everything it holds off the
                // cream. The transport and the Library's own controls are drawn
                // whatever the bar holds.
                transport_row(app, ui);
                centre_box(app, ui);
                search_row(app, ui);
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
///
/// The two halves of the row are placed in rectangles of their own rather than
/// run together in one layout. The buttons are a fixed row at the bar's left
/// edge, and the volume is centred on the room between that row and the box
/// beside it: what the user asked for is the volume halfway along the empty bar
/// rather than stacked against the buttons, and a layout that simply advanced a
/// cursor would leave it wherever the buttons stopped.
fn transport_row(app: &mut MusicCraftApp, ui: &mut egui::Ui) -> Rect {
    // The row is placed by its own rectangle so its MIDDLE is the centre box's,
    // which is the bar's own middle: a horizontal layout would put the row's
    // TOP where the cursor is and the controls would ride high in the bar.
    let row = Rect::from_center_size(
        ui.max_rect().center(),
        Vec2::new(ui.max_rect().width(), COVER_SIZE),
    );

    let buttons = buttons_rect(row);
    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_transport")
            .max_rect(buttons)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| controls(app, ui),
    );

    let volume_row = volume_rect(row, centre_box_rect(ui.max_rect()));
    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_volume")
            .max_rect(volume_row)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| volume(app, ui),
    );

    row
}

/// Where the three transport buttons sit: a fixed row against the bar's left
/// edge, on the centre box's own line.
///
/// Wide enough for the three buttons AND the two gaps between them, so the
/// rectangle is what the buttons actually fill rather than a width the third
/// one hangs out of.
fn buttons_rect(row: Rect) -> Rect {
    let width = 3.0 * TRANSPORT_BUTTON_WIDTH + 2.0 * TRANSPORT_GAP;
    Rect::from_min_size(
        Pos2::new(row.left(), row.center().y - TRANSPORT_BUTTON_HEIGHT / 2.0),
        Vec2::new(width, TRANSPORT_BUTTON_HEIGHT),
    )
}

/// Where the volume group sits: centred on the empty bar between the transport
/// buttons and the centre box.
///
/// The two are the things it is between — the box is what the volume belongs to
/// and the buttons are what it follows — so the group's own centre is the
/// midpoint of the room they leave, and its ends sit off from them by whatever
/// that room happens to be rather than by a gap of its own.
fn volume_rect(row: Rect, pill: Rect) -> Rect {
    let button_edge = row.left() + 3.0 * TRANSPORT_BUTTON_WIDTH + 2.0 * TRANSPORT_GAP;
    let left = f32::midpoint(button_edge, pill.left()) - volume_width() / 2.0;

    Rect::from_min_size(
        Pos2::new(left, row.center().y - TRANSPORT_BUTTON_HEIGHT / 2.0),
        Vec2::new(volume_width(), TRANSPORT_BUTTON_HEIGHT),
    )
}

/// How wide the speaker and the slider are together, in points.
fn volume_width() -> f32 {
    VOLUME_GLYPH_WIDTH + TRANSPORT_GAP + VOLUME_SLIDER_WIDTH
}

/// The Library's own controls, at the bar's right end: the three page buttons
/// and the search field with the Clear button beside it.
///
/// They sit on the same line as the transport and the box — placed by their own
/// rectangles off the bar's middle, the way the volume is — rather than in a row
/// of the bar's layout: the field's right edge is the bar's own inner edge,
/// whatever the window is, and nothing else on the bar can move it.
///
/// All of it is drawn only while the Library is the open section. The pages are
/// the Library's, and the query filters the Library, so the bar offers them
/// there and nowhere else; a query already typed stays in the application and
/// is waiting in the field when the section is opened again.
fn search_row(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    if app.section() != NavSection::Library {
        return;
    }

    let bar = ui.max_rect();
    let pill = centre_box_rect(bar);
    // The field is sized first and the pages take the room that is left over:
    // at the app's narrowest window the bar has none to spare, and a field that
    // took its full width anyway would push the pages back over the box.
    let field = search_rect(bar, pill);
    // The Clear button's room, which the field leaves to its left whether or
    // not a query is on: the way back out of a filter belongs to the field.
    let clear = Rect::from_min_size(
        Pos2::new(field.left() - SEARCH_GAP - SEARCH_CLEAR_WIDTH, field.top()),
        Vec2::new(SEARCH_CLEAR_WIDTH, field.height()),
    );
    let pages = page_buttons_rect(bar, pill, field, clear);

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_pages")
            .max_rect(pages)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            // The group is placed from its own width, and the layout's default
            // spacing would put the buttons further apart than that width
            // accounts for — the last one then hangs out of the room the group
            // was centred in. The gap is the group's own.
            ui.spacing_mut().item_spacing.x = PAGE_BUTTON_GAP;
            page_buttons(app, ui);
        },
    );

    ui.scope_builder(
        UiBuilder::new()
            .id_salt("playback_search")
            .max_rect(field.union(clear)),
        |ui| search_field(app, ui, field, clear),
    );
}

/// Where the Library's three page buttons sit: centred on the room the centre
/// box and the search field leave between them.
///
/// The two are what the request said the buttons are between, so the group is
/// placed on the MIDPOINT of the room they leave — its ends sit off from them by
/// whatever that room happens to be, and the two gaps are equal by construction.
/// Placing it against the box instead would slide it about as the window
/// changed; a gap of its own would say a distance the request never named.
///
/// The room runs to the field's own leading edge — the white edge the user sees,
/// and the searchbar the buttons were asked to sit beside — rather than to the
/// Clear button's reserved room beyond it, which is part of the field's own strip
/// and would drag the group left of the middle of what is on screen.
///
/// The button's room is passed in as a bound to clear, since it is reserved
/// whether or not a query is on: the centre box and the Clear button are the two
/// things the group is wedged between, and its trailing end is pushed back along
/// the room rather than left sitting under the button. On a window wide enough
/// for the exact centre the bound is slack and nothing moves.
fn page_buttons_rect(bar: Rect, pill: Rect, field: Rect, clear: Rect) -> Rect {
    let room = f32::midpoint(pill.right(), field.left());
    let group = Rect::from_center_size(
        Pos2::new(room, bar.center().y),
        Vec2::new(page_buttons_width(), PAGE_BUTTON_SIZE),
    );

    // Written as the shift it is rather than as a clamp, so the group keeps its
    // width and only slides.
    let overlap = clear.left() - group.right();
    if overlap < 0.0 {
        return group.translate(Vec2::new(overlap, 0.0));
    }

    group
}

/// How wide the three page buttons are together, in points.
///
/// The sum of the three and the two gaps between them, so the rectangle is what
/// the buttons actually fill rather than a width the last one hangs out of.
fn page_buttons_width() -> f32 {
    3.0 * PAGE_BUTTON_SIZE + 2.0 * PAGE_BUTTON_GAP
}

/// One button per Library page, each with the icon that tells it apart.
///
/// All three are white at the user's request, like the transport's buttons and
/// the search field beside them, so the row reads as one control strip on the
/// bar rather than as three wordless squares resting on it.
///
/// The fill is set on the button rather than left to the style, because
/// `Button::fill` is what overrides the state's fill in every state — and
/// `selected` cannot say which page is open the way it did before. On a white
/// button the open page shows itself with the accent instead: the accent ring
/// and an accent icon, against the grey ring and the bar's ink on the two that
/// are closed. That is the same accent the sidebar used to fill the selected
/// entry with, moved to the edge of a control that is no longer filled by the
/// theme.
///
/// The closed pages' icons take the bar's own ink — the same ink the
/// transport's glyphs take on their white buttons, since a white control on
/// this bar is drawn with dark ink: the bar's fill is light in every theme, and
/// the theme's button ink is near-white in the dark one.
fn page_buttons(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    for (kind, glyph, label) in PAGES {
        let selected = app.browse().kind() == kind;
        let size = Vec2::splat(PAGE_BUTTON_SIZE);

        // One weight for all three: the open page's mark is the accent ring and
        // the accent icon, not a heavier edge, so the closed two sit in the same
        // frame as the open one and the row reads as one set.
        let ink = if selected {
            theme::ACCENT
        } else {
            theme::PLAYBACK_BAR_INK
        };
        let edge = if selected {
            Stroke::new(PAGE_BUTTON_EDGE, theme::ACCENT)
        } else {
            Stroke::new(PAGE_BUTTON_EDGE, theme::PLAYBACK_BAR_CONTROL_EDGE)
        };

        let response = ui
            .add_sized(
                size,
                Button::new(RichText::new(glyph).color(ink))
                    .selected(selected)
                    .fill(theme::PLAYBACK_BAR_CONTROL)
                    .stroke(edge)
                    .frame(true),
            )
            .on_hover_text(label);
        // The PAGES table is the single source for what each button is called:
        // the pointer reads it as hover text, a screen reader reads it as the
        // node's name, and a keyboard user sees it under the ring.
        theme::accessible_name(ui, &response, label);
        theme::focus_ring_if_focused(ui, &response);

        if response.clicked() {
            app.browse_mut().set_kind(kind);
        }
    }
}

/// Where the search field sits: its right edge at the bar's right-hand edge.
///
/// Kept to `SEARCH_WIDTH` wherever the bar has them — the width the query was
/// typed into when the field lived on the Library's heading — and trimmed on a
/// narrow window, because the field is what gives way: the box is centred on the
/// window and holds the track, the pages are the navigation, and a narrower field
/// only elides its query.
///
/// The room it leaves is not just the page buttons'. The buttons are centred on
/// the middle of the room between the box and the field's leading edge, so half
/// of what the field keeps comes back to the right of that middle — and what the
/// field keeps also has to hold the Clear button's reserved room, set off its
/// leading edge. For the buttons' trailing end to clear that room, their width
/// and BOTH copies of the button's are what has to be left: `page_buttons_width`
/// once for the group itself, and `SEARCH_GAP + SEARCH_CLEAR_WIDTH` twice,
/// because one copy sits on each side of the group's centre. Without the second
/// copy the group is pushed back against the box on a narrow window and the
/// button's room reaches under its last square.
///
/// On a wide window the whole reservation is smaller than `SEARCH_WIDTH`, so
/// nothing is trimmed and the field is the width it has always been; the trim
/// only bites once the window is too narrow for the box, the buttons, the box's
/// room and the field to sit side by side — which is below the width the app
/// opens at, so the initial window shows the field at full width.
fn search_rect(bar: Rect, pill: Rect) -> Rect {
    let right = bar.right();
    let reserved = page_buttons_width() + 2.0 * (SEARCH_GAP + SEARCH_CLEAR_WIDTH);
    let width = (right - pill.right() - reserved).clamp(0.0, SEARCH_WIDTH);

    Rect::from_min_size(
        Pos2::new(right - width, bar.center().y - SEARCH_HEIGHT / 2.0),
        Vec2::new(width, SEARCH_HEIGHT),
    )
}

/// The query field itself, and the way back out of a query.
///
/// Both are placed in rectangles of their own rather than run through a layout:
/// the Clear button appears and disappears with the query, and the field must
/// not move when it does. The field is filled from its rectangle — the caller
/// reserved the button's room to the left of it — and that room is kept whether
/// or not the button is drawn, so the field's two edges are the same whatever
/// the query comes to.
fn search_field(app: &mut MusicCraftApp, ui: &mut egui::Ui, field: Rect, clear: Rect) {
    // The button beside the field is white like the field, at the user's
    // request, so its label is the bar's dark ink rather than the theme's
    // button ink — which is near-white in the dark theme and would vanish on it.
    let label = RichText::new("Clear").color(theme::PLAYBACK_BAR_INK);

    if app.browse().is_filtered()
        && ui
            .put(clear, Button::new(label).fill(SEARCH_FILL))
            .on_hover_text("Show the whole library again")
            .clicked()
    {
        app.browse_mut().set_query_text(String::new());
    }

    let response = ui.put(
        field,
        // The field and the buttons beside it are a WHITE surface, at the user's
        // request, so the query, its hint, the caret and the button's label all
        // take the bar's dark ink and read on it — well over 10:1, where the
        // theme's own text-edit fill is near-black in the dark theme and would
        // have put the bar's dark ink on a black tile.
        TextEdit::singleline(app.browse_mut().query_text_mut())
            .hint_text("Search…")
            .background_color(SEARCH_FILL),
    );
    // The hint reaches a screen reader as the field's placeholder, but the
    // words give it a name outright; the ring shows where Tab has reached on a
    // field whose only other sign of focus is a caret in empty white.
    theme::accessible_name(ui, &response, "Search");
    theme::focus_ring_if_focused(ui, &response);
    response.on_hover_text("Matches title, artist, album, genre and path");
}

/// The cream box on the window's centre: cover, track details, queue button,
/// and the seek row under them.
///
/// The cream pill is drawn whatever the bar holds — it is the widget's own
/// surface and keeps the bar's middle — but what sits IN it is drawn only while
/// there is something to say. With nothing playing and nothing being indexed
/// the box is a bare cream shape: no cover, no "Nothing playing" line, no seek
/// row and no queue button, just the surface the widget returns to.
///
/// Indexing takes the top row from the track while a scan or copy runs: the
/// bar's centre is the one place the user is guaranteed to be looking when
/// music is coming in, and the queue button holds its exact spot through the
/// swap. A track that is playing keeps its seek row underneath.
///
/// An update news takes the same row when no job is running: newer version,
/// arriving bytes, or the word that it is installed and wants a restart.
/// Indexing outranks it — a job the user started is louder news — and the ☰
/// holds its spot through that swap too.
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
    let pill = centre_box_rect(ui.max_rect());

    // The surface first and unconditionally: the pill is the widget's own
    // shape, so with nothing playing it is the whole of what is drawn — the box
    // keeps its place on the cream rather than closing when the queue is empty.
    // Under the contents, so the cover, the text and the rail sit on the cream
    // rather than the cream over them.
    theme::centre_box_surface(ui.painter(), pill);

    // Owned before anything draws, for the same reason the track's parts were
    // always owned first: the closures below read the app and then mutate it
    // (the queue, the Cancel button), and a borrow held across them conflicts.
    let indexing = app.indexing_status();
    // The check's silence is the track's chance: while a background check is
    // only wondering, the box says nothing of it and the track keeps its row.
    let update = app
        .update_status()
        .filter(|status| *status != UpdateStatus::Checking);
    let playing = app.now_playing().is_some();

    // The bare pill: nothing on the bar to say.
    if indexing.is_none() && update.is_none() && !playing {
        return;
    }

    let cover_row = top_row_rect(pill);
    let seek_row = seek_row_rect(pill);

    if let Some(status) = indexing {
        ui.scope_builder(
            UiBuilder::new()
                .id_salt("playback_centre_box")
                .max_rect(cover_row)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                theme::apply_centre_box_visuals(&mut ui.style_mut().visuals);
                ui.spacing_mut().item_spacing = Vec2::new(CENTRE_BOX_GAP, DETAILS_LINE_GAP);

                indexing_widget(app, ui, &status, cover_row.width());
                queue_button_slot(app, ui, cover_row, pill);
            },
        );
    } else if let Some(status) = update {
        ui.scope_builder(
            UiBuilder::new()
                .id_salt("playback_update_box")
                .max_rect(cover_row)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                theme::apply_centre_box_visuals(&mut ui.style_mut().visuals);
                ui.spacing_mut().item_spacing = Vec2::new(CENTRE_BOX_GAP, DETAILS_LINE_GAP);

                update_widget(app, ui, &status, cover_row.width());
                queue_button_slot(app, ui, cover_row, pill);
            },
        );
    } else if let Some(playing) = app.now_playing() {
        let title = playing.title.clone();
        let artwork = playing.artwork;
        let failure = app.playback_error();
        let state = failure.clone().unwrap_or_else(|| subtitle(app));

        // The two rows are given their own rectangles rather than stacked by the
        // layout, so each one lands where the pill's own height puts it. A row
        // that grew — a title and a state line at the box's spacing measure 35
        // points against the cover's 32 — would otherwise push the seek row out
        // through the box's foot, and the pill is placed from constants that
        // know nothing about how tall the text came out.
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

                queue_button_slot(app, ui, cover_row, pill);
            },
        );
    }

    // The seek row belongs to the track, not the top row, so indexing does
    // not claim it: a track playing during a scan stays seekable.
    if playing {
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
}

/// The queue button, pinned to the box's right edge and centred on the BOX's
/// own middle rather than the row's.
///
/// It is a slot of its own because two different top rows — the track's cover
/// and details, and the indexing widget — both draw it at the same coordinates:
/// the ☰ is the one thing on the pill that must not move when one takes over
/// from the other.
fn queue_button_slot(app: &mut MusicCraftApp, ui: &mut egui::Ui, cover_row: Rect, pill: Rect) {
    // The button is at the row's trailing edge, not after the details: the
    // pill is sized to hold both and its own padding, so placing it against
    // the edge lands it there whatever the row's contents came to.
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
}

/// The Indexing widget: the wording on top, iTunes 7's candy bar beneath it,
/// and Cancel on the bar's line at its right.
///
/// It fills exactly the space the cover and details use — everything left of
/// the gap before the queue button — so the pill never resizes, and the ☰
/// never shifts, when the widget takes the row over from a track.
fn indexing_widget(
    app: &mut MusicCraftApp,
    ui: &mut egui::Ui,
    status: &IndexingStatus,
    row_width: f32,
) {
    let width = row_width - QUEUE_BUTTON_SIZE - CENTRE_BOX_GAP;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);

        ui.label(RichText::new(indexing_caption(status)).small().weak());

        ui.horizontal(|ui| {
            let bar_width = (width - CANCEL_BUTTON_WIDTH - ui.spacing().item_spacing.x).max(40.0);
            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(bar_width, theme::CANDY_BAR_HEIGHT),
                Sense::hover(),
            );
            theme::candy_progress(
                ui,
                rect,
                (status.total > 0).then(|| status.done as f32 / status.total as f32),
            );

            if ui
                .add_sized(
                    [CANCEL_BUTTON_WIDTH, 18.0],
                    egui::Button::new(RichText::new("Cancel").small()),
                )
                .on_hover_text("Stop indexing after the file it is on")
                .clicked()
            {
                app.cancel_indexing();
            }
        });
    });
}

/// The widget's wording, in the numbers the job actually has.
///
/// No total is claimed while the walk is still counting — the bar slides and
/// the line reads simply "Indexing…" — and the left-out clause stays away
/// while nothing has been left out, the way iTunes' sheet said nothing it
/// could not yet say.
fn indexing_caption(status: &IndexingStatus) -> String {
    if status.total == 0 {
        return "Indexing…".to_owned();
    }

    if status.left_out == 0 {
        return format!("Indexing — {} of {}", status.done, status.total);
    }

    format!(
        "Indexing — {} of {} · {} left out",
        status.done, status.total, status.left_out
    )
}

/// The updater's voice in the centre box: the news that a newer version is
/// out, the bytes of it arriving, or the word that it is in and wants a
/// restart.
///
/// A job the user started outranks it — indexing took this row first and the
/// update waits its turn — but news of a version outranks the track: the
/// update waits across restarts, and the box saying so until it is taken is
/// the whole of its pointing. The button is deliberately not here: the gear
/// menu holds it, next to the Settings that talk about the application; the
/// second line only says where to find it.
fn update_widget(
    app: &mut MusicCraftApp,
    ui: &mut egui::Ui,
    status: &UpdateStatus,
    row_width: f32,
) {
    let width = row_width - QUEUE_BUTTON_SIZE - CENTRE_BOX_GAP;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);

        match status {
            UpdateStatus::Available {
                version,
                can_install,
            } => {
                let news = RichText::new(format!("Update available — {version}")).strong();
                ui.add(egui::Label::new(news).truncate().selectable(false));

                let way = if *can_install {
                    "Take it from the gear menu at the bar's right end"
                } else {
                    "This copy cannot replace itself — the gear menu opens the release"
                };
                ui.add(
                    egui::Label::new(RichText::new(way).small().weak())
                        .truncate()
                        .selectable(false),
                );
            }
            UpdateStatus::Downloading {
                version,
                done,
                total,
            } => {
                let caption = if *total == 0 {
                    format!("Updating to {version}…")
                } else {
                    format!(
                        "Updating to {version} — {} of {}",
                        crate::update::format_bytes(*done),
                        crate::update::format_bytes(*total)
                    )
                };
                ui.add(
                    egui::Label::new(RichText::new(caption).small().weak())
                        .truncate()
                        .selectable(false),
                );

                ui.horizontal(|ui| {
                    let bar_width =
                        (width - CANCEL_BUTTON_WIDTH - ui.spacing().item_spacing.x).max(40.0);
                    let (rect, _response) = ui.allocate_exact_size(
                        Vec2::new(bar_width, theme::CANDY_BAR_HEIGHT),
                        Sense::hover(),
                    );
                    theme::candy_progress(
                        ui,
                        rect,
                        (*total > 0).then(|| *done as f32 / *total as f32),
                    );

                    if ui
                        .add_sized(
                            [CANCEL_BUTTON_WIDTH, 18.0],
                            egui::Button::new(RichText::new("Cancel").small()),
                        )
                        .on_hover_text(
                            "Stop the download; the version you are running stays as it is",
                        )
                        .clicked()
                    {
                        app.cancel_update();
                    }
                });
            }
            UpdateStatus::Installed { version } => {
                let news = RichText::new(format!("{version} is installed")).strong();
                ui.add(egui::Label::new(news).truncate().selectable(false));
                ui.add(
                    egui::Label::new(
                        RichText::new("Restart to run it — the gear menu offers it")
                            .small()
                            .weak(),
                    )
                    .truncate()
                    .selectable(false),
                );
            }
            UpdateStatus::Failed { reason } => {
                let news = RichText::new("The update did not finish").strong();
                ui.add(egui::Label::new(news).truncate().selectable(false));
                ui.add(
                    egui::Label::new(RichText::new(reason).small().weak())
                        .truncate()
                        .selectable(false),
                );
            }
            // The box stays silent while a check is merely wondering; the
            // gear menu says "Checking…" to whoever looks.
            UpdateStatus::Checking => {}
        }
    });
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

    // Framed alike, the picture and its absence: the hairline rim is what
    // makes a small square read as an object set on the box rather than a
    // patch of one colour, and it is the same rim the box itself wears.
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, theme::CENTRE_BOX_EDGE),
        StrokeKind::Inside,
    );
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
    // The three glyphs are painted in the bar's OWN ink, not the white the
    // speaker and the volume rail take: their button is a white surface now, at
    // the user's request, and white on white would be nothing. It was called
    // `white` while it returned white, so the name is gone with the colour.
    fn glyph_ink(glyph: &str) -> RichText {
        RichText::new(glyph).color(theme::PLAYBACK_BAR_INK)
    }
    let size = [TRANSPORT_BUTTON_WIDTH, TRANSPORT_BUTTON_HEIGHT];

    // Each button gets the words the pointer already reads as hover text: a
    // screen reader is told "Previous" rather than "⏮", and the ring is what
    // Tab leaves behind. One shape so the three cannot disagree about which
    // control says what.
    let transport_button = |ui: &mut egui::Ui, glyph: &str, words: &str| {
        let response = ui
            .add_enabled(
                has_current,
                Button::new(glyph_ink(glyph)).min_size(size.into()),
            )
            .on_hover_text(words);
        theme::accessible_name(ui, &response, words);
        theme::focus_ring_if_focused(ui, &response);
        response.clicked()
    };

    if transport_button(ui, "⏮", "Previous") {
        app.previous_track();
    }

    let play = if app.playback().is_playing() {
        "⏸"
    } else {
        "▶"
    };
    if transport_button(ui, play, "Play or pause") {
        app.toggle_playback();
    }

    if transport_button(ui, "⏭", "Next") {
        app.next_track();
    }
}

/// The burger button at the right of the centre box, which opens the queue.
///
/// The glyph is drawn at its own size rather than at a text style's: see
/// `QUEUE_GLYPH_SIZE` for why the burger gets a size of its own.
///
/// Its face is painted rather than left to a button style: the little
/// push buttons wore their light — a rim, a lit top half, and a darker sink
/// while pressed or held down — and no widget fill gives that. The `Button`
/// is kept underneath, stripped to transparent fills, so the click, the
/// focus ring, the accessible name and the selectable state all stay egui's
/// own: only the paint has changed hands.
fn queue_button(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let queued = app.playback().queue().len();
    let open = app.queue_open();
    let words = format!(
        "Queue ({} {})",
        queued,
        if queued == 1 { "track" } else { "tracks" }
    );

    // The face is decided off the pointer directly rather than off a Response:
    // it is painted before the widget whose face it is, and a Response does
    // not exist yet. The same input says the same thing a frame later, when
    // the widget reads it — no disagreement between what is lit and what is
    // clickable.
    let (face, _) = ui.allocate_exact_size(Vec2::splat(QUEUE_BUTTON_SIZE), Sense::hover());
    let hovered = ui
        .input(|input| input.pointer.hover_pos())
        .is_some_and(|pos| face.contains(pos));
    let pushed = hovered && ui.input(|input| input.pointer.any_down());
    let sunk = pushed || open;

    let radius = CornerRadius::same(6);
    let fill = if sunk {
        theme::CENTRE_BOX_PRESSED
    } else if hovered {
        theme::CENTRE_BOX_GLOSS
    } else {
        theme::CENTRE_BOX_CONTROL
    };
    ui.painter().rect_filled(face, radius, fill);
    ui.painter().rect_stroke(
        face,
        radius,
        Stroke::new(1.0, theme::CENTRE_BOX_EDGE),
        StrokeKind::Inside,
    );
    if sunk {
        // Pushed in, the light goes and the shadow gathers along the top —
        // the rim of the hole the button has sunk into.
        ui.painter().hline(
            face.x_range().shrink(2.0),
            face.top() + 1.0,
            Stroke::new(1.0, theme::CENTRE_BOX_EDGE),
        );
    } else {
        let gloss = Rect::from_min_max(
            Pos2::new(face.left() + 1.0, face.top() + 1.0),
            Pos2::new(face.right() - 1.0, face.center().y),
        );
        ui.painter()
            .rect_filled(gloss, CornerRadius::same(5), theme::CENTRE_BOX_GLOSS);
    }

    // The button stripped: every fill it would paint, replaced by nothing,
    // for the states this one widget walks through. Inside its own scope, so
    // no other button on the bar loses its paint with it.
    let visuals = &mut ui.style_mut().visuals;
    for state in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        state.weak_bg_fill = Color32::TRANSPARENT;
        state.bg_fill = Color32::TRANSPARENT;
    }

    let response = ui
        .put(
            face,
            Button::selectable(open, RichText::new(QUEUE_GLYPH).size(QUEUE_GLYPH_SIZE)),
        )
        .on_hover_text(&words);
    theme::accessible_name(ui, &response, &words);
    theme::focus_ring_if_focused(ui, &response);

    if response.clicked() {
        app.toggle_queue();
    }
}

/// The queue popover, pinned under the playback bar.
///
/// It is a free-floating window rather than a panel child: the bar is already
/// drawn, so the queue can only be an overlay below it. Its ✕ writes the same
/// flag the burger button reads, so the button's highlight stays truthful
/// whichever way the popover was closed.
///
/// The strip across its top is the status bar's own gray. egui fills a window's
/// title bar from the theme's "open" widget shade, which moves with the theme —
/// pale on the light theme, near-black on the dark one — and the request is that
/// this strip read as the bar at the foot of the window does, in every theme.
/// The shade is set for this one call and put straight back afterwards, so no
/// other widget that draws an open state (a combo box, a menu) changes with it.
fn queue_popup(app: &mut MusicCraftApp, ui: &mut egui::Ui, bar: Rect) {
    if !app.queue_open() {
        return;
    }

    let mut open = true;

    let ctx = ui.ctx().clone();
    let previous = ctx.global_style();
    ctx.global_style_mut(|style| {
        style.visuals.widgets.open.weak_bg_fill = theme::STATUS_BAR_FILL;
    });

    egui::Window::new("Queue")
        .fixed_pos(bar.left_bottom() + egui::vec2(16.0, 12.0))
        .default_size([420.0, 300.0])
        .resizable(true)
        .open(&mut open)
        .show(&ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .show(ui, |ui| queueview::show(app, ui));
        });

    ctx.set_global_style(previous);

    if !open {
        app.close_queue();
    }
}

/// Speaker icon and volume slider, at the tail of the transport: the buttons
/// come first and the slider follows them, so the pointer's first stop is the
/// control that starts the music.
///
/// The group is handed a rectangle of its own by [`transport_row`], sized from
/// [`volume_width`], so the speaker is allocated exactly its box and the rail
/// exactly [`VOLUME_SLIDER_WIDTH`]: the label is a fixed 16 points rather than
/// the 11 the glyph inks, and the pair's own centre is the point the group is
/// placed by. A label sized to its text would make the group 5 points narrower
/// than the rectangle it is centred in and leave the rail that much right of
/// where the room between the buttons and the box says it belongs.
fn volume(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.spacing_mut().slider_width = VOLUME_SLIDER_WIDTH;
    ui.add_sized(
        Vec2::new(VOLUME_GLYPH_WIDTH, TRANSPORT_BUTTON_HEIGHT),
        egui::Label::new(
            RichText::new(VOLUME_GLYPH)
                .small()
                .color(theme::PLAYBACK_BAR_CONTROL_INK),
        ),
    );

    let mut value = app.playback().volume();

    // The rail and the handle are the one control the bar does not paint from a
    // widget fill: the slider asks for its rail and handle through the
    // interaction state's `bg_fill`, so the ink it is drawn in has to be set
    // here for the two of them and restored afterwards, or the transport
    // buttons that follow in the same row would be left without the bar's own
    // fills. It reads far better on the bar's gray than the white it was, which
    // is what the transport came off for.
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
    // The rail answers the arrow keys while it has focus; the name and the
    // ring are what say that this is the volume, and that the keyboard is here.
    theme::accessible_name(ui, &response, "Volume");
    theme::focus_ring_if_focused(ui, &response);

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

    /// The Library's pages moved out of the sidebar and onto the bar, at the
    /// request: three icon buttons centred in the room between the cream box and
    /// the search field, each with the icon named for it — a musical note, a
    /// compact disc, and a person's head and shoulders.
    ///
    /// Centre, not merely "somewhere between": the two gaps are measured, since
    /// a group that merely fitted in the room would pass an ordering check while
    /// sitting against one end of it.
    #[test]
    fn the_page_buttons_are_centred_between_the_box_and_the_search_field() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let field = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| (rect.width() - SEARCH_WIDTH).abs() < 1.0)
            .max_by(|a, b| a.right().total_cmp(&b.right()))
            .expect("the field paints its own background");

        // The three buttons' own squares, in the order they are drawn: their
        // glyphs are narrower than the buttons they sit in, so it is the squares
        // that say where the group's two ends are.
        let mut buttons: Vec<egui::Rect> = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| {
                (rect.width() - PAGE_BUTTON_SIZE).abs() < 1.0
                    && (rect.height() - PAGE_BUTTON_SIZE).abs() < 1.0
                    && rect.center().x > pill.right()
            })
            .collect();
        buttons.sort_by(|a, b| a.left().total_cmp(&b.left()));
        assert_eq!(buttons.len(), 3, "the bar does not draw three pages");

        let first = buttons[0];
        let last = buttons[2];

        assert!(
            first.left() > pill.right() && last.right() < field.left(),
            "the pages are not between the box and the field: \
             {first:?}, {last:?}, {pill:?}, {field:?}"
        );

        // The two gaps, end to end — each button's own edge, since the group is
        // what is centred rather than the first and last glyph.
        let before = first.left() - pill.right();
        let after = field.left() - last.right();
        assert!(
            (before - after).abs() <= 1.0,
            "the pages are not centred between the box and the field: \
             {before} before them, {after} after them"
        );

        // And the group's own centre is the room's midpoint, which is the same
        // statement read from the other end.
        assert!(
            (f32::midpoint(first.left(), last.right()) - f32::midpoint(pill.right(), field.left()))
                .abs()
                <= 1.0,
            "the group's centre is not the midpoint of the room: {} against {}",
            f32::midpoint(first.left(), last.right()),
            f32::midpoint(pill.right(), field.left())
        );
    }

    /// All three page buttons are filled white, and the open one is still told
    /// apart from the closed two.
    ///
    /// The fill is the point of the request, and it also takes away the cue that
    /// used to say which page was open: a `Button`'s `selected` fills it through
    /// the theme's selection, and an explicit `fill` overrides that in every
    /// state. So the open page has to say so some other way, and the accent ring
    /// is it — if the white fill ever leaves all three looking alike, this fails.
    #[test]
    fn every_page_button_is_white_and_the_open_one_is_still_marked() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        let in_group = |rect: &egui::Rect| {
            (rect.width() - PAGE_BUTTON_SIZE).abs() < 3.0
                && (rect.height() - PAGE_BUTTON_SIZE).abs() < 3.0
                && rect.center().x > pill.right()
        };

        // The three white squares, found by their size, to the right of the box.
        let mut white: Vec<egui::Rect> = painted
            .rects_with(theme::PLAYBACK_BAR_CONTROL)
            .into_iter()
            .filter(|rect| in_group(rect))
            .collect();
        white.sort_by(|a, b| a.left().total_cmp(&b.left()));

        assert_eq!(
            white.len(),
            3,
            "not every page button is filled with the bar's white ({} found: {white:?})",
            white.len()
        );

        // The open page is Tracks by default, which is the leftmost button; its
        // ring is the accent, where the two closed ones wear the grey edge.
        let accents = painted.stroked_with(theme::ACCENT);
        assert!(
            !accents.is_empty(),
            "the open page no longer marks itself, so all three read alike"
        );
        let ring = accents
            .into_iter()
            .find(|rect| in_group(rect))
            .expect("the accent ring is on the open page's button");

        assert!(
            (ring.center().x - white[0].center().x).abs() <= 2.0,
            "the accent ring is not on the open page: ring {ring:?}, \
             first button {:?}, all {white:?}",
            white[0]
        );
    }

    /// Each page carries the icon the request named for it, and the one that is
    /// open says so. The glyphs are read off the painted shapes rather than off
    /// the constants, so a button that stopped drawing its icon fails here.
    ///
    /// The open page's icon is the accent, like its ring: the three buttons are
    /// white now, so the bar's ink alone would not say which one you are on.
    #[test]
    fn each_page_button_carries_its_own_icon() {
        let mut app = app();

        for (kind, glyph) in [
            (BrowseKind::Tracks, TRACKS_GLYPH),
            (BrowseKind::Albums, ALBUMS_GLYPH),
            (BrowseKind::Artists, ARTISTS_GLYPH),
        ] {
            app.browse_mut().set_kind(kind);
            let painted = paint_bar(&mut app);

            assert!(
                painted.ink(glyph).is_some(),
                "the {} page draws no icon",
                kind.label()
            );
            assert!(
                painted.coloured(glyph, theme::ACCENT).is_some(),
                "the open {} page's icon is not the accent, which is what marks \\
                 the page you are on now that every button is white",
                kind.label()
            );

            // And the two closed pages' icons keep the bar's ink, which is what
            // reads on a white button.
            for (other, other_glyph) in [
                (BrowseKind::Tracks, TRACKS_GLYPH),
                (BrowseKind::Albums, ALBUMS_GLYPH),
                (BrowseKind::Artists, ARTISTS_GLYPH),
            ] {
                if other == kind {
                    continue;
                }
                assert!(
                    painted
                        .coloured(other_glyph, theme::PLAYBACK_BAR_INK)
                        .is_some(),
                    "the closed {} page's icon is not in the bar's ink",
                    other.label()
                );
            }
        }
    }

    /// Clicking a page button opens that page: the pages moved off the sidebar,
    /// and a control that no longer answers is not a control that moved.
    #[test]
    fn a_click_on_a_page_button_opens_that_page() {
        let mut app = app();
        assert_eq!(app.browse().kind(), BrowseKind::Tracks);

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

        // One pass so egui learns the buttons' rectangles: a hit is decided
        // against the previous frame.
        let painted = frame(Vec::new());
        let albums = painted
            .ink(ALBUMS_GLYPH)
            .expect("the Albums button was drawn")
            .center();

        frame(vec![egui::Event::PointerMoved(albums)]);
        frame(vec![egui::Event::PointerButton {
            pos: albums,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }]);
        frame(vec![egui::Event::PointerButton {
            pos: albums,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }]);

        assert_eq!(
            app.browse().kind(),
            BrowseKind::Albums,
            "clicking the Albums button at {albums:?} did not open the page"
        );
    }

    /// The pages are the Library's, so the bar offers them only there — the same
    /// rule the search field beside them follows.
    #[test]
    fn the_page_buttons_are_only_on_the_library() {
        let mut app = app();
        app.set_section(crate::ui::navigation::NavSection::Playlists);

        let painted = paint_bar(&mut app);

        for glyph in [TRACKS_GLYPH, ALBUMS_GLYPH, ARTISTS_GLYPH] {
            assert!(
                painted.ink(glyph).is_none(),
                "the bar offers the Library's pages outside the Library"
            );
        }
    }

    /// The three icons really paint, rather than falling back to the
    /// replacement box.
    ///
    /// The app installs no fonts of its own and paints with epaint's bundled
    /// set, so a glyph outside that set would come out as `◻` here — and a
    /// request for a musical note, a disc and a person would be answered with
    /// three identical boxes.
    ///
    /// `Fonts::has_glyphs` is NOT the check to use: epaint picks its
    /// replacement glyph's face with `has_glyph` and returns false for every
    /// char that face owns, so `has_glyphs` reports a false negative for the
    /// whole of NotoEmoji — the one face that carries these three. The glyphs
    /// are read off the atlas instead: each is rasterised, the ink it paints is
    /// hashed, and the hash is compared against a glyph nothing can draw. A
    /// glyph that is really present has its own shape; one that is replaced
    /// comes back with the replacement's.
    #[test]
    fn the_page_icons_paint_their_own_shapes() {
        let ctx = egui::Context::default();
        let mut drawn: Vec<(&str, u64)> = Vec::new();
        let mut replacement = 0;

        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::FontId::proportional(PAGE_BUTTON_SIZE / 2.0);

            // A char no bundled face can draw, to learn what a replaced glyph
            // looks like on this build.
            replacement = painted_ink(ui, &font, "\u{FE838}");

            for glyph in [TRACKS_GLYPH, ALBUMS_GLYPH, ARTISTS_GLYPH] {
                drawn.push((glyph, painted_ink(ui, &font, glyph)));
            }
        });
        output.drop_without_applying_deltas();

        for (glyph, ink) in drawn {
            assert_ne!(
                ink, replacement,
                "{glyph:?} is painted as the replacement box: the bundled fonts \
                 do not carry it, so that button would not show its icon"
            );
        }
    }

    /// The ink one glyph paints, hashed: enough to tell one shape from another
    /// without naming any of them.
    ///
    /// The glyph is laid out and the atlas region it was rasterised into is
    /// read back — the same pixels the renderer would upload.
    fn painted_ink(ui: &egui::Ui, font: &egui::FontId, glyph: &str) -> u64 {
        ui.ctx().fonts_mut(|fonts| {
            let galley = fonts.layout_no_wrap(glyph.to_owned(), font.clone(), egui::Color32::WHITE);
            let image = fonts.image();
            let (width, height) = (image.size[0], image.size[1]);
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

            for row in &galley.rows {
                for g in &row.glyphs {
                    let uv = g.uv_rect;
                    let x_end = (uv.max[0] as usize).min(width);
                    let y_end = (uv.max[1] as usize).min(height);

                    for y in uv.min[1] as usize..y_end {
                        for x in uv.min[0] as usize..x_end {
                            let [r, g, b, a] = image.pixels[y * width + x].to_array();
                            let pixel = u64::from(r)
                                | (u64::from(g) << 8)
                                | (u64::from(b) << 16)
                                | (u64::from(a) << 24);
                            hash ^= pixel;
                            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                        }
                    }
                }
            }

            hash
        })
    }

    /// On a window too narrow for everything, the field gives way and the pages
    /// still fit between the box and it.
    ///
    /// The bar has only so much room, and at the app's narrowest window the
    /// three buttons and the field cannot both have their full width. What must
    /// not happen is the pages being pushed back over the box, or the buttons
    /// ending up under the Clear button's reserved room — the buttons are the
    /// navigation and the field only elides a query, so the field is what is
    /// trimmed.
    #[test]
    fn the_pages_still_fit_on_the_narrowest_window() {
        for width in [760.0, 900.0, 1000.0, 1180.0] {
            let bar = Rect::from_min_size(Pos2::ZERO, Vec2::new(width, theme::PLAYBACK_BAR_HEIGHT));
            let pill = centre_box_rect(bar);
            let field = search_rect(bar, pill);
            let clear = Rect::from_min_size(
                Pos2::new(field.left() - SEARCH_GAP - SEARCH_CLEAR_WIDTH, field.top()),
                Vec2::new(SEARCH_CLEAR_WIDTH, field.height()),
            );
            let pages = page_buttons_rect(bar, pill, field, clear);

            assert!(
                pages.left() >= pill.right(),
                "at {width} wide the pages overlap the centre box: {pages:?} vs {pill:?}"
            );
            assert!(
                pages.right() <= clear.left(),
                "at {width} wide the pages run under the Clear button: \
                 {pages:?} vs {clear:?}"
            );
            assert!(
                field.left() >= pill.right(),
                "at {width} wide the field overlaps the centre box: {field:?} vs {pill:?}"
            );
            assert!(
                field.right() <= bar.right(),
                "at {width} wide the field runs off the bar: {field:?} vs {bar:?}"
            );

            // And the group is still centred in the room, which is what the
            // request asked for at every width.
            let before = pages.left() - pill.right();
            let after = field.left() - pages.right();
            assert!(
                (before - after).abs() <= 1.0,
                "at {width} wide the pages are not centred: {before} before, {after} after"
            );
        }

        // The bar is measured through the real frame at its own width, so the
        // helper above is not the only thing agreeing with itself.
        let mut app = app();
        let painted = paint_bar(&mut app);
        assert!(
            painted.ink(TRACKS_GLYPH).is_some(),
            "the pages are not drawn on the window the bar is painted into"
        );
    }

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

    /// With nothing on the bar the pill is still there but empty: the cream
    /// surface stays, and nothing it holds — no cover slot, no title and state
    /// line, no seek row and no queue button — is painted on it. The transport
    /// and the Library's own controls are the bar's, not the widget's, so they
    /// stay too.
    #[test]
    fn an_empty_bar_keeps_the_cream_box_but_puts_nothing_in_it() {
        let mut app = app();
        assert_eq!(
            app.now_playing(),
            None,
            "the fixture has no track on the bar"
        );

        let painted = paint_bar(&mut app);

        // The widget's own surface is still there.
        assert!(
            !painted.rects_with(theme::CENTRE_BOX_FILL).is_empty(),
            "the empty bar closed the centre box instead of leaving it bare"
        );

        // But nothing is drawn on it.
        assert!(
            painted.ink("Nothing playing").is_none(),
            "the empty bar still names the nothing-playing state"
        );
        assert!(
            painted.ink(QUEUE_GLYPH).is_none(),
            "the empty bar still draws the queue button"
        );
        assert!(
            painted.ink("--:--").is_none(),
            "the empty bar still draws the seek row"
        );
        let cover_slot = painted
            .rects_with(theme::CENTRE_BOX_CONTROL)
            .into_iter()
            .find(|rect| {
                (rect.width() - COVER_SIZE).abs() < 0.5 && (rect.height() - COVER_SIZE).abs() < 0.5
            });
        assert!(
            cover_slot.is_none(),
            "the empty bar still marks out the cover slot"
        );

        // Only the widget's contents go: the transport and the Library's own
        // controls are still on the bar.
        assert!(
            painted.ink("▶").is_some(),
            "the transport went with the widget's contents"
        );
        assert!(
            painted.ink(TRACKS_GLYPH).is_some(),
            "the Library's own controls went with the widget's contents"
        );
    }

    /// A track on the bar fills the box again: the surface is the same, and now
    /// the track's title, its queue button and its seek row are painted on it.
    #[test]
    fn a_track_on_the_bar_fills_the_centre_box() {
        let mut app = app_with_a_loaded_track();
        let painted = paint_bar(&mut app);

        assert!(
            !painted.rects_with(theme::CENTRE_BOX_FILL).is_empty(),
            "the centre box is not drawn for a track on the bar"
        );
        assert!(
            painted.ink("song").is_some(),
            "the track on the bar is not named"
        );
        assert!(
            painted.ink(QUEUE_GLYPH).is_some(),
            "the queue button is not drawn for a track on the bar"
        );
        assert!(
            painted.ink("Nothing playing").is_none(),
            "the empty-state wording is still painted beside a track"
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
        #[allow(dead_code)]
        rects: Vec<(egui::Rect, egui::Color32)>,
        /// Every rectangle's outline, which is where a hand-painted edge lives —
        /// `Shape::Rect` carries its fill and its stroke separately, so an
        /// accent ring on a white button is NOT in `rects`.
        strokes: Vec<(egui::Rect, egui::Color32)>,
        images: Vec<egui::Rect>,
        texts: Vec<egui::epaint::TextShape>,
    }

    impl Painted {
        fn collect(&mut self, shape: &egui::Shape) {
            match shape {
                egui::Shape::Rect(rect) => {
                    self.rects.push((rect.rect, rect.fill));
                    self.strokes.push((rect.rect, rect.stroke.color));
                }
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

        /// Every rectangle OUTLINED in `colour` — the stroke, not the fill.
        fn stroked_with(&self, colour: egui::Color32) -> Vec<egui::Rect> {
            self.strokes
                .iter()
                .filter(|(_, stroke)| *stroke == colour)
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
    /// Found by its PARTIAL TRANSLUCENCY rather than its colour or its size:
    /// the rail and the played part of it over it are drawn with an alpha,
    /// where every opaque rectangle on this bar is a surface — the bar's own
    /// fill is 1180 points wide and would otherwise be the widest rectangle
    /// in the frame. Fully transparent fills are excluded with the opaque
    /// ones: since the box gained its rim, those are the hand-painted edges —
    /// a stroke's rectangle carries no fill at all. A slider with no known
    /// duration is drawn DISABLED, in egui's own disabled style rather than
    /// the box's control shade, so the colour is not a property these tests
    /// can lean on either.
    fn seek_rail(painted: &Painted) -> egui::Rect {
        painted
            .rects
            .iter()
            .filter(|(_, colour)| (1..255).contains(&colour.a()))
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

    /// The volume group is centred on the room the transport buttons and the
    /// centre box leave between them.
    ///
    /// The two are what it is between: the user asked for the volume control in
    /// the middle of the gap rather than pushed up against the buttons, so the
    /// assertion is on the two gaps — button row to the speaker, and the rail to
    /// the pill — being equal. Measured off a real frame, where both ends of the
    /// group are painted rather than read from the constants that placed it.
    #[test]
    fn the_volume_is_centred_between_the_transport_and_the_centre_box() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let speaker = painted.ink(VOLUME_GLYPH).expect("the volume speaker");
        let rail = painted
            .rects_with(theme::PLAYBACK_BAR_CONTROL_INK)
            .into_iter()
            .filter(|rect| (rect.width() - VOLUME_SLIDER_WIDTH).abs() < 2.0)
            .max_by(|a, b| a.width().total_cmp(&b.width()))
            .expect("the volume slider's rail");
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        // The buttons' own rectangles, not their glyphs' ink: a glyph is
        // narrower than the button it sits in, and it is the button the volume
        // is set off from.
        let buttons_end = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| {
                (rect.width() - TRANSPORT_BUTTON_WIDTH).abs() < 0.5
                    && (rect.height() - TRANSPORT_BUTTON_HEIGHT).abs() < 0.5
                    && rect.left() < WINDOW_WIDTH / 4.0
            })
            .map(|rect| rect.right())
            .max_by(f32::total_cmp)
            .expect("the transport buttons paint their own rectangles");

        // The glyph is inset in its 16-point box, so the group's left edge is
        // half the inset further left than the speaker's own ink.
        let group_left = speaker.center().x - VOLUME_GLYPH_WIDTH / 2.0;
        let before = group_left - buttons_end;
        let after = pill.left() - rail.right();

        assert!(
            before > 0.0 && after > 0.0,
            "the volume is not between the buttons and the box: {before} before, \
             {after} after ({speaker:?}, {rail:?}, {pill:?})"
        );
        assert!(
            (before - after).abs() <= 1.0,
            "the volume is not centred between the transport and the box: \
             {before} before it, {after} after it"
        );

        // And the group as a whole — the speaker's fixed box and the rail — is
        // the thing that is centred. The box is at the group's head, where the
        // glyph is inset both sides, so the group's left edge is that half-inset
        // out from the speaker's own ink; its right edge is the rail's.
        let group_right = rail.right();
        assert!(
            ((group_left + group_right) / 2.0 - f32::midpoint(buttons_end, pill.left())).abs()
                <= 0.5,
            "the volume group's centre is not the room's midpoint: \
             group {} against {}",
            f32::midpoint(group_left, group_right),
            f32::midpoint(buttons_end, pill.left())
        );
    }

    /// The transport split when the user asked for the search field and the
    /// buttons to be white. The field and the buttons are now a WHITE surface,
    /// so the three glyphs printed on a button take the bar's own dark ink, and
    /// the white is what is left to the two controls drawn straight ON the bar's
    /// gray — the speaker at the head of the volume slider and the slider's own
    /// rail. Colours come off the painted shapes, not off the constants, so a
    /// call site that stopped applying one fails here.
    #[test]
    fn the_transport_glyphs_take_the_bars_ink_on_their_white_buttons() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        // The buttons the glyphs are printed on are white, the surface the user
        // asked for, and the glyphs on them are the bar's ink so they read.
        assert_eq!(
            theme::PLAYBACK_BAR_CONTROL,
            egui::Color32::WHITE,
            "the buttons are no longer the white surface the user asked for"
        );
        assert_eq!(
            theme::PLAYBACK_BAR_INK,
            egui::Color32::from_rgb(32, 32, 36),
            "the bar's ink moved, and the glyphs are read on the white tiles"
        );

        for (glyph, what) in [("⏮", "previous"), ("▶", "play"), ("⏭", "next")] {
            let ink = painted
                .coloured(glyph, theme::PLAYBACK_BAR_INK)
                .unwrap_or_else(|| {
                    panic!("the {what} glyph is not painted in the bar's ink on its white button")
                });
            assert!(
                ink.width() > 1.0,
                "the {what} glyph is painted but carries no ink: {ink:?}"
            );

            // And the button behind it is the fill the glyph is read off: this
            // fixture carries no track, so the three buttons are DISABLED and
            // egui paints one by multiplying the control's colour by the
            // theme's `disabled_alpha` — the white control at half is a
            // #80808080 rect over the bar. Reading it back through the same
            // gamma-multiply is what proves the button is the white control
            // dimmed, rather than the theme's own fill left in place.
            let button = egui::Rect::from_center_size(ink.center(), egui::vec2(38.0, 28.0));
            let painted_button = painted
                .rects
                .iter()
                .find(|(rect, _)| {
                    (rect.width() - TRANSPORT_BUTTON_WIDTH).abs() < 0.5
                        && (rect.height() - TRANSPORT_BUTTON_HEIGHT).abs() < 0.5
                        && rect.intersects(button)
                })
                .unwrap_or_else(|| panic!("the {what} button paints no background: {button:?}"));
            assert_eq!(
                painted_button.1,
                dimmed(theme::PLAYBACK_BAR_CONTROL),
                "the {what} button is not the white control dimmed as a disabled one, \
                 but {:#?}: {button:?}",
                painted_button.1
            );
        }
    }

    /// A disabled widget's fill, as egui paints it: the control's own colour
    /// through the same `gamma_multiply` the painter applies when it dims a
    /// widget, at the alpha the visuals carry. Alpha included, so a button
    /// painted in the theme's fill rather than the bar's would fail here.
    fn dimmed(fill: egui::Color32) -> egui::Color32 {
        fill.gamma_multiply(egui::Visuals::dark().disabled_alpha())
    }

    /// The three transport buttons rest in the white the user asked for,
    /// whatever their state: an enabled button takes it from `inactive` and a
    /// disabled one — the transport with nothing queued, which is how the app
    /// opens — from `noninteractive`, and egui dims both by the same alpha. A
    /// call site that set only one of them shows white tiles with a track
    /// playing and theme-coloured ones without.
    #[test]
    fn a_playback_control_rests_in_the_white_in_every_theme() {
        for theme in [egui::Visuals::light(), egui::Visuals::dark()] {
            let mut visuals = theme;
            theme::apply_playback_bar_visuals(&mut visuals);

            assert_eq!(
                visuals.widgets.inactive.weak_bg_fill,
                theme::PLAYBACK_BAR_CONTROL,
                "an enabled button is not rested in the white"
            );
            assert_eq!(
                visuals.widgets.noninteractive.weak_bg_fill,
                theme::PLAYBACK_BAR_CONTROL,
                "a disabled button — the transport with nothing queued — takes \
                 the theme's own fill instead of the white"
            );
            assert_eq!(theme::PLAYBACK_BAR_CONTROL, egui::Color32::WHITE);
        }
    }

    /// What is left white is the pair that has no surface of its own: the
    /// speaker at the head of the volume slider and the rail it draws on the
    /// bar's gray. White on that gray is about 2.1:1 — the price of the user's
    /// request — and both are painted in it together, so the volume reads as one
    /// control rather than as white ink beside a rail of another shade.
    #[test]
    fn the_volume_group_is_painted_white_on_the_bar() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        let white = theme::PLAYBACK_BAR_CONTROL_INK;
        assert_eq!(
            white,
            egui::Color32::WHITE,
            "the volume group's ink is no longer white"
        );

        let speaker = painted
            .coloured(VOLUME_GLYPH, white)
            .expect("the speaker is not painted white");

        // The rail is the white rectangle after the speaker: the three transport
        // glyphs carry the bar's ink now, so the rail is the only rectangle of
        // the white in the frame.
        let rail = painted
            .rects_with(white)
            .into_iter()
            .filter(|rect| rect.width() > 20.0)
            .max_by(|a, b| a.left().total_cmp(&b.left()));
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
        let mut app = app_with_a_loaded_track();
        let painted = paint_bar(&mut app);

        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let burger = painted.ink(QUEUE_GLYPH).expect("the queue button");
        let title = painted.ink("song").expect("the title");

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
        let mut app = app_with_a_loaded_track();
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

    /// The queue icon is painted at the size of the glyphs it sits among,
    /// rather than left to `.small()`.
    ///
    /// Asserted on what the BAR painted and not on a font looked up by name: a
    /// test that lays the glyph out at a size of its own choosing measures the
    /// font rather than the button, and went on passing with the button put back
    /// the way it was. This reads the text the widget actually emitted, so it
    /// fails when the button's own size changes.
    #[test]
    fn the_queue_icon_is_as_big_as_the_glyphs_beside_it() {
        let mut app = app_with_a_loaded_track();
        let painted = paint_bar(&mut app);

        let queue = painted.ink(QUEUE_GLYPH).expect("the queue button");
        let play = painted
            .ink("\u{25B6}")
            .expect("the transport draws its play glyph");

        // The transport's glyphs are the ones the queue icon sits level with.
        assert!(
            queue.height() >= play.height() - 1.0,
            "the queue icon is shorter than the transport's: {queue:?} against {play:?}"
        );

        // And it still fits the square that caps it.
        let button = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .find(|rect| (rect.width() - QUEUE_BUTTON_SIZE).abs() < 0.5)
            .expect("the queue button paints its own square");
        assert!(
            button.contains_rect(queue),
            "the queue icon has outgrown its button: {queue:?} in {button:?}"
        );
    }

    /// The queue button still opens the queue where it landed: the box moved it
    /// to the middle of the bar, and a control that no longer answers is not a
    /// control that moved.
    #[test]
    fn a_click_on_the_queue_button_opens_the_queue() {
        let mut app = app_with_a_loaded_track();
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

    /// The box's cream stood off the bar rather than dissolving into it: a
    /// hairline rim on the outside, and a lighter line just inside it — the
    /// edge catching the light, which is what makes a flat shape read as a
    /// raised one. Painted from the constants, and asserted as painted.
    #[test]
    fn the_centre_box_is_rimmed_and_lit_like_a_raised_panel() {
        let mut app = app_with_a_loaded_track();
        let painted = paint_bar(&mut app);
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        assert!(
            painted.stroked_with(theme::CENTRE_BOX_EDGE).contains(&pill),
            "the box has no rim of its own against the bar: {pill:?}"
        );
        assert!(
            painted
                .stroked_with(theme::CENTRE_BOX_GLOSS)
                .contains(&pill.shrink(1.0)),
            "nothing catches the light along the box's inner edge"
        );
    }

    /// The queue button is a small push button of the same school: lighter
    /// face than the box it sits on, rimmed, and lit along its top half.
    #[test]
    fn the_queue_button_is_a_lit_push_button_on_the_box() {
        let mut app = app_with_a_loaded_track();
        let painted = paint_bar(&mut app);
        let face = painted
            .rects_with(theme::CENTRE_BOX_CONTROL)
            .into_iter()
            .find(|rect| {
                (rect.width() - QUEUE_BUTTON_SIZE).abs() < 0.5
                    && (rect.height() - QUEUE_BUTTON_SIZE).abs() < 0.5
            })
            .expect("the queue button paints its own face");

        assert!(
            painted.stroked_with(theme::CENTRE_BOX_EDGE).contains(&face),
            "the button has no rim: {face:?}"
        );

        let gloss = painted
            .rects_with(theme::CENTRE_BOX_GLOSS)
            .into_iter()
            .find(|rect| face.contains_rect(*rect))
            .expect("the button's top half is lit");
        assert!(
            gloss.top() <= face.top() + 2.0 && (gloss.bottom() - face.center().y).abs() <= 1.0,
            "the light sits somewhere other than the button's top: {gloss:?} in {face:?}"
        );
    }

    /// With the queue open the button shows pushed in: the sink of a pressed
    /// face, so the open queue has a source on the bar that reads as held
    /// down rather than merely recoloured.
    #[test]
    fn an_open_queue_shows_the_button_pushed_in() {
        let mut app = app_with_a_loaded_track();
        app.toggle_queue();
        let painted = paint_bar(&mut app);

        let sunk = painted
            .rects_with(theme::CENTRE_BOX_PRESSED)
            .into_iter()
            .find(|rect| {
                (rect.width() - QUEUE_BUTTON_SIZE).abs() < 0.5
                    && (rect.height() - QUEUE_BUTTON_SIZE).abs() < 0.5
            })
            .expect("an open queue leaves its button sunk");
        let glyph = painted.ink(QUEUE_GLYPH).expect("the queue button");
        assert!(
            sunk.contains_rect(glyph),
            "the sunk face is not under the button it belongs to: {sunk:?} vs {glyph:?}"
        );
    }

    /// The cover wears the box's rim too, picture and empty slot alike: the
    /// hairline is what makes the small square read as an object set ON the
    /// box rather than a patch of colour let into it.
    #[test]
    fn the_cover_slot_wears_the_boxs_rim() {
        let mut app = app();
        app.open_path(Path::new("/music/song.mp3"));
        let painted = paint_bar(&mut app);

        let slot = painted
            .rects_with(theme::CENTRE_BOX_CONTROL)
            .into_iter()
            .find(|rect| {
                (rect.width() - COVER_SIZE).abs() < 0.5 && (rect.height() - COVER_SIZE).abs() < 0.5
            })
            .expect("the empty cover slot is marked out");
        assert!(
            painted.stroked_with(theme::CENTRE_BOX_EDGE).contains(&slot),
            "the slot has no frame: {slot:?}"
        );
    }

    /// Ask the app to index a scratch folder and leave it mid-job.
    ///
    /// Nothing here polls the library, so the app keeps reporting itself as
    /// scanning for every frame the bar draws — the state the Indexing widget
    /// exists for. The session itself looks at an empty scratch folder and
    /// ends on its own thread, where nobody waits for it.
    fn start_indexing(app: &mut MusicCraftApp) {
        app.ask_to_add(crate::ui::dialog::PendingAdd::Folders {
            paths: vec![crate::fixtures::scratch("index-watch")],
        });
        app.add_paths_in_place();

        assert!(app.is_scanning(), "the app should be mid-scan");
    }

    /// An app holding a library but no track: the bar's pill is bare until
    /// something plays or something is being indexed.
    fn app_with_a_bare_bar() -> MusicCraftApp {
        let library =
            crate::library::Library::in_memory(crate::fixtures::scratch("index-bare-bar"))
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

    /// Indexing takes the box's top row from the track — the wording, the
    /// candy groove and Cancel all land there — and the ☰ holds its exact
    /// spot through the swap: the one thing on the pill that must not move.
    #[test]
    fn indexing_takes_the_top_row_from_a_playing_track() {
        let mut app = app_playing_a_timed_track();
        let resting = paint_bar(&mut app);
        let burger_before = resting
            .ink(QUEUE_GLYPH)
            .expect("the queue button sits on the track's row");
        // The bar's title is the opened path's stem, not the stored row's:
        // `open_path` plays the file, and the file's stem is its name.
        assert!(resting.ink("Seek").is_some(), "the track is on the bar");

        start_indexing(&mut app);
        let painted = paint_bar(&mut app);

        // The walk has not reported back yet on this empty folder, so the
        // honest caption is the sliding one — and the numbers clause only
        // ever appears with a total to put them against.
        assert!(
            painted.ink("Indexing").is_some(),
            "the widget did not take the row: {:?}",
            painted
                .texts
                .iter()
                .map(|text| text.galley.text())
                .collect::<Vec<_>>()
        );
        assert!(painted.ink("Cancel").is_some(), "no way out of the job");
        assert!(
            painted.ink("Seek").is_none(),
            "the title stayed on the row the widget took"
        );
        assert!(
            !painted.rects_with(theme::CANDY_GROOVE_FILL).is_empty(),
            "the widget drew no groove to fill"
        );

        let burger_after = painted
            .ink(QUEUE_GLYPH)
            .expect("the queue button holds its spot while indexing");
        assert!(
            (burger_before.min - burger_after.min).length() <= 1.0,
            "the ☰ moved when the widget took the row: {burger_before:?} vs {burger_after:?}"
        );
    }

    /// Nothing playing is not nothing happening: while a job runs the widget
    /// speaks from the otherwise-bare pill, and the ☰ comes out with it —
    /// the queue is reachable through a scan exactly as it is under a track.
    #[test]
    fn indexing_speaks_from_the_pill_even_with_nothing_playing() {
        let mut app = app_with_a_bare_bar();
        let idle = paint_bar(&mut app);
        assert!(
            idle.ink("Indexing").is_none() && idle.ink(QUEUE_GLYPH).is_none(),
            "an idle bar with no track has nothing to say"
        );

        start_indexing(&mut app);
        let painted = paint_bar(&mut app);

        assert!(painted.ink("Indexing").is_some(), "the job is silent");
        assert!(painted.ink("Cancel").is_some(), "no way out of the job");
        assert!(
            painted.ink(QUEUE_GLYPH).is_some(),
            "the queue is unreachable while indexing"
        );
        assert!(
            !painted.rects_with(theme::CANDY_GROOVE_FILL).is_empty(),
            "the widget drew no groove to fill"
        );
    }

    /// Drawing the widget is not clicking Cancel: a frame that shows the job
    /// must never be the frame that stops it.
    #[test]
    fn drawing_the_indexing_cancel_button_cancels_nothing() {
        let mut app = app_with_a_bare_bar();
        start_indexing(&mut app);

        paint_bar(&mut app);

        assert!(app.is_scanning(), "drawing is not clicking");
    }

    /// A newer release takes the box's top row from the track, says where the
    /// button lives, and leaves the ☰ where it is: the message moved to the
    /// box, the action stayed in the gear menu.
    #[test]
    fn an_available_update_speaks_from_the_box() {
        let mut app = app_with_a_loaded_track();
        app.set_release_update(crate::update::Update::Available {
            version: "v9.9.9".to_owned(),
            url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
            asset: None,
        });

        let painted = paint_bar(&mut app);

        assert!(
            painted.ink("Update available — v9.9.9").is_some(),
            "the news did not take the box"
        );
        assert!(
            painted.ink("gear menu").is_some(),
            "the line does not say where to take the update from"
        );
        assert!(
            painted.ink("song").is_none(),
            "the track kept the row the update took"
        );
        assert!(
            painted.ink(QUEUE_GLYPH).is_some(),
            "the ☰ left the box for the notice"
        );
    }

    /// The check being merely underway says nothing on the bar: the track
    /// keeps its row until there is an answer worth showing.
    #[test]
    fn a_check_underway_says_nothing_in_the_box() {
        let mut app = app_with_a_loaded_track();
        app.set_check_pending();

        let painted = paint_bar(&mut app);

        assert!(
            painted.ink("Update").is_none(),
            "the box spoke while the check only wondered"
        );
        assert!(
            painted.ink("song").is_some(),
            "the track lost its row anyway"
        );
    }

    /// A job the user started outranks news the app found on its own: while
    /// files are coming in, that is what the box is for.
    #[test]
    fn indexing_outranks_an_update_for_the_row() {
        let mut app = app_with_a_bare_bar();
        app.set_release_update(crate::update::Update::Available {
            version: "v9.9.9".to_owned(),
            url: "https://e/v9.9.9".to_owned(),
            asset: None,
        });

        start_indexing(&mut app);
        let painted = paint_bar(&mut app);

        assert!(painted.ink("Indexing").is_some(), "the job lost the row");
        assert!(
            painted.ink("Update available").is_none(),
            "the news spoke over a job in progress"
        );
    }

    /// An update arriving is a job like any other: it fills the same candy
    /// groove with its bytes and offers the same Cancel beside it.
    #[test]
    fn a_downloading_update_fills_the_candy_bar_and_offers_cancel() {
        let mut app = app_with_a_bare_bar();
        app.set_install(
            Some("v9.9.9"),
            Some(crate::update::Install::Running {
                done: 3 * 1024 * 1024,
                total: 12 * 1024 * 1024,
            }),
        );

        let painted = paint_bar(&mut app);

        assert!(painted.ink("Updating to v9.9.9").is_some(), "no word");
        assert!(
            painted.ink("3.0 MB of 12.0 MB").is_some(),
            "the byte counts are not the ones the download has: {:?}",
            painted
                .texts
                .iter()
                .map(|text| text.galley.text())
                .collect::<Vec<_>>()
        );
        assert!(
            !painted.rects_with(theme::CANDY_GROOVE_FILL).is_empty(),
            "the download drew no groove to fill"
        );
        assert!(painted.ink("Cancel").is_some(), "no way out of the job");
    }

    /// An installed update keeps the box until the restart takes it: it
    /// waits across that restart anyway, and forgetting to say so is how an
    /// update ends up installed and unrun for weeks.
    #[test]
    fn an_installed_update_asks_to_be_restarted() {
        let mut app = app_with_a_bare_bar();
        app.set_install(
            Some("v9.9.9"),
            Some(crate::update::Install::Installed {
                version: "v9.9.9".to_owned(),
            }),
        );

        let painted = paint_bar(&mut app);

        assert!(painted.ink("v9.9.9 is installed").is_some());
        assert!(painted.ink("Restart to run it").is_some());
    }

    #[test]
    fn the_queue_popover_draws_over_the_bar_while_it_is_open() {
        let mut app = app();
        app.toggle_queue();

        egui::__run_test_ui(|ui| super::bar(&mut app, ui));

        assert!(app.queue_open(), "the popover stays up until it is closed");
    }

    /// The queue popover's title strip is the status bar's own gray, in both
    /// themes, so the overlay reads as the bar at the foot of the window rather
    /// than as a second surface of its own.
    ///
    /// Read off the frame rather than off the closure: egui fills a window's
    /// title bar from `widgets.open.weak_bg_fill`, so the assertion is that a
    /// full-popup-width strip in that fill is on screen. A test of the constant
    /// alone would pass while the strip kept egui's own shade.
    #[test]
    fn the_queue_popover_carries_the_status_bars_colour_across_its_top() {
        for mode in [
            crate::config::ThemeMode::Dark,
            crate::config::ThemeMode::Light,
        ] {
            let mut app = app();
            app.toggle_queue();

            let ctx = egui::Context::default();
            crate::ui::theme::apply(&ctx, mode);
            let mut painted = Painted::default();

            // Enough frames for the popup's fade-in to finish: a floating area
            // has to measure itself before it paints, and egui multiplies the
            // opacity while it fades, so a strip read off the first few frames
            // is the right colour at a fraction of its alpha.
            for _ in 0..40 {
                let mut run = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1180.0, 808.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        super::queue_popup(
                            &mut app,
                            ui,
                            egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(1180.0, theme::PLAYBACK_BAR_HEIGHT),
                            ),
                        );
                    },
                );
                run.textures_delta.clear();
                painted = Painted::default();
                for clipped in &run.shapes {
                    painted.collect(&clipped.shape);
                }
            }

            let strips = painted.rects_with(theme::STATUS_BAR_FILL);
            assert!(
                !strips.is_empty(),
                "{mode:?}: the popover painted no strip in the status bar's gray: {:?}",
                painted.rects
            );

            // A strip, not a square: wide enough to be the popup's top and much
            // shorter than it is wide. Anything taller is a surface, not the
            // title bar.
            assert!(
                strips
                    .iter()
                    .any(|rect| rect.width() > 100.0 && rect.height() < rect.width() / 3.0),
                "{mode:?}: none of the gray rectangles is a strip across the top: {strips:?}"
            );
        }
    }

    /// The Library's search field rides at the bar's right end: on the bar's own
    /// line, inside the bar's height, and against the bar's right-hand edge —
    /// to the right of the cream box rather than on the heading line in the
    /// content area, which is where it used to be.
    #[test]
    fn the_search_field_sits_at_the_right_of_the_bar() {
        let mut app = app();
        let painted = paint_bar(&mut app);

        // The bar's own inner right edge: its 16-point frame margin is what the
        // field's trailing edge is placed against.
        let field = painted
            .ink("Search…")
            .expect("the search field is on the bar");
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");
        let background = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| (rect.width() - SEARCH_WIDTH).abs() < 1.0)
            .max_by(|a, b| a.right().total_cmp(&b.right()))
            .expect("the field paints its own background");

        assert!(
            (background.right() - (WINDOW_WIDTH - 16.0)).abs() <= 1.0,
            "the field is not against the bar's right edge: {background:?}"
        );
        assert!(
            (background.width() - SEARCH_WIDTH).abs() < 1.0,
            "the field is not the width it is given: {background:?}"
        );
        assert_eq!(
            painted
                .rects
                .iter()
                .find(|(rect, _)| *rect == background)
                .map(|(_, colour)| *colour),
            Some(SEARCH_FILL),
            "the field is not filled white as the user asked"
        );
        assert_eq!(
            SEARCH_FILL,
            egui::Color32::WHITE,
            "the field's fill is no longer white"
        );
        assert!(
            background.left() > pill.right(),
            "the field is not clear of the centre box: {background:?} vs {pill:?}"
        );
        assert!(
            field.top() >= 0.0 && field.bottom() <= theme::PLAYBACK_BAR_HEIGHT,
            "the field is not on the bar's own line: {field:?}"
        );
        assert!(
            (background.center().y - pill.center().y).abs() <= 2.0,
            "the field is not level with the box it sits beside: {background:?} vs {pill:?}"
        );
    }

    #[test]
    fn the_search_field_is_only_on_the_library() {
        let mut app = app();
        app.set_section(crate::ui::navigation::NavSection::Playlists);

        let painted = paint_bar(&mut app);

        assert!(
            painted.ink("Search…").is_none(),
            "the bar offers a search field outside the Library"
        );
    }

    /// A query puts a Clear button beside the field, and clicking it empties the
    /// query: the way back out of a filter is the field that set it.
    #[test]
    fn a_query_puts_a_clear_button_beside_the_field() {
        let mut app = app();
        app.browse_mut().set_query_text("xtal");

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
        let clear = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| (rect.width() - SEARCH_CLEAR_WIDTH).abs() < 1.0)
            .max_by(|a, b| a.right().total_cmp(&b.right()))
            .expect("the Clear button paints its own room");
        let field = painted
            .rects
            .iter()
            .map(|(rect, _)| *rect)
            .filter(|rect| (rect.width() - SEARCH_WIDTH).abs() < 1.0)
            .max_by(|a, b| a.right().total_cmp(&b.right()))
            .expect("the field paints its own background");
        let pill = painted
            .rects_with(theme::CENTRE_BOX_FILL)
            .into_iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the centre box paints its own fill");

        assert!(
            clear.left() > pill.right() && clear.right() <= field.left(),
            "the Clear button is not between the box and the field: \
             {clear:?}, {field:?}, {pill:?}"
        );

        // The reserved room is what the field is set off from, so the button
        // sits in it whether or not its label fills it — the way the field's
        // trailing edge stays put.
        //
        // The button is WHITE: `Button::new(..).fill(SEARCH_FILL)` overrides the
        // interactive frame unconditionally, so the room under the pointer — and
        // the click coordinate taken from its centre — sits in the same white as
        // the field beside it, and the two read as one control row.
        let pos = clear.center();
        assert!(
            painted.ink("Clear").is_some(),
            "the button does not draw its own label"
        );
        assert!(
            painted.coloured("Clear", theme::PLAYBACK_BAR_INK).is_some(),
            "the Clear button's label is not in the bar's ink, which is what \
             reads on the white fill it is drawn in"
        );
        assert!(
            painted
                .rects
                .iter()
                .any(|(rect, colour)| *colour == SEARCH_FILL && rect.contains(pos)),
            "the button is not drawn as one of the bar's controls: {pos:?}"
        );
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
            app.browse().query_text().is_empty() && !app.browse().is_filtered(),
            "clicking Clear at {pos:?} left the query at {:?}",
            app.browse().query_text()
        );
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

    /// An application with a track on the bar, stopped rather than playing.
    ///
    /// The centre box is drawn only while there is a track there, so the tests
    /// that measure it — and the tests that place the Library's pages and search
    /// field against its pill — need a bar that carries one. Opened and then
    /// stopped so the transport still shows its resting "▶" and not "⏸".
    fn app_with_a_loaded_track() -> MusicCraftApp {
        let mut app = app();
        app.open_path(Path::new("/music/song.mp3"));
        app.stop_playback();
        app
    }
}
