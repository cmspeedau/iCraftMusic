//! Visual constants and style setup for the MusicCraft shell.
//!
//! Stage 1 only needs a small, mostly flat look: a sidebar, a content area and
//! a bottom playback bar. Everything here is shared by the views so the shell
//! stays visually consistent and the spacing is defined in exactly one place.

use egui::{Color32, Context, CornerRadius, Frame, Margin, Style, ThemePreference, vec2};

use crate::config::ThemeMode;

/// Width of the left-hand navigation sidebar, in points.
pub const SIDEBAR_WIDTH: f32 = 208.0;

/// Height of the bottom playback bar, in points.
pub const PLAYBACK_BAR_HEIGHT: f32 = 92.0;

/// Height of a single sidebar entry, in points.
pub const NAV_ITEM_HEIGHT: f32 = 28.0;

/// Accent colour used for selections and links in both themes.
pub const ACCENT: Color32 = Color32::from_rgb(10, 132, 255);

/// Colour used for non-fatal warnings, such as a settings file that failed to
/// load.
pub const WARNING: Color32 = Color32::from_rgb(255, 159, 10);

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

/// Frame used by the header strip across the top of the window.
pub fn top_bar_frame(style: &Style) -> Frame {
    Frame::new()
        .inner_margin(Margin::symmetric(12, 8))
        .fill(style.visuals.panel_fill)
}

/// Frame used by the navigation sidebar.
pub fn sidebar_frame(style: &Style) -> Frame {
    Frame::new()
        .inner_margin(Margin::symmetric(8, 10))
        .fill(style.visuals.panel_fill)
}

/// Frame used by the bottom playback bar.
///
/// It is deliberately darker than the panels so the transport reads as a
/// separate strip rather than as part of the content area.
pub fn playback_bar_frame(style: &Style) -> Frame {
    Frame::new()
        .inner_margin(Margin::symmetric(16, 10))
        .fill(style.visuals.extreme_bg_color)
}
