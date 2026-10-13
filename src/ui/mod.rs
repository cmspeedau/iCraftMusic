//! The Stage 1 user interface shell, grown by the stages after it.
//!
//! [`render`] is the single entry point: it lays out the status bar, the
//! playback bar, the sidebar and the content area, in that order, on top of the
//! root `Ui` that `eframe` hands to [`crate::app::MusicCraftApp::ui`]. Each
//! sub-module draws one region and holds its own small amount of logic.
//!
//! The window has no header strip: what one showed (the application name and
//! version, and the failure to load the stored settings) lives in the Settings
//! section's About pane, so the first row of the window is the transport.
//!
//! Two things are drawn over everything rather than inside a section: the
//! metadata editor ([`editview`]) and the confirmation a destructive library
//! action asks for ([`dialog`]). Both are floating windows, so they can be
//! opened from any section and cannot be clipped by one. The keyboard is read
//! last ([`shortcuts`]), which is what keeps a shortcut from stealing a key
//! from a widget that has just claimed it.

pub mod artwork;
pub mod dialog;
pub mod editview;
pub mod libraryview;
pub mod navigation;
pub mod nowplaying;
pub mod playlistedit;
pub mod playlistview;
pub mod queueview;
pub mod settingsview;
pub mod shortcuts;
pub mod statusbar;
pub mod theme;

use egui::{CentralPanel, RichText, ScrollArea};

use crate::app::MusicCraftApp;

use navigation::NavSection;

/// Draw the whole window.
pub fn render(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Panels are added in the order they should claim space: the two bars that
    // frame the window first — the status bar below, the transport above — then
    // the sidebar, then whatever is left over.
    statusbar::bar(app, ui);
    nowplaying::bar(app, ui);
    navigation::sidebar(app, ui);
    content(app, ui);

    // The editor and the confirmation are overlays on top of whatever section
    // is open, and the keyboard is read last so a shortcut cannot fight with a
    // widget that has just claimed the key for itself.
    editview::window(app, ui);
    playlistedit::window(app, ui);
    dialog::confirm(app, ui);
    shortcuts::handle(app, ui);
}

/// Content area: the open playlist's line, then the section's view.
///
/// Nothing here names the Library or its Pages: the sidebar's selected entry
/// says which section is on screen, and the Library's own line says which of
/// its three pages is showing, so a heading over the table was a third saying
/// of the same thing. What is left is the one line the section cannot say for
/// itself — an open playlist, its counts and the buttons that act on it.
///
/// The Library's search field is not here either: it rides at the right end of
/// the playback bar, which is on screen in every section.
fn content(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    CentralPanel::default().show(ui, |ui| {
        ui.add_space(4.0);

        // Only the Playlists section takes the open playlist's name: a playlist
        // left open in the sidebar must not come up over the Library. The two
        // strings are taken off the app before the closure opens, so the
        // buttons inside it can borrow the app in turn.
        let section = app.section();
        let open = app.selected_playlist_summary();
        let (heading, counts) = match (section, open) {
            (NavSection::Playlists, Some(summary)) => (
                Some(summary.display_name().to_owned()),
                Some(summary.totals()),
            ),
            _ => (None, None),
        };

        if let Some(heading) = heading {
            ui.horizontal(|ui| {
                ui.label(RichText::new(heading).heading());
                if let Some(counts) = counts {
                    ui.label(RichText::new(counts).small().weak());
                }
                playlistview::controls(app, ui);
            });
            ui.add_space(10.0);
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match app.section() {
                NavSection::Library => libraryview::show(app, ui),
                NavSection::Playlists => playlistview::show(app, ui),
                NavSection::Settings => settingsview::show(app, ui),
            });
    });
}

/// A centred placeholder for sections that have no data yet.
///
/// `bullets` are shown as a short list under the explanation; pass `&[]` to
/// show the title and detail only.
pub fn empty_state(ui: &mut egui::Ui, title: &str, detail: &str, bullets: &[&str]) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(18.0).strong());
        ui.add_space(6.0);
        ui.label(RichText::new(detail).weak());

        if !bullets.is_empty() {
            ui.add_space(14.0);
            ui.label(RichText::new("What is coming").small().weak());
            for bullet in bullets {
                ui.label(RichText::new(format!("• {bullet}")).small().weak());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{ClickMode, Library, Track, TrackId};

    #[test]
    fn the_editor_and_the_question_draw_over_the_section_that_is_open() {
        let mut app = imported();
        let visible = app.browse().visible_track_ids();
        app.browse_mut()
            .selection_mut()
            .apply(&visible, visible[0], ClickMode::Replace);
        assert!(app.edit_selection());
        app.ask_to_remove_selection();

        let painted = shell(&mut app);

        assert!(
            painted.iter().any(|text| text == "Edit details"),
            "the editor is not on screen: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Remove from the library?")),
            "the question is not on screen: {painted:?}"
        );
    }

    #[test]
    fn nothing_extra_is_drawn_when_nothing_is_open() {
        let mut app = imported();

        let painted = shell(&mut app);

        assert!(
            !painted.iter().any(|text| text == "Edit details"),
            "no editor is open: {painted:?}"
        );
        assert!(painted.iter().any(|text| text == "Library"));
    }

    #[test]
    fn the_playlists_section_is_named_by_the_open_playlist_not_by_the_section() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &[TrackId(1)]);

        app.set_section(NavSection::Playlists);
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        let painted = shell(&mut app);

        assert!(
            painted.iter().any(|text| text == "Evening"),
            "the heading does not name the open playlist: {painted:?}"
        );
        assert_eq!(
            painted
                .iter()
                .filter(|text| text.as_str() == "Evening")
                .count(),
            2,
            "the playlist's name should be the sidebar entry and the heading, and \
             nothing else — a third copy is the label that used to sit in the body: {painted:?}"
        );
        assert!(
            !painted
                .iter()
                .any(|text| text == "Collections you have put together"),
            "the section's own description is still under the heading: {painted:?}"
        );
    }

    /// The Playlists section carries no heading of its own over its view.
    ///
    /// The sidebar's heading over the list is the name, and the section's
    /// description went with the Library's when the headings over the content
    /// area were dropped — with nothing open there is nothing on that line.
    #[test]
    fn the_playlists_section_carries_no_heading_of_its_own() {
        let mut app = imported();
        app.set_section(NavSection::Playlists);

        let painted = shell(&mut app);

        assert!(
            painted.iter().any(|text| text == "Playlists"),
            "the sidebar no longer names the section: {painted:?}"
        );
        assert!(
            !painted
                .iter()
                .any(|text| text == "Collections you have put together"),
            "the section's description is still painted over its view: {painted:?}"
        );
    }

    /// The buttons act on the playlist the heading names, so they belong on
    /// that line, at the far end of it — not on a row of their own above the
    /// tracks, where they read as belonging to whatever came next.
    #[test]
    fn the_open_playlists_buttons_ride_on_the_heading_line_at_its_right() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &[TrackId(1)]);
        app.set_section(NavSection::Playlists);
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        let shapes = frame(&mut app);

        // "Evening" is on screen twice — the sidebar entry and the heading —
        // and the sidebar comes first, so the heading is the one in the
        // content area, past the sidebar's width.
        let heading = texts(&shapes, "Evening")
            .into_iter()
            .find(|rect| rect.left() > 208.0)
            .expect("the heading");
        let counts = named(&shapes, "1 track").expect("the count line");
        let play = named(&shapes, "Play").expect("the Play button");

        assert!(
            play.top() < heading.bottom() && play.bottom() > heading.top(),
            "the buttons are not on the heading's line: {play:?} vs {heading:?}"
        );
        assert!(
            play.left() > heading.right(),
            "the buttons are not to the right of the heading: {play:?} vs {heading:?}"
        );

        // The counts ride on the same line, between the name and the buttons:
        // a line of their own under the heading would be the second row this
        // test was written to rule out, one height down from it.
        assert!(
            counts.top() < heading.bottom() && counts.bottom() > heading.top(),
            "the counts are not on the heading's line: {counts:?} vs {heading:?}"
        );
        assert!(
            counts.left() > heading.right() && counts.right() < play.left(),
            "the counts are not between the name and the buttons: {counts:?} vs {heading:?} / {play:?}"
        );
    }

    /// The tokens are changed on the shapes directly: what is on the screen is
    /// the pixels, not the string parameter that was passed in.
    fn frame(app: &mut MusicCraftApp) -> Vec<egui::epaint::ClippedShape> {
        let ctx = egui::Context::default();
        // The application applies the theme when it draws; a bare context has
        // egui's own styles, so a test that reads a font size would read the
        // wrong one. Applying it here is what the real window does.
        let mode = app.theme();
        crate::ui::theme::apply(&ctx, mode);
        let mut output = None;

        for _ in 0..3 {
            let mut run = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1180.0, 808.0),
                    )),
                    ..Default::default()
                },
                |ui| render(app, ui),
            );
            run.textures_delta.clear();
            output = Some(run);
        }

        output.expect("`run_ui` ran at least once").shapes
    }

    /// The rectangle the first line of text reading `needle` was painted into.
    fn named(shapes: &[egui::epaint::ClippedShape], needle: &str) -> Option<egui::Rect> {
        texts(shapes, needle).into_iter().next()
    }

    /// Every rectangle a line of text reading `needle` was painted into.
    fn texts(shapes: &[egui::epaint::ClippedShape], needle: &str) -> Vec<egui::Rect> {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) if text.galley.text() == needle => {
                    let size = text.galley.size();
                    let min = match text.galley.job.halign {
                        egui::Align::LEFT => text.pos,
                        egui::Align::Center => egui::pos2(text.pos.x - size.x / 2.0, text.pos.y),
                        egui::Align::RIGHT => egui::pos2(text.pos.x - size.x, text.pos.y),
                    };
                    Some(egui::Rect::from_min_size(min, size))
                }
                _ => None,
            })
            .collect()
    }

    /// Nothing in the interface is set at egui's default "small" size: 9.0 is
    /// too small to read on a desktop, so the style raises it to 12.0 once,
    /// rather than each `.small()` call site overriding it.
    ///
    /// The playlist's counts are the one line left in the content area, so it
    /// is the call site the check has to read a size off.
    #[test]
    fn no_small_text_is_left_at_egui_s_own_size() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &[TrackId(1)]);
        app.set_section(NavSection::Playlists);
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        let counts = size_of(&frame(&mut app), "1 track").expect("the counts line");

        assert!(
            counts > 9.0,
            "the playlist's counts are still at egui's own small size: {counts}"
        );
    }

    /// The font size the line of text reading `needle` was set in.
    ///
    /// The painted box, not the glyphs: it carries the line spacing as well, so
    /// a 9pt line measures 10 and a 12pt one measures 14. What the request is
    /// about is the size the text is *set* in.
    fn size_of(shapes: &[egui::epaint::ClippedShape], needle: &str) -> Option<f32> {
        shapes.iter().find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.text() == needle => text
                .galley
                .job
                .sections
                .first()
                .map(|section| section.format.font_id.size),
            _ => None,
        })
    }

    /// Every line of text a whole frame of the shell paints.
    ///
    /// Drawn more than once because a floating area has to measure itself
    /// before it paints anything.
    fn shell(app: &mut MusicCraftApp) -> Vec<String> {
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
                |ui| render(app, ui),
            );
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let output = output.expect("`run_ui` ran at least once");
        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }

        texts
    }

    /// Collect the text of a shape and everything nested inside it.
    fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, texts);
                }
            }
            _ => {}
        }
    }

    /// An application whose library holds one track.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-shell-artwork"))
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

        let mut track = Track::from_path("/music/Roygbiv.mp3");
        track.id = Some(TrackId(1));
        track.title = "Roygbiv".to_owned();
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
        app.refresh().expect("the library should refresh");

        app
    }
}
