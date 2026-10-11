//! The Library section: load something to play, then see the queue.
//!
//! Stage 2 has no database yet, so the "library" is the play field and the
//! queue it fills. Stage 3 replaces the field with a scanned, tag-aware library
//! and keeps the same queue underneath.

use egui::{Button, Key, RichText, TextEdit};

use crate::app::MusicCraftApp;
use crate::ui::{empty_state, theme};

/// Draw the Library.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    play_field(app, ui);
    ui.add_space(18.0);

    if app.playback().queue().is_empty() {
        empty_state(
            ui,
            "Nothing in the queue",
            "Load a file or a folder to start listening.",
            &[
                "Folder scanning into a real library arrives in Stage 3",
                "Tag reading, album art and playlists arrive in Stage 3",
            ],
        );
        return;
    }

    queue(app, ui);
}

/// The field that accepts a path, and the button that plays it.
fn play_field(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Play a file or folder").strong());
    ui.label(
        RichText::new("Paste a path and press Enter. Folders are read one level deep.")
            .small()
            .weak(),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        const BUTTON_WIDTH: f32 = 76.0;

        let field_width = (ui.available_width() - BUTTON_WIDTH - 8.0).max(120.0);
        let field = ui.add_sized(
            [field_width, 26.0],
            TextEdit::singleline(app.path_input_mut()).hint_text("/home/you/Music/album"),
        );

        // A single-line field gives up focus on Enter, which is how a keyboard
        // user starts playback without reaching for the mouse.
        let submitted = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));

        if ui
            .add_sized([BUTTON_WIDTH, 26.0], Button::new("Play"))
            .clicked()
            || submitted
        {
            app.play_path_input();
        }
    });
}

/// The queue, as a numbered list the user can click to jump to a track.
fn queue(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Read everything the list needs before borrowing the app mutably, so
    // clicking a row can start that track.
    let titles: Vec<String> = app
        .playback()
        .queue()
        .items()
        .iter()
        .map(|item| item.title().to_owned())
        .collect();
    let playing = app.playback().queue().current_index();

    ui.horizontal(|ui| {
        ui.label(RichText::new("Queue").strong());
        let count = titles.len();
        ui.label(
            RichText::new(format!(
                "{count} {}",
                if count == 1 { "track" } else { "tracks" }
            ))
            .small()
            .weak(),
        );
    });
    ui.add_space(6.0);

    let mut requested = None;
    for (index, title) in titles.iter().enumerate() {
        let current = playing == Some(index);
        let label = format!("{}. {title}", index + 1);
        let text = if current {
            RichText::new(label).color(theme::ACCENT)
        } else {
            RichText::new(label)
        };

        let width = ui.available_width();
        if ui
            .add_sized([width, 24.0], Button::selectable(current, text))
            .clicked()
        {
            requested = Some(index);
        }
    }

    if let Some(index) = requested {
        app.playback_mut().play_index(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;

    #[test]
    fn the_play_field_accepts_a_folder_and_fills_the_queue() {
        let folder = fixture_folder("library-play-field");
        std::fs::write(folder.join("song.mp3"), b"not really audio")
            .expect("the fixture should be writable");
        let mut app = app();

        *app.path_input_mut() = folder.display().to_string();
        let added = app.play_path_input();

        assert_eq!(added, 1);
        assert_eq!(app.playback().queue().len(), 1);
        assert_eq!(app.path_input(), "", "the field clears after a success");
    }

    #[test]
    fn a_folder_with_no_audio_leaves_the_input_alone() {
        let folder = fixture_folder("library-no-audio");
        std::fs::write(folder.join("readme.txt"), b"text").expect("the fixture should be writable");
        let mut app = app();

        *app.path_input_mut() = folder.display().to_string();

        assert_eq!(app.play_path_input(), 0);
        assert!(app.playback().queue().is_empty());
        assert_eq!(
            app.path_input(),
            folder.display().to_string(),
            "a path that added nothing stays put so it can be corrected"
        );
    }

    /// An empty folder of our own, so the tests never share fixture state.
    fn fixture_folder(name: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("musiccraft-{name}"));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("the fixture folder should be creatable");

        folder
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
}
