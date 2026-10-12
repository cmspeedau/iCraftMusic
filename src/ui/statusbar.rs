//! The status bar across the bottom of the window.
//!
//! One line saying how much music the library holds — songs, albums and artists
//! — centred on a gray strip that spans the window. The bar reports only: what
//! is playing is the transport's business at the top of the window, and what can
//! be done to a selection is the library's, so the bar holds no control.

use egui::{Panel, RichText};

use crate::app::MusicCraftApp;
use crate::ui::theme;

/// Draw the status bar.
pub fn bar(app: &MusicCraftApp, ui: &mut egui::Ui) {
    Panel::bottom("musiccraft_status_bar")
        .resizable(false)
        .exact_size(theme::STATUS_BAR_HEIGHT)
        .frame(theme::status_bar_frame())
        .show(ui, |ui| {
            ui.scope(|ui| {
                // The bar paints its own fill in every theme, so its line takes
                // the bar's ink rather than the theme's foreground, which is
                // near-white in the dark theme and would vanish on the gray.
                theme::apply_status_bar_visuals(&mut ui.style_mut().visuals);
                counts(app, ui);
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
    // out along both axes of the whole bar rather than from its top left.
    ui.centered_and_justified(|ui| {
        ui.label(RichText::new(line).small().weak());
    });
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
        let startup = Startup {
            config: crate::config::AppConfig::default(),
            config_path: None,
            error: None,
        };
        let playback = Playback::with_engine(AudioEngine::without_device(), 1.0);
        let app = MusicCraftApp::with_playback(startup, playback);

        egui::__run_test_ui(|ui| bar(&app, ui));
    }
}
