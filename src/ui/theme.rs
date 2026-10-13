//! Visual constants and style setup for the MusicCraft shell.
//!
//! Stage 1 only needs a small, mostly flat look: a sidebar, a content area and
//! a playback bar across the top. Everything here is shared by the views so the
//! shell stays visually consistent and the spacing is defined in exactly one
//! place.

use egui::{
    Color32, Context, CornerRadius, Frame, Margin, Stroke, Style, ThemePreference, Visuals, vec2,
};

use crate::config::ThemeMode;

/// Width of the left-hand navigation sidebar, in points.
pub const SIDEBAR_WIDTH: f32 = 208.0;

/// Height of the playback bar across the top of the window, in points.
pub const PLAYBACK_BAR_HEIGHT: f32 = 92.0;

/// Height of a single sidebar entry, in points.
pub const NAV_ITEM_HEIGHT: f32 = 28.0;

/// Height of the status bar across the bottom of the window, in points.
///
/// One line of text with the room to centre it vertically, plus the squares
/// holding the two corner buttons — new playlist at the left, Settings at the
/// right.
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
/// A light gray keeps the transport controls legible whatever the active theme
/// is and gives the bar its own surface, separate from the content area below
/// it. Like [`SIDEBAR_FILL`] it is deliberately not taken from the theme.
pub const PLAYBACK_BAR_FILL: Color32 = Color32::from_rgb(229, 229, 232);

/// Label colour of the playback bar.
///
/// The bar is a light surface in both themes, so it carries its own dark ink;
/// the theme's foreground is near-white in the dark theme.
pub const PLAYBACK_BAR_INK: Color32 = Color32::from_rgb(32, 32, 36);

/// Muted [`PLAYBACK_BAR_INK`], for the elapsed and total times.
pub const PLAYBACK_BAR_INK_MUTED: Color32 = Color32::from_rgb(96, 98, 104);

/// Fill of an idle control on the playback bar.
///
/// The theme's own button fill is dark and would swallow the bar's ink, so the
/// bar's buttons come in their own grays.
pub const PLAYBACK_BAR_CONTROL: Color32 = Color32::from_rgb(214, 214, 219);

/// Fill of a control on the playback bar the pointer is over.
pub const PLAYBACK_BAR_CONTROL_HOVER: Color32 = Color32::from_rgb(199, 199, 205);

/// Fill of a control on the playback bar that is held or selected.
///
/// Doubles as the seek bar's rail, which is the same shade of gray and reads as
/// the ungrooved part of the track.
pub const PLAYBACK_BAR_CONTROL_ACTIVE: Color32 = Color32::from_rgb(185, 185, 192);

/// Outline of the playback bar's controls, so they stay distinct from the bar
/// itself without needing a shadow.
pub const PLAYBACK_BAR_EDGE: Color32 = Color32::from_rgb(176, 178, 184);

/// Colour used for non-fatal warnings, such as a settings file that failed to
/// load.
pub const WARNING: Color32 = Color32::from_rgb(255, 159, 10);

/// Background colour of the status bar across the bottom of the window.
///
/// A mid gray, well darker than [`PLAYBACK_BAR_FILL`], so the two bars that
/// frame the window are never taken for one another. Like the other bars it is
/// deliberately not taken from the active theme.
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
/// bar paints a light fill in every theme, so its buttons take the bar's grays
/// instead of the theme's near-black ones.
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
        widget.bg_stroke.color = PLAYBACK_BAR_EDGE;
    }
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

    // The gear is the bar's only control, so it needs a hover frame of its own:
    // the themes' hover fill is near-black in the dark theme, which would put
    // the bar's dark ink on a dark square.
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
    fn the_playback_bar_is_light_gray() {
        let [r, g, b, a] = PLAYBACK_BAR_FILL.to_array();
        assert_eq!(a, 255, "the bar surface must be opaque");
        assert!(
            r.max(g).max(b) - r.min(g).min(b) <= 8,
            "the bar must stay a gray: {PLAYBACK_BAR_FILL:?} leans on a channel"
        );
        assert!(
            luminance(PLAYBACK_BAR_FILL) > 0.6,
            "the bar must stay light"
        );
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
        // The dark theme is the interesting case: left alone, its button fill is
        // near-black and its foreground near-white, both wrong on a light bar.
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
    fn the_status_bar_is_gray_and_distinct_from_the_playback_bar() {
        let [r, g, b, a] = STATUS_BAR_FILL.to_array();
        assert_eq!(a, 255, "the bar surface must be opaque");
        assert!(
            r.max(g).max(b) - r.min(g).min(b) <= 8,
            "the bar must read as a gray: {STATUS_BAR_FILL:?} leans on a channel"
        );
        assert!(
            contrast(STATUS_BAR_FILL, PLAYBACK_BAR_FILL) > 1.2,
            "the two bars that frame the window must not look like one surface"
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
