//! The status bar across the bottom of the window.
//!
//! One line saying how much music the library holds — songs, albums and artists
//! — centred on a gray strip that spans the window, with a control in each
//! corner: a `+` at the left that names a new playlist, and a gear at the right
//! that opens Settings. The bar reports, and those are the two things it holds;
//! what is playing is the transport's business at the top of the window, and
//! what can be done to a selection is the library's.
//!
//! Both arrived here the same way, leaving a section that had better things to
//! do with its rows: Settings had a sidebar entry of its own, and making a
//! playlist had a button across the top of the Playlists section, in the way of
//! the list it was making. Neither is a section — you reach for them from
//! anywhere — so they sit in the corners, out of the way of the music.

use egui::{Button, Panel, Rect, RichText};

use crate::app::MusicCraftApp;
use crate::playlists::PlaylistEditKind;
use crate::ui::navigation::NavSection;
use crate::ui::theme;

/// The gear drawn at the right end of the bar, opening Settings.
const GEAR_GLYPH: &str = "⚙";

/// The `+` drawn at the left end of the bar, naming a new playlist.
const PLUS_GLYPH: &str = "+";

/// Which end of the bar a corner button belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// The `+` that makes a playlist.
    Left,
    /// The gear that opens Settings.
    Right,
}

/// Draw the status bar.
pub fn bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    Panel::bottom("musiccraft_status_bar")
        .resizable(false)
        .exact_size(theme::STATUS_BAR_HEIGHT)
        .frame(theme::status_bar_frame())
        .show(ui, |ui| {
            ui.scope(|ui| {
                // The bar paints its own fill in every theme, so its line and
                // its buttons take the bar's ink rather than the theme's
                // foreground, which is near-white in the dark theme and would
                // vanish on the gray.
                theme::apply_status_bar_visuals(&mut ui.style_mut().visuals);

                // Neither corner button pads its glyph. The bar is only as tall
                // as what it holds, so at the size the glyphs are drawn,
                // padding alone is enough to push a button -- and with it the
                // bar's own fill -- past the foot of the window. The gray a
                // button leaves around its glyph comes from
                // [`theme::STATUS_BAR_BUTTON_SIZE`], not from its padding.
                ui.style_mut().spacing.button_padding = egui::Vec2::ZERO;

                // The line first, so the buttons draw over it rather than
                // under: they claim their own rectangles, which the line's
                // centred layout knows nothing about.
                counts(app, ui);
                new_playlist(app, ui);
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
    // stays centred in the window rather than in what the corner buttons leave
    // over, so neither button is placed by taking a row.
    ui.centered_and_justified(|ui| {
        ui.label(RichText::new(line).small().weak());
    });
}

/// The `+` at the left end of the bar, which names a new playlist.
///
/// It opens the same name editor the Playlists section used to raise, and asks
/// the library for the same thing; what moved is where the control sits, not
/// what it does. It carries no highlight, unlike the gear: a new playlist is
/// something you ask for, not somewhere you are.
fn new_playlist(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let response = ui.put(
        corner_rect(ui.max_rect(), End::Left),
        Button::new(RichText::new(PLUS_GLYPH).size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE)),
    );

    if response
        .on_hover_text("Name a playlist, then add tracks to it")
        .clicked()
    {
        app.start_playlist_edit(PlaylistEditKind::Create);
    }
}

/// The gear at the right end of the bar, which opens Settings.
///
/// It carries the highlight while Settings is open, the same way the sidebar
/// entries did — the gear is now the only place that shows where you are when
/// you are there.
///
/// Both glyphs are drawn at [`theme::STATUS_BAR_BUTTON_GLYPH_SIZE`] rather than
/// at the bar's line size: each is the only thing in its corner to aim at, so
/// each is drawn as tall as the bar has room for.
fn settings(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let response = ui.put(
        corner_rect(ui.max_rect(), End::Right),
        Button::selectable(
            app.section() == NavSection::Settings,
            RichText::new(GEAR_GLYPH).size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE),
        ),
    );

    if response
        .on_hover_text(NavSection::Settings.label())
        .clicked()
    {
        app.set_section(NavSection::Settings);
    }
}

/// Where a corner button sits on a bar filling `bar`: a square tucked into one
/// end, inset from it, and centred on the bar's own vertical centre.
fn corner_rect(bar: Rect, end: End) -> Rect {
    let side = theme::STATUS_BAR_BUTTON_SIZE;
    let top = bar.center().y - side / 2.0;
    let (left, right) = match end {
        End::Left => {
            let left = bar.left() + theme::STATUS_BAR_BUTTON_INSET;
            (left, left + side)
        }
        End::Right => {
            let right = bar.right() - theme::STATUS_BAR_BUTTON_INSET;
            (right - side, right)
        }
    };

    Rect::from_min_max(
        egui::pos2(left, top),
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
        let gear = corner_rect(bar, End::Right);

        assert_eq!(
            bar.right() - gear.right(),
            theme::STATUS_BAR_BUTTON_INSET,
            "the gear is inset from the right end"
        );
        assert_eq!(
            gear.right(),
            gear.left() + theme::STATUS_BAR_BUTTON_SIZE,
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
    fn the_plus_sits_in_the_left_corner_of_the_bar() {
        let bar = Rect::from_min_max(egui::pos2(0.0, 970.0), egui::pos2(1180.0, 1000.0));
        let plus = corner_rect(bar, End::Left);
        let gear = corner_rect(bar, End::Right);

        assert_eq!(
            plus.left() - bar.left(),
            theme::STATUS_BAR_BUTTON_INSET,
            "the + is inset from the left end"
        );
        assert_eq!(
            plus.right(),
            plus.left() + theme::STATUS_BAR_BUTTON_SIZE,
            "and square"
        );
        assert_eq!(
            plus.size(),
            gear.size(),
            "the two corner buttons are the same size: {plus:?} vs {gear:?}"
        );
        assert_eq!(
            plus.center().y,
            bar.center().y,
            "centred on the bar, level with the gear at the other end"
        );
        assert!(
            bar.contains_rect(plus),
            "the + stays on the bar: {plus:?} is not inside {bar:?}"
        );
        assert!(
            plus.right() < gear.left(),
            "the corners do not meet: {plus:?} overlaps {gear:?}"
        );
    }

    #[test]
    fn the_gear_is_a_gear_the_fonts_can_draw() {
        assert_eq!(GEAR_GLYPH, "⚙");

        // At the size it is actually drawn: a glyph the proportional face can
        // only supply at the bar's line size would come out blank.
        let size = theme::STATUS_BAR_BUTTON_GLYPH_SIZE;

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
    fn the_plus_is_a_plus_the_fonts_can_draw() {
        assert_eq!(PLUS_GLYPH, "+");

        let size = theme::STATUS_BAR_BUTTON_GLYPH_SIZE;
        let ctx = egui::Context::default();
        let mut drawable = false;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            drawable = ui
                .ctx()
                .fonts_mut(|fonts| fonts.has_glyphs(&egui::FontId::proportional(size), PLUS_GLYPH));
        });
        output.drop_without_applying_deltas();

        assert!(drawable, "none of egui's fonts can draw {PLUS_GLYPH}");
    }

    #[test]
    fn the_corner_glyphs_stand_out_from_the_bar_line() {
        // The counts are drawn `.small`; the corner glyphs must not be, or the
        // bar's controls read as more words on the bar. Resolved through the
        // app's own style, so this reads the size the bar actually draws at.
        let ctx = egui::Context::default();
        theme::apply(&ctx, crate::config::ThemeMode::Light);
        let line = egui::TextStyle::Small.resolve(&ctx.style_of(egui::Theme::Light));

        assert!(
            theme::STATUS_BAR_BUTTON_GLYPH_SIZE > line.size,
            "the corner glyphs ({}) are no bigger than the bar's line ({})",
            theme::STATUS_BAR_BUTTON_GLYPH_SIZE,
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

    /// Each corner button paints on the bar, at its own end, bigger than the
    /// line between them.
    #[test]
    fn the_corner_buttons_sit_at_their_ends_and_read_bigger_than_the_counts() {
        let screen = screen();
        let shapes = painted(screen);
        let bar = filled(&shapes, theme::STATUS_BAR_FILL).expect("the bar never painted its fill");
        let counts = written(&shapes, "songs").expect("the bar says nothing");
        let gear = written(&shapes, "⚙").expect("the bar has no gear");
        let plus = written(&shapes, "+").expect("the bar has no +");

        for (label, button) in [("+", plus), ("gear", gear)] {
            assert!(
                bar.contains_rect(button),
                "the {label} is off the bar: {button:?} inside {bar:?}"
            );
            assert!(
                button.height() > counts.height(),
                "the {label} ({}) does not read bigger than the counts ({}): {button:?} vs {counts:?}",
                button.height(),
                counts.height()
            );
        }

        // The `+` at the left end and the gear at the right, each within a
        // quarter of the window of its own edge, and neither one meeting the
        // other across the bar.
        assert!(
            plus.left() - screen.left() < screen.width() / 4.0,
            "the + is not in the left corner: {plus:?} on {screen:?}"
        );
        assert!(
            screen.right() - gear.right() < screen.width() / 4.0,
            "the gear is not in the right corner: {gear:?} on {screen:?}"
        );
        assert!(
            plus.right() < gear.left(),
            "the two corner buttons meet: {plus:?} overlaps {gear:?}"
        );

        // The gear's own move out of the sidebar replaced a 19-point glyph,
        // which drew a 22-point box. It still has to stand taller than that, or
        // having moved it bought nothing.
        assert!(
            gear.height() > 22.0,
            "the gear reads {} points, no bigger than the 22 it replaced: {gear:?}",
            gear.height()
        );
    }

    /// Clicking the `+` raises the name editor; clicking nothing does not.
    #[test]
    fn a_click_on_the_plus_starts_a_new_playlist() {
        let screen = screen();
        let mut app = app();
        let ctx = egui::Context::default();

        // Frame 1: lay the bar out and find the `+` the way a pointer would.
        let first = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| crate::ui::render(&mut app, ui),
        );
        let plus = written(&first.shapes, "+").expect("the bar has no +");
        first.drop_without_applying_deltas();

        assert!(
            app.playlist_draft().is_none(),
            "the + opened an editor before it was clicked"
        );

        // Frame 2: press and release on it.
        let centre = plus.center();
        let press = |pressed: bool| {
            vec![egui::Event::PointerButton {
                pos: centre,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }]
        };
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        input.events = vec![egui::Event::PointerMoved(centre)];
        input.events.extend(press(true));
        let down = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
        down.drop_without_applying_deltas();

        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        input.events = press(false);
        let up = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
        up.drop_without_applying_deltas();

        let draft = app
            .playlist_draft()
            .expect("clicking the + did not raise the name editor");
        assert_eq!(
            draft.kind(),
            PlaylistEditKind::Create,
            "the + opened something other than a new playlist"
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
