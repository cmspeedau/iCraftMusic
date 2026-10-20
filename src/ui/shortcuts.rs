//! The keyboard: playback and navigation without the mouse.
//!
//! Every binding is a [`Command`] in one list, so the keys the player answers
//! to can be shown in Settings and tested without a window. The transport
//! shortcuts are the ones a desktop player is expected to have — space, and the
//! arrow keys with the command key held — and they deliberately *do not* run
//! while a text field has focus: a space typed into the search box has to stay a
//! space.
//!
//! There is no global shortcut registration here. [egui] hands the frame the
//! keys that were pressed, and the event is consumed as it is recognised, so a
//! key this module uses cannot also do something else in the same frame.

use egui::{Key, Modifiers};

use crate::app::MusicCraftApp;
use crate::library::BrowseKind;
use crate::ui::navigation::NavSection;

/// How much one volume step moves the level.
pub const VOLUME_STEP: f32 = 0.05;

/// Something a key press can ask the player to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Start or pause playback.
    PlayPause,
    /// Skip to the next track.
    Next,
    /// Go back to the previous track.
    Previous,
    /// Turn the volume up one step.
    VolumeUp,
    /// Turn the volume down one step.
    VolumeDown,
    /// Open the Music section (the sidebar's "Music" row).
    Library,
    /// Open the Playlists section.
    Playlists,
    /// Open the Settings window.
    Settings,
    /// Show every track.
    Tracks,
    /// Show the albums.
    Albums,
    /// Show the artists.
    Artists,
    /// Edit the details of the selected track.
    EditDetails,
    /// Ask to remove the selected tracks from the library.
    RemoveSelection,
    /// Close whatever is open: the editor, a confirmation, or the queue.
    Dismiss,
}

impl Command {
    /// Every command, in the order Settings lists them.
    pub const ALL: [Command; 14] = [
        Self::PlayPause,
        Self::Next,
        Self::Previous,
        Self::VolumeUp,
        Self::VolumeDown,
        Self::Library,
        Self::Playlists,
        Self::Settings,
        Self::Tracks,
        Self::Albums,
        Self::Artists,
        Self::EditDetails,
        Self::RemoveSelection,
        Self::Dismiss,
    ];

    /// The keys that trigger this command, as the user should read them.
    #[must_use]
    pub fn keys(self) -> &'static str {
        match self {
            Self::PlayPause => "Space",
            Self::Next => "Ctrl/⌘ →",
            Self::Previous => "Ctrl/⌘ ←",
            Self::VolumeUp => "Ctrl/⌘ ↑",
            Self::VolumeDown => "Ctrl/⌘ ↓",
            Self::Library => "Ctrl/⌘ L",
            Self::Playlists => "Ctrl/⌘ P",
            Self::Settings => "Ctrl/⌘ ,",
            Self::Tracks => "Ctrl/⌘ 1",
            Self::Albums => "Ctrl/⌘ 2",
            Self::Artists => "Ctrl/⌘ 3",
            Self::EditDetails => "Ctrl/⌘ E",
            Self::RemoveSelection => "Delete",
            Self::Dismiss => "Esc",
        }
    }

    /// What the command does, as the user should read it.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::PlayPause => "Play or pause",
            Self::Next => "Next track",
            Self::Previous => "Previous track",
            Self::VolumeUp => "Louder",
            Self::VolumeDown => "Quieter",
            Self::Library => "Go to the music",
            Self::Playlists => "Go to the playlists",
            Self::Settings => "Open the Settings window",
            Self::Tracks => "Show all tracks",
            Self::Albums => "Show the albums",
            Self::Artists => "Show the artists",
            Self::EditDetails => "Edit the selected track's details",
            Self::RemoveSelection => "Remove the selection from the library",
            Self::Dismiss => "Close the editor, a confirmation or the queue",
        }
    }
}

/// Run a command.
pub fn run(app: &mut MusicCraftApp, command: Command) {
    match command {
        Command::PlayPause => app.toggle_playback(),
        Command::Next => app.next_track(),
        Command::Previous => app.previous_track(),
        Command::VolumeUp => change_volume(app, VOLUME_STEP),
        Command::VolumeDown => change_volume(app, -VOLUME_STEP),
        Command::Library => app.set_section(NavSection::Library),
        Command::Playlists => app.set_section(NavSection::Playlists),
        Command::Settings => app.open_settings(),
        Command::Tracks => show_page(app, BrowseKind::Tracks),
        Command::Albums => show_page(app, BrowseKind::Albums),
        Command::Artists => show_page(app, BrowseKind::Artists),
        Command::EditDetails => {
            app.edit_selection();
        }
        Command::RemoveSelection => app.ask_to_remove_selection(),
        Command::Dismiss => dismiss(app),
    }
}

/// Read the keyboard and run whatever it asked for.
pub fn handle(app: &mut MusicCraftApp, ui: &egui::Ui) {
    let ctx = ui.ctx();

    // A focused text field owns the keyboard: a space in the search box, or a
    // delete inside a path, must not drive the transport or the library.
    if ctx.text_edit_focused() {
        return;
    }

    if let Some(command) = pressed(ctx) {
        run(app, command);
    }
}

/// Close the topmost thing that is open.
///
/// One key, and the levels come off innermost first: a question the user is
/// being asked outranks the editor that asked it, the floating Settings and
/// Equalizer windows outrank the queue popover, and the queue is last.
fn dismiss(app: &mut MusicCraftApp) {
    if app.pending_confirmation().is_some() {
        app.cancel_confirmation();
        return;
    }
    if app.editor().is_some() {
        app.close_editor();
        return;
    }
    if app.settings_open() {
        app.close_settings();
        return;
    }
    if app.equalizer_open() {
        app.close_equalizer();
        return;
    }
    app.close_queue();
}

/// Move the volume by `delta`, clamped to the range the player accepts.
fn change_volume(app: &mut MusicCraftApp, delta: f32) {
    app.set_volume(app.volume() + delta);
}

/// Open one page of the Library.
fn show_page(app: &mut MusicCraftApp, kind: BrowseKind) {
    app.set_section(NavSection::Library);
    app.browse_mut().set_kind(kind);
}

/// The command the keys pressed this frame ask for, consuming the event.
///
/// The most specific combination is matched first, because a shortcut matches
/// even when extra shift or alt are held: without that order, `Ctrl+Shift+E`
/// would reach `Ctrl+E`.
fn pressed(ctx: &egui::Context) -> Option<Command> {
    ctx.input_mut(|input| {
        const NONE: Modifiers = Modifiers::NONE;
        const CMD: Modifiers = Modifiers::COMMAND;

        if input.consume_key(NONE, Key::Escape) {
            return Some(Command::Dismiss);
        }
        if input.consume_key(CMD, Key::E) {
            return Some(Command::EditDetails);
        }
        if input.consume_key(CMD, Key::ArrowRight) {
            return Some(Command::Next);
        }
        if input.consume_key(CMD, Key::ArrowLeft) {
            return Some(Command::Previous);
        }
        if input.consume_key(CMD, Key::ArrowUp) {
            return Some(Command::VolumeUp);
        }
        if input.consume_key(CMD, Key::ArrowDown) {
            return Some(Command::VolumeDown);
        }
        if input.consume_key(CMD, Key::L) {
            return Some(Command::Library);
        }
        if input.consume_key(CMD, Key::P) {
            return Some(Command::Playlists);
        }
        if input.consume_key(CMD, Key::Comma) {
            return Some(Command::Settings);
        }
        if input.consume_key(CMD, Key::Num1) {
            return Some(Command::Tracks);
        }
        if input.consume_key(CMD, Key::Num2) {
            return Some(Command::Albums);
        }
        if input.consume_key(CMD, Key::Num3) {
            return Some(Command::Artists);
        }
        if input.consume_key(NONE, Key::Delete) || input.consume_key(NONE, Key::Backspace) {
            return Some(Command::RemoveSelection);
        }
        if input.consume_key(NONE, Key::Space) {
            return Some(Command::PlayPause);
        }

        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use crate::app::{Startup, TransportState};
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{ClickMode, Library, Track, TrackId};

    /// A key being pressed, as the window would report it.
    fn key(key: Key, modifiers: Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    /// Feed `events` to one frame, with `app` behind the keyboard.
    fn press(app: &mut MusicCraftApp, events: Vec<egui::Event>) {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| handle(app, ui),
        );
        output.textures_delta.clear();
    }

    #[test]
    fn every_command_has_keys_and_a_description_to_show() {
        for command in Command::ALL {
            assert!(!command.keys().is_empty(), "{command:?} has no keys");
            assert!(
                !command.description().is_empty(),
                "{command:?} has no description"
            );
        }
    }

    #[test]
    fn no_two_commands_claim_the_same_keys() {
        for command in Command::ALL {
            let clashes = Command::ALL
                .iter()
                .filter(|other| **other != command && other.keys() == command.keys())
                .count();

            assert_eq!(clashes, 0, "{command:?} shares its keys");
        }
    }

    #[test]
    fn the_space_bar_starts_the_player() {
        let mut app = app();
        app.open_path(Path::new("/music/song.mp3"));
        // Opening a track already tries to start it, and without a sound card
        // that leaves a message; the space bar has to leave one of its own.
        app.dismiss_playback_error();
        assert!(app.playback_error().is_none());

        press(&mut app, vec![key(Key::Space, Modifiers::NONE)]);

        // Without a sound card the attempt to start is what fails, which is
        // still proof the transport was asked to run.
        assert!(
            app.playback_error().is_some(),
            "the space bar should have reached the transport"
        );
    }

    #[test]
    fn a_space_typed_into_a_text_field_is_not_a_transport_command() {
        let mut app = app();
        app.open_path(Path::new("/music/song.mp3"));
        app.dismiss_playback_error();
        let mut typed = String::new();
        let ctx = egui::Context::default();

        // A frame with the field in it, so focus can be handed over...
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let response = ui.text_edit_singleline(&mut typed);
            ui.memory_mut(|memory| memory.request_focus(response.id));
        });
        output.textures_delta.clear();

        // ...then the space bar, which belongs to the field.
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![key(Key::Space, Modifiers::NONE)],
                ..Default::default()
            },
            |ui| {
                ui.text_edit_singleline(&mut typed);
                handle(&mut app, ui);
            },
        );
        output.textures_delta.clear();

        assert!(
            app.playback_error().is_none(),
            "typing a space must not start playback"
        );
    }

    #[test]
    fn the_arrow_keys_move_through_the_queue() {
        let mut app = app();
        app.open_paths([
            std::path::PathBuf::from("/music/a.mp3"),
            std::path::PathBuf::from("/music/b.mp3"),
        ]);

        press(&mut app, vec![key(Key::ArrowRight, Modifiers::COMMAND)]);
        assert_eq!(
            app.now_playing().map(|track| track.title),
            Some("b".to_owned())
        );

        press(&mut app, vec![key(Key::ArrowLeft, Modifiers::COMMAND)]);
        assert_eq!(
            app.now_playing().map(|track| track.title),
            Some("a".to_owned())
        );
    }

    #[test]
    fn a_plain_arrow_key_is_left_to_the_widgets() {
        let mut app = app();
        app.open_paths([
            std::path::PathBuf::from("/music/a.mp3"),
            std::path::PathBuf::from("/music/b.mp3"),
        ]);

        press(&mut app, vec![key(Key::ArrowRight, Modifiers::NONE)]);

        assert_eq!(
            app.now_playing().map(|track| track.title),
            Some("a".to_owned())
        );
    }

    #[test]
    fn the_volume_keys_move_the_level_by_one_step() {
        let mut app = app();
        app.set_volume(0.5);

        press(&mut app, vec![key(Key::ArrowUp, Modifiers::COMMAND)]);
        assert!((app.volume() - (0.5 + VOLUME_STEP)).abs() < f32::EPSILON);

        press(&mut app, vec![key(Key::ArrowDown, Modifiers::COMMAND)]);
        assert!((app.volume() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn the_volume_keys_stop_at_the_ends_of_the_range() {
        let mut app = app();
        app.set_volume(1.0);

        press(&mut app, vec![key(Key::ArrowUp, Modifiers::COMMAND)]);
        assert!((app.volume() - 1.0).abs() < f32::EPSILON);

        app.set_volume(0.0);
        press(&mut app, vec![key(Key::ArrowDown, Modifiers::COMMAND)]);
        assert!((app.volume() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_command_and_number_keys_open_the_library_pages() {
        let mut app = app();

        press(&mut app, vec![key(Key::Num2, Modifiers::COMMAND)]);
        assert_eq!(app.section(), NavSection::Library);
        assert_eq!(app.browse().kind(), BrowseKind::Albums);

        press(&mut app, vec![key(Key::Num3, Modifiers::COMMAND)]);
        assert_eq!(app.browse().kind(), BrowseKind::Artists);

        press(&mut app, vec![key(Key::Num1, Modifiers::COMMAND)]);
        assert_eq!(app.browse().kind(), BrowseKind::Tracks);
    }

    #[test]
    fn the_section_keys_open_every_section() {
        let mut app = app();

        press(&mut app, vec![key(Key::P, Modifiers::COMMAND)]);
        assert_eq!(app.section(), NavSection::Playlists);

        press(&mut app, vec![key(Key::L, Modifiers::COMMAND)]);
        assert_eq!(app.section(), NavSection::Library);
    }

    /// `Ctrl/⌘ ,` opens the Settings window rather than a page — and the
    /// window arriving must not move the page underneath it, because the
    /// keystroke means "change something", not "stop looking at this".
    #[test]
    fn the_comma_opens_the_settings_window_over_the_page() {
        let mut app = app();
        press(&mut app, vec![key(Key::P, Modifiers::COMMAND)]);

        press(&mut app, vec![key(Key::Comma, Modifiers::COMMAND)]);
        assert!(app.settings_open(), "the comma opened nothing");
        assert_eq!(
            app.section(),
            NavSection::Playlists,
            "the Settings window pushed the page off screen"
        );
    }

    /// `Esc` puts away the floating windows too, topmost of the open things
    /// first, the way it already closes the question and the editor.
    #[test]
    fn the_escape_key_closes_the_floating_windows() {
        let mut app = app();

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        app.toggle_equalizer();
        app.open_settings();

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(
            !app.settings_open() && app.equalizer_open(),
            "Esc closed the wrong window: settings {}, equalizer {}",
            app.settings_open(),
            app.equalizer_open()
        );

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(!app.equalizer_open(), "Esc let the equalizer stay");
    }

    #[test]
    fn a_command_and_a_letter_that_is_not_bound_does_nothing() {
        let mut app = app();
        let before = app.section();

        press(&mut app, vec![key(Key::Q, Modifiers::COMMAND)]);

        assert_eq!(app.section(), before);
        assert_eq!(app.transport(), TransportState::Unavailable);
    }

    #[test]
    fn the_edit_key_opens_the_editor_for_one_selected_track() {
        let mut app = imported();
        let visible = app.browse().visible_track_ids();
        select(&mut app, visible[0]);

        press(&mut app, vec![key(Key::E, Modifiers::COMMAND)]);

        assert_eq!(
            app.editor().map(|editor| editor.track_id()),
            Some(visible[0]),
            "the editor opens on the row that is selected"
        );
    }

    #[test]
    fn the_remove_key_asks_before_it_removes_anything() {
        let mut app = imported();
        let visible = app.browse().visible_track_ids();
        select(&mut app, visible[0]);

        press(&mut app, vec![key(Key::Delete, Modifiers::NONE)]);

        assert!(app.pending_confirmation().is_some(), "the user is asked");
        assert_eq!(
            app.browse().track_count(),
            2,
            "and nothing is removed until they answer"
        );
    }

    #[test]
    fn the_escape_key_closes_the_innermost_thing_that_is_open() {
        let mut app = imported();
        app.toggle_queue();
        assert!(app.queue_open());

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(!app.queue_open(), "the queue popover closes");

        let visible = app.browse().visible_track_ids();
        select(&mut app, visible[0]);
        app.open_editor(visible[0]);
        app.ask_to_remove_selection();
        assert!(app.pending_confirmation().is_some());

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(
            app.pending_confirmation().is_none(),
            "the question outranks the editor that asked it"
        );
        assert!(app.editor().is_some(), "and the editor stays open");

        press(&mut app, vec![key(Key::Escape, Modifiers::NONE)]);
        assert!(app.editor().is_none(), "then the editor itself");
    }

    /// Select one row of the library table, as a click would.
    fn select(app: &mut MusicCraftApp, id: TrackId) {
        let visible = app.browse().visible_track_ids();
        app.browse_mut()
            .selection_mut()
            .apply(&visible, id, ClickMode::Replace);
    }

    fn app() -> MusicCraftApp {
        MusicCraftApp::with_playback(startup(), playback())
    }

    fn startup() -> Startup {
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        }
    }

    fn playback() -> Playback {
        Playback::with_engine(AudioEngine::without_device(), 1.0)
    }

    /// An application whose library holds two tracks.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-keys-artwork"))
            .expect("an in-memory library");
        let mut app = MusicCraftApp::with_library(startup(), playback(), library);

        for (id, title) in [(1, "Xtal"), (2, "Roygbiv")] {
            let mut track = Track::from_path(format!("/music/{title}.mp3"));
            track.id = Some(TrackId(id));
            track.title = title.to_owned();
            app.library_mut()
                .expect("a library")
                .repository_mut()
                .upsert_track(&track)
                .expect("the track should be stored");
        }

        app.refresh().expect("the library should refresh");

        app
    }
}
