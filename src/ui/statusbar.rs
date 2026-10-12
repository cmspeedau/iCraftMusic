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
/// the bar's line size: it is the only thing in the corner to aim at, so it is
/// drawn as tall as the bar has room for.
fn settings(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // The bar is only as tall as what it holds, so the button must not pad the
    // glyph: at the size the gear is drawn, padding alone is enough to push the
    // button -- and with it the bar's own fill -- past the foot of the window.
    // The gray the button leaves above and below the gear comes from
    // [`theme::STATUS_BAR_GEAR_SIZE`], not from its padding.
    ui.style_mut().spacing.button_padding = egui::Vec2::ZERO;

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

    /// The shapes the whole app paints on `screen`, after the frames it takes
    /// to settle.
    fn painted(screen: egui::Rect) -> Vec<egui::epaint::ClippedShape> {
        let mut app = app();
        let ctx = egui::Context::default();
        let mut out = None;
        for _ in 0..3 {
            let input = egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            };
            let mut o = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
            o.textures_delta.clear();
            out = Some(o);
        }
        out.expect("`run_ui` ran at least once").shapes
    }

    /// The rectangle of the shape that fills itself with `fill`, and the
    /// bottom-most one wins: the bar sits under everything else.
    fn filled(shapes: &[egui::epaint::ClippedShape], fill: egui::Color32) -> Option<egui::Rect> {
        shapes
            .iter()
            .filter_map(|cs| match &cs.shape {
                egui::Shape::Rect(r) if r.fill == fill => Some(r.rect),
                _ => None,
            })
            .max_by(|a, b| a.bottom().total_cmp(&b.bottom()))
    }

    /// The rectangle a text shape's ink covers.
    ///
    /// A galley that is centred on its line is painted *from* its centre, so
    /// its shape's position is the centre of the line and its top — not its
    /// top left, which is only true of a left-aligned galley.
    fn written(shapes: &[egui::epaint::ClippedShape], needle: &str) -> Option<egui::Rect> {
        shapes.iter().find_map(|cs| match &cs.shape {
            egui::Shape::Text(t) if t.galley.text().contains(needle) => {
                let size = t.galley.size();
                let min = match t.galley.job.halign {
                    egui::Align::LEFT => t.pos,
                    egui::Align::Center => egui::pos2(t.pos.x - size.x / 2.0, t.pos.y),
                    egui::Align::RIGHT => egui::pos2(t.pos.x - size.x, t.pos.y),
                };
                Some(egui::Rect::from_min_size(min, size))
            }
            _ => None,
        })
    }

    /// A window the size of the user's own.
    fn screen() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0))
    }

    /// The bar is the foot of the window, as wide as the window, filled with
    /// the bar's own grey.
    #[test]
    fn the_bar_paints_the_foot_of_the_window_in_its_own_grey() {
        let screen = screen();
        let bar = filled(&painted(screen), theme::STATUS_BAR_FILL)
            .expect("the bar never painted its fill");

        assert_eq!(
            bar.width(),
            screen.width(),
            "the bar spans the window: {bar:?}"
        );
        assert_eq!(
            bar.bottom(),
            screen.bottom(),
            "the bar is at the foot of the window: {bar:?}"
        );
        assert!(
            bar.height() >= theme::STATUS_BAR_HEIGHT - 1.0,
            "the bar is {} points tall, not {}: {bar:?}",
            bar.height(),
            theme::STATUS_BAR_HEIGHT
        );
    }

    /// The counts line is centred on the window, whatever it says.
    #[test]
    fn the_counts_are_centred_on_the_window() {
        let screen = screen();
        let counts = written(&painted(screen), "songs").expect("the bar says nothing");

        assert!(
            (counts.center().x - screen.center().x).abs() <= 1.0,
            "the counts are not centred: {counts:?} on {screen:?}"
        );
    }

    /// The gear is drawn inside the bar's right corner and reads bigger than
    /// the line it sits next to.
    #[test]
    fn the_gear_sits_in_the_corner_and_reads_bigger_than_the_counts() {
        let screen = screen();
        let shapes = painted(screen);
        let bar = filled(&shapes, theme::STATUS_BAR_FILL).expect("the bar never painted its fill");
        let counts = written(&shapes, "songs").expect("the bar says nothing");
        let gear = written(&shapes, "⚙").expect("the bar has no gear");

        assert!(
            bar.contains_rect(gear),
            "the gear is off the bar: {gear:?} inside {bar:?}"
        );
        assert!(
            screen.right() - gear.right() < screen.width() / 4.0,
            "the gear is not in the right corner: {gear:?} on {screen:?}"
        );
        assert!(
            gear.height() > counts.height(),
            "the gear ({}) does not read bigger than the counts ({}): {gear:?} vs {counts:?}",
            gear.height(),
            counts.height()
        );
        // The move out of the sidebar replaced a 19-point glyph, which drew a
        // 22-point box. The gear drawn here has to stand taller than that, or
        // having moved it bought nothing.
        assert!(
            gear.height() > 22.0,
            "the gear reads {} points, no bigger than the 22 it replaced: {gear:?}",
            gear.height()
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
