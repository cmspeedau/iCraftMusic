//! The status bar across the bottom of the window.
//!
//! One line saying how much music the library holds — songs, albums and artists
//! — centred on a gray strip that spans the window, and a gear at the right end
//! that opens Settings. The bar reports and it holds that one control: what is
//! playing is the transport's business at the top of the window, what can be
//! done to a selection is the library's, and the sections you browse are the
//! sidebar's. Settings had a sidebar entry of its own; it is here now, in the
//! corner, out of the way of the sections that hold the music.

use egui::{Button, Panel, Rect, Response, RichText};

use crate::app::MusicCraftApp;
use crate::ui::navigation::NavSection;
use crate::ui::theme;

/// The gear drawn at the right end of the bar.
const GEAR_GLYPH: &str = "⚙";

/// Draw the status bar.
pub fn bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    Panel::bottom("musiccraft_status_bar")
        .resizable(false)
        .exact_size(theme::STATUS_BAR_HEIGHT)
        .frame(theme::status_bar_frame())
        .show(ui, |ui| {
            ui.scope(|ui| {
                // The bar paints its own fill in every theme, so its line and
                // its button take the bar's ink rather than the theme's
                // foreground, which is near-white in the dark theme and would
                // vanish on the gray.
                theme::apply_status_bar_visuals(&mut ui.style_mut().visuals);
                counts(app, ui);
                settings(app, ui);
            });
        });
}

/// The bar's one line, centred in it.
fn counts(app: &MusicCraftApp, ui: &mut egui::Ui) {
    let browse = app.browse();
    let line = line_text(
        browse.track_count(),
        browse.album_count(),
        browse.artist_count(),
    );

    // The bar is taller than its text in either direction, so the line is laid
    // out along both axes of the whole bar rather than from its top left. It
    // stays centred in the window rather than in what the gear leaves over, so
    // the gear is placed by its rectangle instead of taking a row.
    ui.centered_and_justified(|ui| {
        ui.label(RichText::new(line).small().weak());
    });
}

/// The gear at the right end of the bar, which opens Settings.
///
/// It carries the highlight while Settings is open, the same way the sidebar
/// entries did — the gear is now the only place that shows where you are when
/// you are there.
///
/// The gear is drawn at [`theme::STATUS_BAR_GEAR_GLYPH_SIZE`] rather than at
/// the bar's line size: it is the only thing in the corner to aim at, so it
/// fills the button it sits in.
fn settings(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let rect = gear_rect(ui.max_rect());
    let response: Response = ui.put(
        rect,
        Button::selectable(
            app.section() == NavSection::Settings,
            RichText::new(GEAR_GLYPH).size(theme::STATUS_BAR_GEAR_GLYPH_SIZE),
        ),
    );

    if response
        .on_hover_text(NavSection::Settings.label())
        .clicked()
    {
        app.set_section(NavSection::Settings);
    }
}

/// Where the gear sits on a bar filling `bar`: a square tucked into the right
/// end, centred on the bar's own vertical centre.
fn gear_rect(bar: Rect) -> Rect {
    let side = theme::STATUS_BAR_GEAR_SIZE;
    let right = bar.right() - theme::STATUS_BAR_GEAR_INSET;

    Rect::from_min_max(
        egui::pos2(right - side, bar.center().y - side / 2.0),
        egui::pos2(right, bar.center().y + side / 2.0),
    )
}

/// What the bar says: `18 songs · 5 albums · 4 artists`.
fn line_text(tracks: usize, albums: usize, artists: usize) -> String {
    [(tracks, "song"), (albums, "album"), (artists, "artist")]
        .into_iter()
        .map(|(count, name)| format!("{count} {name}{}", if count == 1 { "" } else { "s" }))
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};

    #[test]
    fn the_line_counts_songs_albums_and_artists_in_order() {
        assert_eq!(line_text(0, 0, 0), "0 songs · 0 albums · 0 artists");
        assert_eq!(line_text(18, 5, 4), "18 songs · 5 albums · 4 artists");
    }

    #[test]
    fn the_line_says_song_when_there_is_only_one() {
        assert_eq!(line_text(1, 1, 1), "1 song · 1 album · 1 artist");
    }

    #[test]
    fn drawing_the_bar_is_harmless_when_the_library_is_empty() {
        let mut app = app();

        egui::__run_test_ui(|ui| bar(&mut app, ui));

        assert_eq!(
            app.section(),
            NavSection::Library,
            "drawing is not clicking: the gear leaves the section alone"
        );
    }

    #[test]
    fn the_gear_sits_in_the_right_corner_of_the_bar() {
        let bar = Rect::from_min_max(egui::pos2(0.0, 970.0), egui::pos2(1180.0, 1000.0));
        let gear = gear_rect(bar);

        assert_eq!(
            bar.right() - gear.right(),
            theme::STATUS_BAR_GEAR_INSET,
            "the gear is inset from the right end"
        );
        assert_eq!(
            gear.right(),
            gear.left() + theme::STATUS_BAR_GEAR_SIZE,
            "and square"
        );
        assert_eq!(
            gear.center().y,
            bar.center().y,
            "centred on the bar, not on its top or bottom edge"
        );
        assert!(
            bar.contains_rect(gear),
            "the gear stays on the bar: {gear:?} is not inside {bar:?}"
        );
    }

    #[test]
    fn the_gear_is_a_gear_the_fonts_can_draw() {
        assert_eq!(GEAR_GLYPH, "⚙");

        // At the size it is actually drawn: a glyph the proportional face can
        // only supply at the bar's line size would come out blank.
        let size = theme::STATUS_BAR_GEAR_GLYPH_SIZE;

        // Not through `__run_test_ui`: that starts a context with no fonts at
        // all, which would make this pass for the wrong reason.
        let ctx = egui::Context::default();
        let mut drawable = false;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            drawable = ui
                .ctx()
                .fonts_mut(|fonts| fonts.has_glyphs(&egui::FontId::proportional(size), GEAR_GLYPH));
        });
        output.drop_without_applying_deltas();

        assert!(drawable, "none of egui's fonts can draw {GEAR_GLYPH}");
    }

    #[test]
    fn the_gear_stands_out_from_the_bar_line() {
        // The counts are drawn `.small`; the gear must not be, or the bar's
        // only control reads as one more word on the bar.
        let line = egui::TextStyle::Small.resolve(&egui::Style::default());

        assert!(
            theme::STATUS_BAR_GEAR_GLYPH_SIZE > line.size,
            "the gear ({}) is no bigger than the bar's line ({})",
            theme::STATUS_BAR_GEAR_GLYPH_SIZE,
            line.size
        );
    }

    /// An application with an empty library and no output device.
    fn app() -> MusicCraftApp {
        MusicCraftApp::with_playback(
            Startup {
                config: crate::config::AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
        )
    }
}
