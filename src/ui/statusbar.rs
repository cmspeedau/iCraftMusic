//! The status bar across the bottom of the window.
//!
//! One line saying how much music the library holds — songs, albums and artists
//! — centred on a gray strip that spans the window, with controls at its ends:
//! at the left the `+` that names a new playlist and, beside it, the shuffle
//! and repeat switches; at the right the gear whose menu adds music and opens
//! Settings. The bar reports, and those are what it holds; what is playing is
//! the transport's
//! business at the top of the window, and what can be done to a selection is
//! the library's.
//!
//! Each arrived here the same way, leaving a section that had better things to
//! do with its rows: Settings had a sidebar entry of its own; making a playlist
//! had a button across the top of the Playlists section, in the way of the list
//! it was making; and shuffle and repeat sat among the transport controls,
//! where each took a button's width to say a state that outlives the track.
//! None is a section — you reach for them from anywhere — so they sit on the
//! bar, out of the way of the music.

use egui::{Button, Panel, Rect, RichText};

use crate::app::{MusicCraftApp, UpdateStatus};
use crate::playlists::PlaylistEditKind;
use crate::ui::theme;
use crate::update;

/// The gear drawn at the right end of the bar, opening its menu.
const GEAR_GLYPH: &str = "⚙";

/// The plus drawn at the left end of the bar, naming a new playlist.
///
/// The heavy Greek cross rather than ASCII `+`: an ASCII plus is drawn thin and
/// narrow in the Ubuntu face egui bundles — at 22 points it inks 12 points
/// across where the gear inks 24 — so it sat in its button looking smaller and
/// lighter than the glyphs on either side of it. `✚` is the same cross, drawn
/// heavy and square, and the bundled fonts carry it.
const PLUS_GLYPH: &str = "✚";

/// The shuffle glyph, crossed out by [`strike_out`] while shuffle is off.
const SHUFFLE_GLYPH: &str = "🔀";

/// The repeat glyph: the arrowed loop, crossed out while repeat is off.
///
/// The same picture the transport's browser tab carries, and the one every
/// player uses for repeat, so the switch needs no label to be read.
const REPEAT_GLYPH: &str = "🔁";

/// The single-track repeat glyph: the loop with a 1 in it.
const REPEAT_ONE_GLYPH: &str = "🔂";

/// Which end of the bar a button belongs to, and how far along that end it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// The `+` that makes a playlist, the first control at the left.
    Left,
    /// The shuffle switch, immediately right of the `+`.
    LeftBeside,
    /// The repeat switch, immediately right of the shuffle.
    LeftThird,
    /// The gear, whose menu adds music and opens Settings.
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
                shuffle(app, ui);
                repeat(app, ui);
                settings(app, ui);
            });
        });
}

/// The bar's one line, centred in it.
///
/// The counts, always. The news of a newer release used to take this line
/// when the check found one, but a notice the whole bar revolves around is
/// one notice too many: the news now speaks from the centre box, where the
/// eye already goes to be told things, and the bar goes on counting.
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

/// The `+` at the left end of the bar, which makes a new playlist — one of
/// the two kinds.
///
/// It used to open the name editor straight away; with smart playlists there
/// are now two ways to fill a new playlist, and the bar cannot know which was
/// meant, so the click asks. The menu is the gear's own habit, one row per
/// kind. It carries no highlight, unlike the gear: a new playlist is something
/// you ask for, not somewhere you are.
fn new_playlist(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let response = ui.put(
        rect_for(ui.max_rect(), End::Left),
        Button::new(
            RichText::new(PLUS_GLYPH)
                .size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE)
                .strong(),
        ),
    );

    // The glyph is a cross to the eye and nothing at all to a screen reader;
    // the ring is the only sign Tab has reached it. Both take what the bar
    // already says about the control under the pointer.
    theme::accessible_name(ui, &response, "Make a new playlist or smart playlist");
    theme::focus_ring_if_focused(ui, &response);

    let response = response.on_hover_text("Make a new playlist or smart playlist");

    egui::Popup::menu(&response).show(|ui| {
        if ui
            .button("New playlist…")
            .on_hover_text("Name a playlist, then add tracks to it")
            .clicked()
        {
            app.start_playlist_edit(PlaylistEditKind::Create);
            ui.close();
        }

        if ui
            .button("New smart playlist…")
            .on_hover_text("Write the rules, and the playlist fills itself")
            .clicked()
        {
            app.start_smart_playlist_create();
            ui.close();
        }
    });
}

/// The gear at the right end of the bar: the menu that adds music and opens
/// Settings.
///
/// It is how music comes in from anywhere: the menu offers the file and folder
/// choosers, so adding does not wait on a page you have to find first. Clicking
/// opens the menu rather than jumping straight to Settings — the destination
/// that was the gear's only job is now one of three. The gear still carries the
/// highlight while Settings is open, the same way the sidebar entries did — the
/// gear is now the only place that shows where you are when you are there.
///
/// Every glyph on the bar is drawn at [`theme::STATUS_BAR_BUTTON_GLYPH_SIZE`]
/// rather than at the bar's line size: each is a control to aim at, so each is
/// drawn as tall as the bar has room for.
fn settings(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let rect = rect_for(ui.max_rect(), End::Right);
    let response = ui.put(
        rect,
        Button::selectable(
            app.settings_open() || app.gear_menu_open(),
            RichText::new(GEAR_GLYPH)
                .size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE)
                .strong(),
        ),
    );

    theme::accessible_name(ui, &response, "Add music or open Settings");
    theme::focus_ring_if_focused(ui, &response);

    if response
        .on_hover_text("Add music or open Settings")
        .clicked()
    {
        app.toggle_gear_menu();
    }

    if !app.gear_menu_open() {
        return;
    }

    // The queue taught the shape: an open flag in the app, a floating window
    // parked by the button that opened it. The pivot puts the window's top at
    // the gear's top and its right edge at the gear's left, so the menu opens
    // up and in from the corner instead of off the bottom of the window.
    let mut open = true;

    egui::Window::new("Add music")
        .collapsible(false)
        .resizable(false)
        .fixed_pos(rect.left_top())
        .pivot(egui::Align2::RIGHT_TOP)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.set_min_width(190.0);

            if ui.button(RichText::new("Add files…").strong()).clicked() {
                app.close_gear_menu();
                app.add_files();
            }
            if ui.button(RichText::new("Add folder…").strong()).clicked() {
                app.close_gear_menu();
                app.add_folder();
            }

            ui.separator();

            update_item(app, ui);

            ui.separator();

            if ui.button("Equalizer").clicked() {
                app.close_gear_menu();
                app.toggle_equalizer();
            }
            if ui.button("Settings").clicked() {
                app.close_gear_menu();
                app.open_settings();
            }
        });

    if !open {
        app.close_gear_menu();
    }
}

/// The menu's one update row: whatever the updater knows says what the item
/// says, and pressing it does the one thing that state has left to do.
///
/// It is always on the menu — "Check for updates…" when nothing is known —
/// because the gear is where a user who wonders comes to be told; and it is
/// never a dead row. An update the app cannot install itself — a release with
/// no image for this machine, or a build that is not an AppImage and so has
/// no file to replace — opens the release page instead of failing with a
/// button, and says so in its hover text before it is pressed.
fn update_item(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    match app.update_status() {
        None => {
            if ui.button("Check for updates…").clicked() {
                app.close_gear_menu();
                app.check_for_release();
            }
        }
        Some(UpdateStatus::Checking) => {
            ui.add_enabled(
                false,
                Button::new(RichText::new("Checking for updates…").weak()),
            );
        }
        Some(UpdateStatus::Available {
            version,
            can_install,
        }) => {
            if can_install {
                let button = ui.button(RichText::new(format!("Update to {version}…")).strong());
                if button
                    .on_hover_text(
                        "downloads the new version beside this file, then swaps it in; \
                         nothing running changes until you restart",
                    )
                    .clicked()
                {
                    app.close_gear_menu();
                    app.start_update();
                }
            } else if ui
                .button(format!("See {version} on GitHub…"))
                .on_hover_text(
                    "this build cannot replace its own file, so the release page \
                     opens instead",
                )
                .clicked()
            {
                app.close_gear_menu();
                if let Some(url) = app.release_url() {
                    let url = url.to_owned();
                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                }
            }
        }
        Some(UpdateStatus::Downloading { done, total, .. }) => {
            // The centre box carries the bar and the Cancel; the menu only
            // says the job is running, so it cannot be started twice.
            let words = if total == 0 {
                "Downloading update…".to_owned()
            } else {
                format!(
                    "Downloading… {} of {}",
                    update::format_bytes(done),
                    update::format_bytes(total)
                )
            };
            ui.add_enabled(false, Button::new(RichText::new(words).weak()));
        }
        Some(UpdateStatus::Installed { version }) => {
            let button = ui.button(RichText::new(format!("Restart to run {version}…")).strong());
            if button
                .on_hover_text(
                    "the new version is installed already; this copy closes and a new one starts",
                )
                .clicked()
            {
                app.close_gear_menu();
                app.relaunch();
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        Some(UpdateStatus::Failed { reason }) => {
            let button = ui.button(RichText::new("Try the update again").strong());
            if button.on_hover_text(reason).clicked() {
                app.close_gear_menu();
                app.start_update();
            }
        }
    }
}

/// The shuffle switch, at the left end beside the `+`.
///
/// It moved off the transport, where it took a button's width in the row of
/// playback controls to say one bit, and onto the bar where it says the same
/// bit at the size of the bar's own controls. It reads as the other two do:
/// the glyph struck through while shuffle is off, plain while it is on, and
/// carrying the bar's button highlight while it is on — the slash would be
/// invisible on the accent fill.
fn shuffle(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let on = app.playback().shuffle();
    let response = ui
        .put(
            rect_for(ui.max_rect(), End::LeftBeside),
            Button::selectable(
                on,
                RichText::new(SHUFFLE_GLYPH)
                    .size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE)
                    .strong(),
            ),
        )
        .on_hover_text(shuffle_hover(on));

    // The hover words carry the on/off state; a screen reader reads them from
    // the node's name, and the ring says the keyboard is here.
    theme::accessible_name(ui, &response, shuffle_hover(on));
    theme::focus_ring_if_focused(ui, &response);

    if !on {
        strike_out(ui, response.rect);
    }

    if response.clicked() {
        app.toggle_shuffle();
    }
}

/// The repeat switch, at the left end beside the shuffle switch.
///
/// It moved off the transport for the reason shuffle did: there it was a
/// button as wide as its own words — `Repeat: All` — in a row of playback
/// controls, to say one of three states. On the bar it is a glyph the size of
/// the bar's other controls, and its hover text names the mode the way the
/// shuffle switch's names its own state.
///
/// Drawn like shuffle: struck through while repeat is off, plain while it is
/// on, and carrying the bar's button highlight while it is on. Repeat has a
/// third state and one bit of ink cannot say it, so the glyph changes as well
/// — the loop for the whole queue, the same loop with a 1 in it for the one
/// track — and the hover text spells the mode out.
fn repeat(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mode = app.playback().repeat();
    let on = mode.is_enabled();
    let response = ui
        .put(
            rect_for(ui.max_rect(), End::LeftThird),
            Button::selectable(
                on,
                RichText::new(repeat_glyph(mode))
                    .size(theme::STATUS_BAR_BUTTON_GLYPH_SIZE)
                    .strong(),
            ),
        )
        .on_hover_text(repeat_hover(mode));

    theme::accessible_name(ui, &response, repeat_hover(mode));
    theme::focus_ring_if_focused(ui, &response);

    if !on {
        strike_out(ui, response.rect);
    }

    if response.clicked() {
        app.cycle_repeat_mode();
    }
}

/// The repeat glyph for a mode: the loop, or the loop with a 1 in it.
fn repeat_glyph(mode: crate::audio::RepeatMode) -> &'static str {
    match mode {
        crate::audio::RepeatMode::One => REPEAT_ONE_GLYPH,
        _ => REPEAT_GLYPH,
    }
}

/// What the repeat switch says when it is pointed at.
fn repeat_hover(mode: crate::audio::RepeatMode) -> &'static str {
    match mode {
        crate::audio::RepeatMode::Off => "Repeat: off",
        crate::audio::RepeatMode::All => "Repeat: all",
        crate::audio::RepeatMode::One => "Repeat: this track",
    }
}

/// What the shuffle switch says when it is pointed at.
fn shuffle_hover(shuffle: bool) -> &'static str {
    if shuffle {
        "Shuffle: on"
    } else {
        "Shuffle: off"
    }
}

/// Cross the shuffle glyph out.
///
/// Painted over the button rather than typed as a combining slash: none of the
/// fonts egui bundles carry one, so a struck-through glyph would silently drop
/// the stroke and read as plain shuffle.
///
/// The bar's ink is not the glyph's: the button is drawn with the bar's ink
/// through [`theme::apply_status_bar_visuals`], but `ui.visuals().text_color()`
/// is what the painter would use, and it is set to the *theme's* foreground —
/// near-white in the dark theme, which would leave the slash invisible on the
/// gray bar.
fn strike_out(ui: &egui::Ui, rect: egui::Rect) {
    let inset = egui::vec2(rect.height() * 0.30, rect.height() * 0.26);

    ui.painter().line_segment(
        [
            rect.left_bottom() + egui::vec2(inset.x, -inset.y),
            rect.right_top() + egui::vec2(-inset.x, inset.y),
        ],
        egui::Stroke::new(1.5, theme::STATUS_BAR_INK),
    );
}

/// Where a bar button sits on a bar filling `bar`: a square tucked into one
/// end, inset from it, and centred on the bar's own vertical centre.
///
/// [`End::LeftBeside`] steps one button and one [`theme::STATUS_BAR_BUTTON_GAP`]
/// further along, so the two at the left read as a pair with a hair of the bar
/// between them.
fn rect_for(bar: Rect, end: End) -> Rect {
    let side = theme::STATUS_BAR_BUTTON_SIZE;
    let top = bar.center().y - side / 2.0;
    let (left, right) = match end {
        End::Left => {
            let left = bar.left() + theme::STATUS_BAR_BUTTON_INSET;
            (left, left + side)
        }
        End::LeftBeside => {
            let left = bar.left()
                + theme::STATUS_BAR_BUTTON_INSET
                + theme::STATUS_BAR_BUTTON_SIZE
                + theme::STATUS_BAR_BUTTON_GAP;
            (left, left + side)
        }
        End::LeftThird => {
            let left = bar.left()
                + theme::STATUS_BAR_BUTTON_INSET
                + 2.0 * (theme::STATUS_BAR_BUTTON_SIZE + theme::STATUS_BAR_BUTTON_GAP);
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
    use crate::audio::{AudioEngine, Playback, RepeatMode};
    use crate::ui::navigation::NavSection;

    #[test]
    fn the_line_counts_songs_albums_and_artists_in_order() {
        assert_eq!(line_text(0, 0, 0), "0 songs · 0 albums · 0 artists");
        assert_eq!(line_text(18, 5, 4), "18 songs · 5 albums · 4 artists");
    }

    #[test]
    fn the_line_says_song_when_there_is_only_one() {
        assert_eq!(line_text(1, 1, 1), "1 song · 1 album · 1 artist");
    }

    /// The news of a newer release used to take the bar's one line; it moved
    /// to the centre box, where things the user must read are said, and the
    /// bar went back to what a status bar is for. The notice being pending
    /// changes nothing here.
    #[test]
    fn a_pending_update_leaves_the_counting_to_the_bar() {
        let mut app = app();
        app.set_release_update(update::Update::Available {
            version: "v9.9.9".to_owned(),
            url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
            asset: None,
        });

        let lines = draw(&mut app, bar).glyphs;

        assert!(
            lines.iter().any(|line| line.contains("songs")),
            "the counts left their bar for a notice: {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("Update available")),
            "the notice still sits on the bar: {lines:?}"
        );
    }

    /// A check that came back up to date changes nothing: the counts stay, and
    /// no notice is drawn for them.
    #[test]
    fn being_up_to_date_leaves_the_counts_alone() {
        let mut app = app();
        app.set_release_update(update::Update::UpToDate);

        let lines = draw(&mut app, bar).glyphs;

        assert!(
            lines.iter().any(|line| line.contains("songs")),
            "the counts left the bar: {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("Update available")),
            "an up-to-date check drew a notice: {lines:?}"
        );
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

    /// The menu's lines, given its window frames enough to measure itself and
    /// fade in — the patience the queue popup taught — and read from the last.
    fn menu_lines(app: &mut MusicCraftApp) -> Vec<String> {
        let ctx = egui::Context::default();
        let mut lines = Vec::new();

        for _ in 0..40 {
            let mut run = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1180.0, 808.0),
                    )),
                    ..Default::default()
                },
                |ui| bar(app, ui),
            );
            run.textures_delta.clear();

            let mut drawn = Drawn::default();
            for clipped in &run.shapes {
                collect(&clipped.shape, &mut drawn);
            }
            lines = drawn.glyphs;
        }

        lines
    }

    #[test]
    fn the_gear_menu_offers_the_ways_to_add_music_and_the_ways_out() {
        let mut app = app();
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        for wanted in [
            "Add music",
            "Add files…",
            "Add folder…",
            "Equalizer",
            "Settings",
            "Check for updates…",
        ] {
            assert!(
                lines.iter().any(|line| line.contains(wanted)),
                "{wanted} did not draw: {lines:?}"
            );
        }

        assert_eq!(
            app.section(),
            NavSection::Library,
            "the menu is open, but no section has been opened"
        );
        assert!(
            !app.settings_open() && !app.equalizer_open(),
            "an open menu must not have opened a window behind itself"
        );
    }

    /// A release this copy cannot install itself — no image for it, or a
    /// build that is not an AppImage and has no file to swap — is offered as
    /// a page to open, never as an update button that could only fail.
    #[test]
    fn the_menu_offers_an_uninstallable_release_as_a_page() {
        let mut app = app();
        app.set_release_update(update::Update::Available {
            version: "v9.9.9".to_owned(),
            url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
            asset: None,
        });
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("See v9.9.9 on GitHub…")),
            "the release drew no way to see it: {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("Update to v9.9.9…")),
            "the menu promised a swap this build cannot perform: {lines:?}"
        );
    }

    /// The same release, and this copy knows its own AppImage file: the row
    /// becomes the update itself, and the pointer is put away.
    #[test]
    fn the_menu_offers_the_update_when_the_app_can_install_it() {
        let mut app = app();
        app.set_release_update(update::Update::Available {
            version: "v9.9.9".to_owned(),
            url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
            asset: Some("https://e/iCraftMusic-9.9.9-x86_64.AppImage".to_owned()),
        });
        app.set_app_image(Some(std::env::temp_dir().join("iCraftMusic-test.AppImage")));
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        assert!(
            lines.iter().any(|line| line.contains("Update to v9.9.9…")),
            "an installable release drew no update button: {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("on GitHub")),
            "the fallback vied with the real thing: {lines:?}"
        );
    }

    /// A download running says so in the menu — so it cannot be started
    /// twice — and counts the bytes it has sent to the centre box's bar.
    #[test]
    fn the_menu_says_so_while_an_update_downloads() {
        let mut app = app();
        app.set_install(
            Some("v9.9.9"),
            Some(update::Install::Running {
                done: 1024 * 1024,
                total: 4 * 1024 * 1024,
            }),
        );
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("Downloading… 1.0 MB of 4.0 MB")),
            "the running download is not on the menu: {lines:?}"
        );
    }

    /// Once the image is in, the menu's row is the restart — the installed
    /// version does not run itself, and saying "installed" without an on-ramp
    /// to running it is half a message.
    #[test]
    fn the_menu_asks_for_a_restart_when_the_update_is_in() {
        let mut app = app();
        app.set_install(
            Some("v9.9.9"),
            Some(update::Install::Installed {
                version: "v9.9.9".to_owned(),
            }),
        );
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("Restart to run v9.9.9")),
            "the installed update drew no way to run it: {lines:?}"
        );
    }

    /// While the check is on its way the menu says the plain truth and goes
    /// on with the rest of its business.
    #[test]
    fn the_menu_says_so_while_it_checks() {
        let mut app = app();
        app.set_check_pending();
        app.toggle_gear_menu();

        let lines = menu_lines(&mut app);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("Checking for updates…")),
            "the pending check drew no word of itself: {lines:?}"
        );
    }

    /// The bar is where the shuffle switch lives now, and the playback bar is
    /// where it used to be and must no longer be: a real egui pass over each,
    /// collecting what was painted, so the two cannot drift back together.
    #[test]
    fn the_shuffle_glyph_is_on_the_bar_and_no_longer_on_the_playback_bar() {
        let mut app = app();

        let on_bar = draw(&mut app, bar).glyphs;
        assert!(
            on_bar.contains(&SHUFFLE_GLYPH.to_owned()),
            "the bar does not draw the shuffle glyph: {on_bar:?}"
        );

        let on_playback = draw(&mut app, crate::ui::nowplaying::bar).glyphs;
        assert!(
            !on_playback.contains(&SHUFFLE_GLYPH.to_owned()),
            "the playback bar still draws the shuffle glyph: {on_playback:?}"
        );
    }

    /// The `+` is painted in the bar's own gray, not in a tile of its own.
    ///
    /// Measured off a real frame, in the fill it lands on the screen — and the
    /// frame this helper runs is the dark theme, whose inactive button fill is
    /// `#3C3C3C`, so matching the bar here is a real measurement rather than a
    /// coincidence of the two palettes.
    #[test]
    fn the_plus_sits_directly_on_the_bars_gray() {
        let mut app = app();
        let drawn = draw(&mut app, bar);

        let fill = drawn
            .glyph_fill(PLUS_GLYPH)
            .expect("the + was drawn on the bar");

        assert_eq!(
            fill,
            theme::STATUS_BAR_FILL,
            "the + is not painted in the bar's own fill: got {fill:?}"
        );
    }

    /// Clicking the shuffle switch turns shuffle on.
    ///
    /// A real click at the centre of the rectangle the glyph was painted into,
    /// so it fails if the switch is drawn but cannot be reached. The first pass
    /// is what a widget needs before egui will report a hit on it — the
    /// rectangle is remembered from one frame to the next — so the click lands
    /// on the second, on the same context.
    #[test]
    fn a_click_on_the_shuffle_switch_toggles_shuffle() {
        let mut app = app();
        assert!(!app.playback().shuffle(), "shuffle starts off");

        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        let mut frame = |events: Vec<egui::Event>| {
            let mut run = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ui| bar(&mut app, ui),
            );
            let mut drawn = Drawn::default();
            for clipped in &run.shapes {
                collect(&clipped.shape, &mut drawn);
            }
            run.textures_delta.clear();
            drawn
        };

        // One pass to let egui learn the switch's rectangle: a hit is decided
        // against the previous frame, so a click on the very first frame lands
        // nowhere.
        let drawn = frame(Vec::new());
        let rect = drawn
            .glyph_rect(SHUFFLE_GLYPH)
            .expect("the shuffle switch was drawn");
        let pos = rect.center();

        // Press and release, as a mouse does — egui counts a click on the
        // release, and needs the move that brought the pointer there.
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
            app.playback().shuffle(),
            "clicking the switch at {pos:?} did not turn shuffle on"
        );
    }

    /// The `+` inks as wide as the glyphs beside it.
    ///
    /// This is the assertion behind "bigger and bold", and it is measured in
    /// ink widths rather than taste: the ASCII plus inked **12.4** points across
    /// at the bar's glyph size while the gear inked **19.8**, which is what made
    /// the `+` read as the runt of the three. The heavy cross inks **19.8** —
    /// the gear's own width — so it needs no size of its own.
    ///
    /// The expected widths are literals on purpose. Comparing the plus against
    /// `GEAR_GLYPH` would compare two numbers that move together, and would pass
    /// with the ASCII plus still in place, which is exactly the shape of test
    /// that catches nothing.
    #[test]
    fn the_plus_inks_as_wide_as_the_glyphs_beside_it() {
        let ctx = egui::Context::default();
        let mut widths = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::FontId::proportional(theme::STATUS_BAR_BUTTON_GLYPH_SIZE);
            let ink = |glyph: &str| {
                ui.ctx()
                    .fonts_mut(|fonts| {
                        fonts.layout_no_wrap(glyph.to_owned(), font.clone(), egui::Color32::WHITE)
                    })
                    .size()
                    .x
            };

            widths = Some((ink(PLUS_GLYPH), ink(GEAR_GLYPH), ink("+")));
        });
        output.drop_without_applying_deltas();

        let (plus, gear, ascii) = widths.expect("the glyphs were measured");

        // Measured off this render: the thin plus 12.4, the gear 19.8.
        assert!(
            ascii < 14.0,
            "the ASCII plus is expected to be the narrow one, but measured {ascii}"
        );
        assert!(
            (gear - plus).abs() < 1.0,
            "the + ({plus}) and the gear ({gear}) do not ink the same width"
        );
        assert!(
            plus >= 19.0,
            "the + inks only {plus} across, well short of the gear's width"
        );
    }

    /// The repeat switch is on the bar now, and the transport has no repeat
    /// button: the two places cannot drift back together.
    #[test]
    fn the_repeat_controls_are_on_the_bar_and_no_longer_on_the_playback_bar() {
        let mut app = app();

        let on_bar = draw(&mut app, bar).glyphs;
        assert!(
            on_bar.contains(&REPEAT_GLYPH.to_owned()),
            "the bar does not draw the repeat glyph: {on_bar:?}"
        );

        let on_playback = draw(&mut app, crate::ui::nowplaying::bar).glyphs;
        assert!(
            !on_playback.iter().any(|text| text.contains("Repeat:")),
            "the playback bar still draws a repeat button: {on_playback:?}"
        );
    }
    /// The glyph a mode is drawn with, taken through the switch's own helper.
    #[test]
    fn the_repeat_glyph_says_which_mode_repeat_is_in() {
        assert_eq!(repeat_glyph(RepeatMode::Off), REPEAT_GLYPH);
        assert_eq!(repeat_glyph(RepeatMode::All), REPEAT_GLYPH);
        assert_eq!(
            repeat_glyph(RepeatMode::One),
            REPEAT_ONE_GLYPH,
            "the single-track mode is drawn as the whole-queue one"
        );
        assert_ne!(REPEAT_ONE_GLYPH, REPEAT_GLYPH, "and the two differ");
    }

    /// Every glyph the repeat switch can draw inks the width of the switch
    /// beside it at the size it is drawn.
    ///
    /// Measured by width rather than by `has_glyphs`: epaint reports the emoji
    /// cluster glyphs as absent even though the fallback face draws them — the
    /// shuffle switch has been shipped with one for stages — so a zero-width
    /// glyph is the signal that a face cannot supply it, and a 22-point one is
    /// the signal that it can.
    #[test]
    fn both_repeat_glyphs_ink_the_width_of_the_switch_beside_them() {
        let size = theme::STATUS_BAR_BUTTON_GLYPH_SIZE;
        let ctx = egui::Context::default();
        let mut widths = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::FontId::proportional(size);
            let width = |glyph: &str| {
                ui.ctx()
                    .fonts_mut(|fonts| {
                        fonts.layout_no_wrap(glyph.to_owned(), font.clone(), egui::Color32::WHITE)
                    })
                    .size()
                    .x
            };

            widths = Some((
                width(REPEAT_GLYPH),
                width(REPEAT_ONE_GLYPH),
                width(SHUFFLE_GLYPH),
            ));
        });
        output.drop_without_applying_deltas();

        let (whole, one, shuffle) = widths.expect("the glyphs were measured");

        // Measured off this render: all three ink 22.6 across at 22 points.
        // The single-track loop carries a digit and still matches the others,
        // which is why it can stand in the same button.
        assert!(
            (whole - one).abs() < 1.0,
            "the single-track loop ({one}) is a different width from the plain one ({whole})"
        );
        assert!(
            (whole - shuffle).abs() < 1.0,
            "the repeat loop ({whole}) is a different width from the shuffle glyph ({shuffle})"
        );
        assert!(
            whole >= 19.0,
            "the repeat glyph inks only {whole} across, short of the other controls"
        );
    }

    /// Clicking the repeat switch moves the mode on and stores the choice, the
    /// same way the transport button did: a switch that changed the engine but
    /// not the setting would come back in the old mode next launch.
    #[test]
    fn a_click_on_the_repeat_switch_cycles_and_remembers_the_mode() {
        let mut app = app();
        assert_eq!(app.playback().repeat(), RepeatMode::Off);
        assert_eq!(app.configured_repeat(), RepeatMode::Off);

        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        let mut frame = |events: Vec<egui::Event>| {
            let mut run = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ui| bar(&mut app, ui),
            );
            let mut drawn = Drawn::default();
            for clipped in &run.shapes {
                collect(&clipped.shape, &mut drawn);
            }
            run.textures_delta.clear();
            drawn
        };

        // One pass to let egui learn the switch's rectangle: a hit is decided
        // against the previous frame, so a click on the very first lands
        // nowhere.
        let drawn = frame(Vec::new());
        let rect = drawn
            .glyph_rect(REPEAT_GLYPH)
            .expect("the repeat switch was drawn");
        let pos = rect.center();

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

        assert_eq!(
            app.playback().repeat(),
            RepeatMode::All,
            "clicking the switch at {pos:?} did not move the repeat mode on"
        );
        assert_eq!(
            app.configured_repeat(),
            RepeatMode::All,
            "the mode moved but the choice was not remembered"
        );
    }

    /// The bar's other controls, their sizes and their colours are unchanged.
    #[test]
    fn the_repeat_switch_is_one_of_the_bar_s_controls() {
        let bar = Rect::from_min_max(egui::pos2(0.0, 970.0), egui::pos2(1180.0, 1000.0));
        let plus = rect_for(bar, End::Left);
        let shuffle = rect_for(bar, End::LeftBeside);
        let repeat = rect_for(bar, End::LeftThird);

        assert_eq!(
            repeat.left() - shuffle.right(),
            theme::STATUS_BAR_BUTTON_GAP,
            "the repeat switch is not set off from the shuffle switch by the \
             bar's own gap: {shuffle:?} vs {repeat:?}"
        );
        assert_eq!(
            repeat.size(),
            plus.size(),
            "the repeat switch is a different size from the +: {repeat:?} vs {plus:?}"
        );
        assert_eq!(
            repeat.center().y,
            shuffle.center().y,
            "the switches are not on the same line: {repeat:?} vs {shuffle:?}"
        );
        assert!(
            bar.contains_rect(repeat),
            "the repeat switch is off the bar: {repeat:?} is not inside {bar:?}"
        );
        assert!(
            repeat.right() < rect_for(bar, End::Right).left(),
            "the repeat switch runs into the gear at the other end"
        );

        // The same three places, read off a real frame: the helper above pins
        // the arithmetic, and this pins the placement the switch actually
        // draws in — a call site that asked for the wrong end would leave the
        // helper's answer untouched.
        let screen = screen();
        let shapes = painted(screen);
        let on_screen =
            |glyph: &str| written(&shapes, glyph).unwrap_or_else(|| panic!("no {glyph}"));
        let plus = on_screen(PLUS_GLYPH);
        let shuffle = on_screen(SHUFFLE_GLYPH);
        let repeat = on_screen(REPEAT_GLYPH);

        assert!(
            repeat.left() > shuffle.right(),
            "the repeat switch is not drawn to the right of the shuffle switch: \
             {shuffle:?} vs {repeat:?}"
        );
        assert!(
            repeat.left() > plus.right(),
            "the repeat switch is not drawn beyond the +: {plus:?} vs {repeat:?}"
        );
        assert!(
            repeat.right() < screen.width() / 2.0,
            "the repeat switch drifted out of the bar's left end: {repeat:?}"
        );

        // It takes the bar's own ink and fill like the switches beside it,
        // which is what the third trip through the same `put` buys.
        let mut visuals = egui::Visuals::dark();
        theme::apply_status_bar_visuals(&mut visuals);
        assert_eq!(
            visuals.widgets.inactive.fg_stroke.color,
            theme::STATUS_BAR_INK,
            "the repeat glyph takes the bar's ink"
        );
        assert_eq!(
            visuals.widgets.inactive.weak_bg_fill,
            theme::STATUS_BAR_BUTTON_FILL,
            "and rests in the bar's own fill, not a tile of its own"
        );
    }

    /// Every button on the bar lights up when the pointer is over it.
    ///
    /// The `+` is a plain `Button::new`, and egui draws a plain button from its
    /// `inactive` visuals even while it is hovered — the bar restyles those to
    /// its own fill, so before this the `+` gave no sign at all that the pointer
    /// had found it, while the gear (a `selectable`) did. This drives a real
    /// pointer onto each control in turn and reads the fill that was painted
    /// under it, so "alight" is measured off a frame rather than assumed.
    #[test]
    fn every_button_on_the_bar_lights_up_under_the_pointer() {
        for glyph in [PLUS_GLYPH, GEAR_GLYPH, SHUFFLE_GLYPH] {
            let mut app = app();
            let ctx = egui::Context::default();

            // Frame 1: find the control the way a pointer would.
            let first = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen()),
                    ..Default::default()
                },
                |ui| crate::ui::render(&mut app, ui),
            );
            let rect =
                written(&first.shapes, glyph).unwrap_or_else(|| panic!("the bar has no {glyph:?}"));
            first.drop_without_applying_deltas();

            // Frame 2: with the pointer sitting on it.
            let pos = rect.center();
            let mut input = egui::RawInput {
                screen_rect: Some(screen()),
                ..Default::default()
            };
            input.events = vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ];
            let hovered = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
            let mut drawn = Drawn::default();
            for clipped in &hovered.shapes {
                collect(&clipped.shape, &mut drawn);
            }
            hovered.drop_without_applying_deltas();

            assert_eq!(
                drawn.glyph_fill(glyph),
                Some(theme::STATUS_BAR_HOVER),
                "{glyph:?} does not light up under the pointer: got {:?}",
                drawn.glyph_fill(glyph)
            );
        }
    }

    /// What one pass over a bar painted.
    #[derive(Default)]
    struct Drawn {
        glyphs: Vec<String>,
        rects: Vec<(String, egui::Rect)>,
        fills: Vec<(egui::Rect, egui::Color32)>,
    }

    impl Drawn {
        fn glyph_rect(&self, glyph: &str) -> Option<egui::Rect> {
            self.rects
                .iter()
                .find(|(text, _)| text.contains(glyph))
                .map(|(_, rect)| *rect)
        }

        /// The fill the given glyph is drawn on: the smallest painted
        /// rectangle covering its ink, which is the widget's own background
        /// rather than the panel behind the whole bar.
        fn glyph_fill(&self, glyph: &str) -> Option<egui::Color32> {
            let ink = self.glyph_rect(glyph)?;

            self.fills
                .iter()
                .filter(|(rect, _)| rect.contains_rect(ink))
                .min_by(|a, b| a.0.area().total_cmp(&b.0.area()))
                .map(|(_, fill)| *fill)
        }
    }

    /// Run a bar through a real egui pass and collect the glyphs it painted.
    fn draw(
        app: &mut MusicCraftApp,
        mut draw: impl FnMut(&mut MusicCraftApp, &mut egui::Ui),
    ) -> Drawn {
        let ctx = egui::Context::default();
        let mut run = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 808.0),
                )),
                ..Default::default()
            },
            |ui| draw(app, ui),
        );

        let mut drawn = Drawn::default();
        for clipped in &run.shapes {
            collect(&clipped.shape, &mut drawn);
        }

        run.textures_delta.clear();
        drawn
    }

    fn collect(shape: &egui::Shape, drawn: &mut Drawn) {
        match shape {
            egui::Shape::Rect(rect) => drawn.fills.push((rect.rect, rect.fill)),
            egui::Shape::Text(text) => {
                let glyphs = text.galley.text().to_owned();
                let size = text.galley.size();
                let min = match text.galley.job.halign {
                    egui::Align::LEFT => text.pos,
                    egui::Align::Center => egui::pos2(text.pos.x - size.x / 2.0, text.pos.y),
                    egui::Align::RIGHT => egui::pos2(text.pos.x - size.x, text.pos.y),
                };
                drawn
                    .rects
                    .push((glyphs.clone(), egui::Rect::from_min_size(min, size)));
                drawn.glyphs.push(glyphs);
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, drawn);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn the_gear_sits_in_the_right_corner_of_the_bar() {
        let bar = Rect::from_min_max(egui::pos2(0.0, 970.0), egui::pos2(1180.0, 1000.0));
        let gear = rect_for(bar, End::Right);

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
        let plus = rect_for(bar, End::Left);
        let gear = rect_for(bar, End::Right);

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

    /// The shuffle switch sits immediately right of the `+`, the same size,
    /// with the requested two points of bar showing between them.
    #[test]
    fn the_shuffle_switch_sits_beside_the_plus_with_a_two_point_gap() {
        let bar = Rect::from_min_max(egui::pos2(0.0, 970.0), egui::pos2(1180.0, 1000.0));
        let plus = rect_for(bar, End::Left);
        let shuffle = rect_for(bar, End::LeftBeside);

        // Two points, spelled out: the request was for this gap, so the test
        // has to fail if the constant is changed rather than follow it.
        assert_eq!(
            shuffle.left() - plus.right(),
            2.0,
            "the gap between the + and the shuffle is not two points: \
             {plus:?} vs {shuffle:?}"
        );
        assert_eq!(
            theme::STATUS_BAR_BUTTON_GAP,
            2.0,
            "the gap constant no longer says two points"
        );
        assert_eq!(
            shuffle.size(),
            plus.size(),
            "the shuffle is a different size from the +: {shuffle:?} vs {plus:?}"
        );
        assert_eq!(
            shuffle.center().y,
            plus.center().y,
            "the two are not on the same line: {shuffle:?} vs {plus:?}"
        );
        assert!(
            bar.contains_rect(shuffle),
            "the shuffle is off the bar: {shuffle:?} is not inside {bar:?}"
        );
        assert!(
            shuffle.right() < rect_for(bar, End::Right).left(),
            "the shuffle runs into the gear at the other end"
        );
    }

    /// The shuffle switch is drawn with the bar's own ink, so it reads on the
    /// bar's gray in either theme — not with the theme's foreground, which is
    /// near-white in the dark theme.
    #[test]
    fn the_shuffle_switch_is_inked_like_the_bar_and_not_like_the_theme() {
        let mut visuals = egui::Visuals::dark();
        theme::apply_status_bar_visuals(&mut visuals);

        assert_eq!(
            visuals.widgets.inactive.fg_stroke.color,
            theme::STATUS_BAR_INK,
            "the shuffle glyph takes the bar's ink"
        );
        assert_eq!(
            visuals.widgets.noninteractive.fg_stroke.color,
            theme::STATUS_BAR_INK,
            "and so does the line it sits on"
        );

        // The hover fill is the bar's own gray, not the theme's near-black.
        assert_eq!(visuals.widgets.hovered.bg_fill, theme::STATUS_BAR_HOVER);
        assert_ne!(
            visuals.widgets.hovered.bg_fill,
            egui::Visuals::dark().widgets.hovered.bg_fill,
            "a hover on the bar took the dark theme's fill"
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
    fn the_plus_is_a_heavy_cross_the_fonts_can_draw() {
        // The heavy Greek cross, not an ASCII plus: an ASCII plus is drawn thin
        // and narrow (12 points across at 22pt, against the gear's 24), so it
        // reads as a smaller, lighter control than the glyphs beside it.
        assert_eq!(PLUS_GLYPH, "✚");

        let size = theme::STATUS_BAR_BUTTON_GLYPH_SIZE;
        let ctx = egui::Context::default();
        let mut measured = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let drawable = ui
                .ctx()
                .fonts_mut(|fonts| fonts.has_glyphs(&egui::FontId::proportional(size), PLUS_GLYPH));
            assert!(drawable, "none of egui's fonts can draw {PLUS_GLYPH}");

            // The reason the glyph was changed, measured rather than asserted
            // from taste: the heavy cross inks far wider than the thin one at
            // the same point size, so it fills its button like the gear does.
            let heavy = ui.ctx().fonts_mut(|fonts| {
                fonts.layout_no_wrap(
                    PLUS_GLYPH.to_owned(),
                    egui::FontId::proportional(size),
                    egui::Color32::WHITE,
                )
            });
            let thin = ui.ctx().fonts_mut(|fonts| {
                fonts.layout_no_wrap(
                    "+".to_owned(),
                    egui::FontId::proportional(size),
                    egui::Color32::WHITE,
                )
            });
            measured = Some((heavy.size().x, thin.size().x));
        });
        output.drop_without_applying_deltas();

        let (heavy, thin) = measured.expect("both glyphs were measured");
        assert!(
            heavy > thin * 1.3,
            "the heavy cross ({heavy}) is not markedly wider than the ASCII plus ({thin})"
        );
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

    /// The top bar and the status bar are the same colour on screen.
    ///
    /// Both fills are found in one frame of the whole shell, by POSITION rather
    /// than by colour, so this cannot be satisfied by querying for one palette
    /// twice: it compares the two bars' own fills, and a bar that changed its
    /// copy of the palette without the other following fails here where a test
    /// of either frame alone would pass.
    #[test]
    fn the_two_bars_that_frame_the_window_are_one_colour() {
        let screen = screen();
        let shapes = painted(screen);

        let mut full_width: Vec<(egui::Rect, egui::Color32)> = shapes
            .iter()
            .filter_map(|cs| match &cs.shape {
                egui::Shape::Rect(r) if (r.rect.width() - screen.width()).abs() < 0.5 => {
                    Some((r.rect, r.fill))
                }
                _ => None,
            })
            .collect();
        full_width.sort_by(|a, b| a.0.top().total_cmp(&b.0.top()));

        let band = |find: &mut dyn FnMut(&(egui::Rect, egui::Color32)) -> bool, of: &str| {
            full_width
                .iter()
                .find(|entry| find(entry))
                .copied()
                .unwrap_or_else(|| panic!("nothing spans the {of} of the window"))
        };
        let top = band(
            &mut |(rect, _)| (rect.top() - screen.top()).abs() < 1.0,
            "top",
        );
        let foot = band(
            &mut |(rect, _)| (rect.bottom() - screen.bottom()).abs() < 1.0,
            "foot",
        );

        assert_ne!(
            top.0, foot.0,
            "both bands came back as the same rectangle -- one bar is missing"
        );
        assert_eq!(
            top.1, foot.1,
            "the two bars that frame the window are different colours: {top:?} against {foot:?}"
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
        let plus = written(&shapes, PLUS_GLYPH).expect("the bar has no +");

        for (label, button) in [("plus", plus), ("gear", gear)] {
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

    /// Clicking the `+` raises the menu that names both kinds of new
    /// playlist, and choosing one opens that kind's editor.
    #[test]
    fn the_plus_offers_both_kinds_of_new_playlist() {
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
        let plus = written(&first.shapes, PLUS_GLYPH).expect("the bar has no +");
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

        assert!(
            app.playlist_draft().is_none() && app.smart_draft().is_none(),
            "the + itself opens nothing — it asks first"
        );

        // Frame 3: the menu is up, and it names both kinds.
        let offered = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| crate::ui::render(&mut app, ui),
        );
        assert!(
            written(&offered.shapes, "New playlist…").is_some(),
            "the menu did not offer a hand-made playlist"
        );
        let smart = written(&offered.shapes, "New smart playlist…")
            .expect("the menu did not offer a smart playlist");
        offered.drop_without_applying_deltas();

        // Frames 4 and 5: press and release on the smart entry, which is the
        // one the old test could not have anticipated.
        let entry = smart.center();
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        input.events = vec![egui::Event::PointerMoved(entry)];
        input.events.extend([egui::Event::PointerButton {
            pos: entry,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        }]);
        let down = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
        down.drop_without_applying_deltas();

        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        input.events = vec![egui::Event::PointerButton {
            pos: entry,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }];
        let up = ctx.run_ui(input, |ui| crate::ui::render(&mut app, ui));
        up.drop_without_applying_deltas();

        assert!(
            app.smart_draft().is_some(),
            "choosing the smart entry did not raise the rule editor"
        );
        assert!(
            app.playlist_draft().is_none(),
            "the smart entry opened the wrong editor"
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
