//! Application state and the `eframe` integration.
//!
//! The shell renders navigation, content panes and the playback bar, and owns
//! the audio side through a single [`Playback`] facade: the output device, the
//! queue and the transport live there. This module keeps the parts that belong
//! to the window — which section is open, the theme, the settings that get
//! written to disk — and the thin wrappers the views call.
//!
//! Reading tags, scanning folders into a library and playlists are Stage 3 work.
//! Stage 2 stops at "load a file or folder, play it, keep a queue", which is
//! what the playback bar and the library pane's play field exercise.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::CreationContext;

use crate::audio::{Playback, PlaybackEvent, RepeatMode};
use crate::config::{AppConfig, ThemeMode};
use crate::error::AppError;
use crate::ui;
use crate::ui::navigation::NavSection;

/// What was learned about the user's settings before the window opened.
///
/// The window has to be able to open even when the settings file is missing,
/// unreadable or malformed, so a failure is carried alongside the defaults
/// rather than aborting startup.
#[derive(Debug)]
pub struct Startup {
    /// Settings loaded from disk, or the defaults when loading failed.
    pub config: AppConfig,
    /// Where the settings file lives, or `None` when the platform has no
    /// per-user configuration directory.
    pub config_path: Option<PathBuf>,
    /// Why the stored settings were not used, if they were not.
    pub error: Option<AppError>,
}

impl Startup {
    /// Read the settings for the current user.
    pub fn load() -> Self {
        match AppConfig::default_path() {
            Ok(path) => {
                let (config, error) = AppConfig::load_or_default(&path);
                Self {
                    config,
                    config_path: Some(path),
                    error,
                }
            }
            Err(error) => Self {
                config: AppConfig::default(),
                config_path: None,
                error: Some(error),
            },
        }
    }
}

/// The state the playback bar mirrors.
///
/// It is derived from [`Playback`] on demand rather than stored, so the bar can
/// never disagree with the engine. [`TransportState::Unavailable`] means no
/// output device was opened — the queue and the settings still work, only sound
/// is impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportState {
    /// No audio backend is attached.
    #[default]
    Unavailable,
    /// A track is loaded and playback is stopped.
    Stopped,
    /// Audio is playing.
    Playing,
    /// Playback is temporarily suspended.
    Paused,
}

impl TransportState {
    /// Short label shown next to the playback controls.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "Not connected",
            Self::Stopped => "Stopped",
            Self::Playing => "Playing",
            Self::Paused => "Paused",
        }
    }
}

/// The track shown in the playback bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    /// Track title, taken from the file name until Stage 3 reads tags.
    pub title: String,
    /// Where the track sits in the queue, e.g. `Track 2 of 3`.
    pub position: Option<String>,
    /// The track's length, e.g. `3:45`, when the decoder reported one.
    pub duration: Option<String>,
}

/// Top-level application state.
pub struct MusicCraftApp {
    config: AppConfig,
    config_path: Option<PathBuf>,
    /// Set whenever a setting changes, cleared once it is written to disk.
    config_dirty: bool,
    config_error: Option<String>,
    section: NavSection,
    /// The output device, the queue and the transport, as one object.
    playback: Playback,
    /// Text the user is typing into the library's play field.
    path_input: String,
    /// The theme currently installed into `egui`, so it is only re-applied when
    /// the preference actually changes.
    applied_theme: Option<ThemeMode>,
}

impl MusicCraftApp {
    /// Build the application from the result of [`Startup::load`].
    pub fn new(_creation_context: &CreationContext<'_>, startup: Startup) -> Self {
        let volume = startup.config.volume;
        Self::with_playback(startup, Playback::start(volume))
    }

    /// Build the application around an existing [`Playback`].
    ///
    /// Tests pass a player whose device could not be opened, so the transport
    /// can be exercised on machines without a sound card.
    pub fn with_playback(startup: Startup, playback: Playback) -> Self {
        let error = startup.error.as_ref().map(ToString::to_string);

        Self {
            config: startup.config,
            config_path: startup.config_path,
            config_dirty: false,
            config_error: error,
            section: NavSection::default(),
            playback,
            path_input: String::new(),
            applied_theme: None,
        }
    }

    /// The section currently shown in the content area.
    pub fn section(&self) -> NavSection {
        self.section
    }

    /// Switch to another section.
    pub fn set_section(&mut self, section: NavSection) {
        self.section = section;
    }

    /// The state of the transport, as the playback bar should draw it.
    #[must_use]
    pub fn transport(&self) -> TransportState {
        if !self.playback.is_available() {
            TransportState::Unavailable
        } else if self.playback.is_playing() {
            TransportState::Playing
        } else if self.playback.is_paused() {
            TransportState::Paused
        } else {
            TransportState::Stopped
        }
    }

    /// The track the playback bar should show, if the queue has one selected.
    ///
    /// Tags are not read yet, so the title comes from the file name and the
    /// only other information available is what the queue and the decoder hold.
    #[must_use]
    pub fn now_playing(&self) -> Option<NowPlaying> {
        let track = self.playback.current()?;
        let position = self
            .playback
            .queue_position()
            .map(|(index, total)| format!("Track {index} of {total}"));
        let duration = self.playback.duration().map(format_duration);

        Some(NowPlaying {
            title: track.title().to_owned(),
            position,
            duration,
        })
    }

    /// How far into the current track playback has reached.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.playback.position()
    }

    /// The length of the current track, when the decoder reported one.
    #[must_use]
    pub fn track_duration(&self) -> Option<Duration> {
        self.playback.duration()
    }

    /// How far through the current track playback is, in `0.0..=1.0`.
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.playback.progress()
    }

    /// The audio side, for views that only need to read it.
    #[must_use]
    pub fn playback(&self) -> &Playback {
        &self.playback
    }

    /// The audio side, for views that drive the transport.
    pub fn playback_mut(&mut self) -> &mut Playback {
        &mut self.playback
    }

    /// Start or pause, whichever the current state calls for.
    pub fn toggle_playback(&mut self) {
        self.playback.toggle();
    }

    /// Skip to the next track.
    pub fn next_track(&mut self) {
        self.playback.next();
    }

    /// Go back to the previous track.
    pub fn previous_track(&mut self) {
        self.playback.previous();
    }

    /// Stop playback and unload the track.
    pub fn stop_playback(&mut self) {
        self.playback.stop();
    }

    /// Seek to a fraction of the current track.
    pub fn seek_to_fraction(&mut self, fraction: f32) {
        self.playback.seek_fraction(fraction);
    }

    /// Change how the queue behaves at the end of a track.
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.playback.set_repeat(mode);
    }

    /// Turn shuffle on or off, returning the new state.
    pub fn toggle_shuffle(&mut self) -> bool {
        self.playback.toggle_shuffle()
    }

    /// Let the player notice that a track ended, and advance if it did.
    ///
    /// The engine cannot call back into the interface, so this runs once per
    /// frame from [`eframe::App::ui`].
    pub fn poll_playback(&mut self) -> Option<PlaybackEvent> {
        self.playback.poll()
    }

    /// The failure the player wants the user to see, ready to display.
    #[must_use]
    pub fn playback_error(&self) -> Option<String> {
        self.playback.error().map(ToString::to_string)
    }

    /// Hide the playback failure message.
    pub fn dismiss_playback_error(&mut self) {
        self.playback.dismiss_error();
    }

    /// The text in the library's "play a file or folder" field.
    #[must_use]
    pub fn path_input(&self) -> &str {
        &self.path_input
    }

    /// The play field's text, for the view to edit.
    pub fn path_input_mut(&mut self) -> &mut String {
        &mut self.path_input
    }

    /// Play whatever the user typed into the play field, if anything.
    ///
    /// The field is cleared once something was added, so the next entry starts
    /// from an empty line.
    pub fn play_path_input(&mut self) -> usize {
        let text = self.path_input.trim().to_owned();
        if text.is_empty() {
            return 0;
        }

        let added = self.open_path(Path::new(&text));
        if added > 0 {
            self.path_input.clear();
        }

        added
    }

    /// Loads one file or folder into the queue, and plays it when idle.
    pub fn open_path(&mut self, path: &Path) -> usize {
        let added = self.playback.enqueue_path(path);
        self.start_if_idle(added);

        added
    }

    /// Loads several files or folders, and plays the first when idle.
    ///
    /// The command line uses this so `musiccraft album/` starts playing
    /// immediately, and folders are scanned one level deep.
    pub fn open_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;
        for path in paths {
            added += self.playback.enqueue_path(&path);
        }
        self.start_if_idle(added);

        added
    }

    /// Starts the selected track when something was just added and nothing is
    /// playing, so selecting a folder begins at its first track.
    fn start_if_idle(&mut self, added: usize) {
        if added > 0 && !self.playback.is_playing() {
            self.playback.play_current();
        }
    }

    /// The configured theme.
    pub fn theme(&self) -> ThemeMode {
        self.config.theme
    }

    /// Change the theme and schedule a save.
    pub fn set_theme(&mut self, theme: ThemeMode) {
        if self.config.theme != theme {
            self.config.theme = theme;
            self.config_dirty = true;
        }
    }

    /// The configured default volume, in the range `0.0..=1.0`.
    pub fn volume(&self) -> f32 {
        self.config.volume
    }

    /// Change the default volume and schedule a save.
    ///
    /// The value is clamped because the setting is written to disk, and it is
    /// handed to the player straight away so the change is audible.
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        if self.config.volume != volume {
            self.config.volume = volume;
            self.config_dirty = true;
        }
        self.playback.set_volume(volume);
    }

    /// Where settings are stored, when the platform provides a location.
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// The most recent settings failure, shown in the top bar.
    pub fn config_error(&self) -> Option<&str> {
        self.config_error.as_deref()
    }

    /// Hide the settings failure banner.
    pub fn dismiss_config_error(&mut self) {
        self.config_error = None;
    }

    /// Write the settings back to disk when something changed.
    ///
    /// Failures are reported through the top bar rather than panicking: losing
    /// a preference is annoying, not fatal, and the in-memory value is still
    /// correct for this session.
    fn persist(&mut self) {
        if !self.config_dirty {
            return;
        }

        let Some(path) = self.config_path.clone() else {
            // Nothing to write to; stop retrying every frame.
            self.config_dirty = false;
            return;
        };

        match self.config.save(&path) {
            Ok(()) => {
                tracing::debug!("settings written to {}", path.display());
            }
            Err(error) => {
                tracing::error!("failed to save settings to {}: {error}", path.display());
                self.config_error = Some(error.to_string());
            }
        }

        // Either way, do not retry on every frame: the user sees the banner and
        // the next change will try again.
        self.config_dirty = false;
    }
}

/// Format a track length the way a player shows it: `3:45`, or `1:02:03` once
/// a track runs past an hour.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

impl eframe::App for MusicCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.config.theme) {
            ui::theme::apply(ui.ctx(), self.config.theme);
            self.applied_theme = Some(self.config.theme);
        }

        // A track ends between frames and nothing calls back into the interface,
        // so the frame loop is what advances the queue.
        self.poll_playback();

        // The progress bar and the elapsed time only move while audio plays, so
        // ask for another frame for exactly as long as that is true.
        if self.transport() == TransportState::Playing {
            ui.ctx().request_repaint();
        }

        ui::render(self, ui);
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::audio::AudioEngine;

    #[test]
    fn transport_starts_unavailable() {
        assert_eq!(TransportState::default(), TransportState::Unavailable);
    }

    #[test]
    fn every_transport_state_has_a_label() {
        for state in [
            TransportState::Unavailable,
            TransportState::Stopped,
            TransportState::Playing,
            TransportState::Paused,
        ] {
            assert!(!state.label().is_empty(), "{state:?} has no label");
        }
    }

    #[test]
    fn startup_without_config_dir_still_yields_defaults() {
        let startup = Startup {
            config: AppConfig::default(),
            config_path: None,
            error: Some(AppError::NoConfigDir),
        };

        assert!(startup.config_path.is_none());
        assert!(startup.error.is_some());
        assert!((startup.config.volume - crate::config::DEFAULT_VOLUME).abs() < f32::EPSILON);
    }

    #[test]
    fn set_section_changes_the_active_section() {
        let mut app = app();
        assert_eq!(app.section(), NavSection::Library);

        app.set_section(NavSection::Settings);
        assert_eq!(app.section(), NavSection::Settings);
    }

    #[test]
    fn changing_the_theme_marks_the_config_dirty() {
        let mut app = app();
        assert!(!app.config_dirty);

        app.set_theme(ThemeMode::Light);
        assert_eq!(app.theme(), ThemeMode::Light);
        assert!(app.config_dirty, "a real change should schedule a save");
    }

    #[test]
    fn setting_the_theme_it_already_has_is_a_no_op() {
        let mut app = app();

        app.set_theme(app.theme());
        assert!(
            !app.config_dirty,
            "an unchanged value should not dirty the config"
        );
    }

    #[test]
    fn set_volume_clamps_marks_dirty_and_reaches_the_player() {
        let mut app = app();

        app.set_volume(2.0);
        assert_eq!(app.volume(), 1.0);
        assert!((app.playback().volume() - 1.0).abs() < f32::EPSILON);

        app.set_volume(-1.0);
        assert_eq!(app.volume(), 0.0);
        assert!((app.playback().volume() - 0.0).abs() < f32::EPSILON);

        assert!(app.config_dirty);
    }

    #[test]
    fn persist_without_a_path_clears_the_dirty_flag() {
        let mut app = app();
        app.set_theme(ThemeMode::Light);

        app.persist();

        assert!(!app.config_dirty, "nothing to write to, so stop retrying");
        assert!(app.config_error().is_none());
    }

    #[test]
    fn lengths_are_formatted_as_a_player_shows_them() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0:00");
        assert_eq!(format_duration(Duration::from_secs(9)), "0:09");
        assert_eq!(format_duration(Duration::from_secs(225)), "3:45");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1:00:00");
        assert_eq!(format_duration(Duration::from_secs(3723)), "1:02:03");
    }

    #[test]
    fn the_transport_is_unavailable_without_an_output_device() {
        let app = app();

        assert_eq!(app.transport(), TransportState::Unavailable);
        assert!(!app.playback().is_available());
    }

    #[test]
    fn opening_a_file_selects_it_and_reports_it_as_now_playing() {
        let mut app = app();

        let added = app.open_path(Path::new("/music/song.mp3"));

        assert_eq!(added, 1);
        assert_eq!(
            app.now_playing(),
            Some(NowPlaying {
                title: "song".to_owned(),
                position: Some("Track 1 of 1".to_owned()),
                duration: None,
            })
        );
    }

    #[test]
    fn a_missing_file_surfaces_an_error_instead_of_panicking() {
        let mut app = app();

        app.open_path(Path::new("/music/does-not-exist.mp3"));

        // The file was accepted into the queue, so playing it is what fails.
        // Without an output device the device error is what the user hears
        // about first, so the assertion is that a message reaches the bar at
        // all — the file is named again on a machine with sound.
        let message = app
            .playback_error()
            .expect("a failing track should be reported for the interface");
        assert!(!message.is_empty());

        // A failure leaves the app coherent: the queue survives it and the
        // transport is not left claiming to play.
        assert_eq!(app.playback().queue().len(), 1);
        assert!(!app.playback().is_playing());

        app.dismiss_playback_error();
        assert!(app.playback_error().is_none(), "the message is dismissable");
    }

    #[test]
    fn a_folder_with_no_audio_adds_nothing_and_stays_quiet() {
        let mut app = app();
        let folder = std::env::temp_dir().join("musiccraft-empty-folder");
        std::fs::create_dir_all(&folder).expect("a temp folder should be creatable");
        std::fs::write(folder.join("cover.png"), b"not audio").expect("the file should be written");

        let added = app.open_path(&folder);

        assert_eq!(added, 0);
        assert_eq!(app.now_playing(), None);
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn transport_controls_without_a_device_do_not_panic() {
        let mut app = app();

        app.toggle_playback();
        assert!(
            app.playback_error().is_some(),
            "starting an empty queue should be reported"
        );

        app.dismiss_playback_error();
        app.next_track();
        app.previous_track();
        app.seek_to_fraction(0.5);
        app.stop_playback();
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn playing_the_input_field_clears_it_once_something_was_added() {
        let mut app = app();
        *app.path_input_mut() = "  /music/song.mp3  ".to_owned();

        let added = app.play_path_input();

        assert_eq!(added, 1);
        assert_eq!(app.path_input(), "", "the field resets after a success");
    }

    #[test]
    fn playing_an_empty_input_field_does_nothing() {
        let mut app = app();
        *app.path_input_mut() = "   ".to_owned();

        assert_eq!(app.play_path_input(), 0);
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn a_player_reports_its_own_state_through_the_transport() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 0.5);
        playback.enqueue([PathBuf::from("/music/song.mp3")]);
        let app = MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            playback,
        );

        assert!((app.volume() - crate::config::DEFAULT_VOLUME).abs() < f32::EPSILON);
        assert_eq!(app.transport(), TransportState::Unavailable);
    }

    /// An application with a player that has no output device.
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
