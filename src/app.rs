//! Application state and the `eframe` integration.
//!
//! Stage 1 deliberately keeps this small. The shell renders navigation, content
//! panes and the playback bar, while real audio, library scanning and playlists
//! arrive in later stages. Everything the user can change here is persisted
//! through [`AppConfig`], so the settings written in Stage 1 stay meaningful as
//! the rest of the application lands.

use std::path::PathBuf;

use eframe::CreationContext;

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
/// Stage 1 never plays audio, so the transport starts — and stays — in
/// [`TransportState::Unavailable`]. The other variants exist now so the bar can
/// be wired up in Stage 2 without redesigning the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportState {
    /// No audio backend is attached yet.
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
    /// Track title.
    pub title: String,
    /// Performing artist.
    pub artist: String,
    /// Album the track belongs to.
    pub album: String,
}

/// Top-level application state.
pub struct MusicCraftApp {
    config: AppConfig,
    config_path: Option<PathBuf>,
    /// Set whenever a setting changes, cleared once it is written to disk.
    config_dirty: bool,
    config_error: Option<String>,
    section: NavSection,
    transport: TransportState,
    now_playing: Option<NowPlaying>,
    /// The theme currently installed into `egui`, so it is only re-applied when
    /// the preference actually changes.
    applied_theme: Option<ThemeMode>,
}

impl MusicCraftApp {
    /// Build the application from the result of [`Startup::load`].
    pub fn new(_creation_context: &CreationContext<'_>, startup: Startup) -> Self {
        let error = startup.error.as_ref().map(ToString::to_string);

        Self {
            config: startup.config,
            config_path: startup.config_path,
            config_dirty: false,
            config_error: error,
            section: NavSection::default(),
            transport: TransportState::default(),
            now_playing: None,
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

    /// The persistence state of the playback bar.
    pub fn transport(&self) -> TransportState {
        self.transport
    }

    /// The track shown in the playback bar, if any.
    pub fn now_playing(&self) -> Option<&NowPlaying> {
        self.now_playing.as_ref()
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
    /// The value is clamped because the setting is written to disk and later
    /// stages will feed it straight to the audio backend.
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        if self.config.volume != volume {
            self.config.volume = volume;
            self.config_dirty = true;
        }
    }

    /// Where settings are stored, when the platform provides a location.
    pub fn config_path(&self) -> Option<&std::path::Path> {
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

impl eframe::App for MusicCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.config.theme) {
            ui::theme::apply(ui.ctx(), self.config.theme);
            self.applied_theme = Some(self.config.theme);
        }

        ui::render(self, ui);
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
