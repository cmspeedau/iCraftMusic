//! Visual constants and style setup for the MusicCraft shell.
//!
//! Stage 1 only needs a small, mostly flat look: a sidebar, a content area and
//! a playback bar across the top. Everything here is shared by the views so the
//! shell stays visually consistent and the spacing is defined in exactly one
//! place.

use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Frame, Margin, Stroke, Style, TextStyle,
    ThemePreference, Visuals, vec2,
};

use crate::config::ThemeMode;

/// Width of the left-hand navigation sidebar, in points.
pub const SIDEBAR_WIDTH: f32 = 208.0;

/// Height of the playback bar across the top of the window, in points.
pub const PLAYBACK_BAR_HEIGHT: f32 = 92.0;

/// Height of a single sidebar entry, in points.
pub const NAV_ITEM_HEIGHT: f32 = 28.0;

/// Height of one playlist in the sidebar's list, in points.
///
/// A playlist name is the whole of what a list entry says, so it can be a
/// little shorter than one of the browsing entries above it without crowding.
pub const PLAYLIST_ITEM_HEIGHT: f32 = 24.0;

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

/// Background colour of the navigation sidebar.
///
/// A light blue keeps navigation visually separate from the content area without
/// competing with [`ACCENT`], the colour of the selected entry. It is
/// deliberately not taken from the active theme: the sidebar is a light surface
/// in both the light and the dark theme.
pub const SIDEBAR_FILL: Color32 = Color32::from_rgb(214, 233, 250);

/// Label colour of the navigation sidebar.
///
/// The sidebar is a light surface, but the theme's own foreground is near-white
/// in the dark theme and would vanish on [`SIDEBAR_FILL`], so the sidebar
/// carries its own dark ink in every theme.
pub const SIDEBAR_INK: Color32 = Color32::from_rgb(26, 45, 66);

/// Muted [`SIDEBAR_INK`], for the sidebar's group heading.
pub const SIDEBAR_INK_MUTED: Color32 = Color32::from_rgb(70, 92, 116);

/// Fill behind a sidebar entry the pointer is over.
pub const SIDEBAR_HOVER: Color32 = Color32::from_rgb(170, 203, 234);

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
/// The theme's own button fill would put a light tile on the bar, so the bar's
/// controls come in its own grays: the whole ramp is the status bar's gray,
/// a step darker at each state, so a control still marks itself out on a bar
/// that is now that same gray, and the outline deeper again.
pub const PLAYBACK_BAR_CONTROL: Color32 = Color32::from_rgb(156, 158, 163);

/// Fill of a control on the playback bar the pointer is over.
pub const PLAYBACK_BAR_CONTROL_HOVER: Color32 = Color32::from_rgb(130, 132, 137);

/// Fill of a control on the playback bar that is held or selected.
///
/// Doubles as the seek bar's rail, which is the same shade of gray and reads as
/// the ungrooved part of the track.
pub const PLAYBACK_BAR_CONTROL_ACTIVE: Color32 = Color32::from_rgb(110, 112, 118);

/// Ink of the transport controls on the playback bar: white, at the user's
/// request, where the rest of the bar's controls take [`PLAYBACK_BAR_INK`].
///
/// The bar is a mid-gray in every theme and this is the one colour that reads on
/// it in both, so it is its own constant rather than the theme's foreground —
/// which is near-white in the dark theme and would leave the light theme's
/// transport invisible.
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

/// Install the MusicCraft style for `mode`.
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

/// Apply the MusicCraft spacing and rounding to a single style.
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

/// Frame used by the navigation sidebar.
///
/// It fills with [`SIDEBAR_FILL`] rather than the theme's panel fill, so it
/// takes no style.
pub fn sidebar_frame() -> Frame {
    Frame::new()
        .inner_margin(Margin::symmetric(8, 10))
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
pub fn apply_sidebar_visuals(visuals: &mut Visuals) {
    ink_surface(visuals, SIDEBAR_INK, SIDEBAR_INK_MUTED);

    // A hovered entry needs a frame that is visible against the blue fill.
    for widget in [&mut visuals.widgets.hovered, &mut visuals.widgets.active] {
        widget.bg_fill = SIDEBAR_HOVER;
        widget.weak_bg_fill = SIDEBAR_HOVER;
    }
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
    visuals.widgets.inactive.weak_bg_fill = PLAYBACK_BAR_CONTROL;
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
        assert!(luminance(SIDEBAR_FILL) > 0.6, "the sidebar must stay light");
    }

    #[test]
    fn the_sidebar_labels_stay_readable_on_the_sidebar() {
        for ink in [SIDEBAR_INK, SIDEBAR_INK_MUTED] {
            assert!(
                contrast(ink, SIDEBAR_FILL) >= 4.5,
                "sidebar ink {ink:?} is too faint on {SIDEBAR_FILL:?}"
            );
        }
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
        assert_eq!(
            visuals.selection.stroke.color,
            Color32::WHITE,
            "the selected entry is filled with ACCENT, so its text must be white"
        );
    }

    #[test]
    fn a_hovered_entry_shows_itself_against_the_sidebar() {
        let mut visuals = Visuals::dark();
        apply_sidebar_visuals(&mut visuals);

        assert_ne!(visuals.widgets.hovered.bg_fill, SIDEBAR_FILL);
        assert!(
            contrast(visuals.widgets.hovered.bg_fill, SIDEBAR_FILL) > 1.2,
            "the hover fill must be distinguishable from the sidebar"
        );
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

    /// A control on the playback bar has to stay visible where the bar is now
    /// the status bar's own gray: at rest it is a step from that gray, under
    /// the pointer a further step, and the outline deeper again. The floors are
    /// the ratios this palette actually produces, so a later nudge toward the
    /// bar's own colour fails rather than sliding under the eye.
    #[test]
    fn the_playback_bars_controls_still_show_against_its_own_fill() {
        let ramp = [
            ("resting", PLAYBACK_BAR_CONTROL, 1.2),
            ("hovered", PLAYBACK_BAR_CONTROL_HOVER, 1.5),
            ("active", PLAYBACK_BAR_CONTROL_ACTIVE, 1.8),
            ("edge", PLAYBACK_BAR_CONTROL_EDGE, 2.5),
        ];
        for (state, fill, at_least) in ramp {
            assert!(
                contrast(fill, PLAYBACK_BAR_FILL) >= at_least,
                "the {state} fill {fill:?} does not show on {PLAYBACK_BAR_FILL:?}: \
                 {:.2} against {at_least}",
                contrast(fill, PLAYBACK_BAR_FILL)
            );
        }

        // The white the transport is painted in has to READ on the bar's fill
        // and on the control fill under its hover. The floor is the ratio this
        // palette actually produces and it is a LOW one: white on a mid-gray bar
        // is about 2.1:1, which is the price of the user's request. What the
        // assertion is for is the direction — a later nudge of the bar's fill
        // toward white would take it to nothing, and that fails here.
        for (fill, at_least) in [(PLAYBACK_BAR_FILL, 2.0), (PLAYBACK_BAR_CONTROL_HOVER, 2.5)] {
            assert!(
                contrast(PLAYBACK_BAR_CONTROL_INK, fill) >= at_least,
                "the transport's white ink is too faint on {fill:?}: {:.2}",
                contrast(PLAYBACK_BAR_CONTROL_INK, fill)
            );
        }

        // And the bar's ink still reads on any of those fills, since the labels
        // and the seek bar are drawn in it over them.
        for fill in [
            PLAYBACK_BAR_CONTROL,
            PLAYBACK_BAR_CONTROL_HOVER,
            PLAYBACK_BAR_CONTROL_ACTIVE,
        ] {
            assert!(
                contrast(PLAYBACK_BAR_INK, fill) >= 3.0,
                "the bar's ink is too faint on the control fill {fill:?}: {:.2}",
                contrast(PLAYBACK_BAR_INK, fill)
            );
        }
    }

    /// The search field is a control on the bar rather than a field on a page,
    /// so it takes the bar's own palette: a fill of one of the bar's control
    /// shades and, in the bar's ink, the query and the caret. The theme's
    /// text-edit fill is near-black in the dark theme, which would put the bar's
    /// dark ink on a black tile, and egui's own caret is the theme's, which
    /// would vanish on the field's light fill.
    #[test]
    fn the_search_field_wears_the_playback_bars_own_controls() {
        let mut visuals = Visuals::dark();
        apply_playback_bar_visuals(&mut visuals);

        assert_eq!(
            visuals.text_cursor.stroke.color, PLAYBACK_BAR_INK,
            "the caret would not show on the fill the field is drawn on"
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
}
