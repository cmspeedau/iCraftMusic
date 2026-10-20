//! Settings the interface reads and writes: theme, volume, equalizer, output device, and the diagnostics behind Settings.

use super::MusicCraftApp;
use crate::audio::{AudioDevice, EqualizerSettings, MAX_CROSSFADE_SECONDS, RepeatMode};
use crate::config::{AppConfig, ThemeMode};
use crate::diagnostics::{self, Diagnostics};
use crate::error::AppError;
use std::path::Path;

impl MusicCraftApp {
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

    /// The repeat mode that was stored, as the config holds it.
    ///
    /// It is read through here rather than off the playback engine because the
    /// setting is what survives a restart: the two can only disagree for the
    /// moment between a change and `persist()`, and a test that compares the
    /// remembered choice with the engine's needs to see the former.
    pub fn configured_repeat(&self) -> RepeatMode {
        self.config.repeat
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

    /// The stored equalizer settings.
    pub fn equalizer(&self) -> &EqualizerSettings {
        &self.config.equalizer
    }

    /// Replace the equalizer settings and schedule a save.
    ///
    /// The value is handed to the player straight away, and because each track
    /// reads the settings as it runs, the change is heard on the track that is
    /// already playing.
    pub fn set_equalizer(&mut self, settings: EqualizerSettings) {
        if self.config.equalizer != settings {
            self.config.equalizer = settings.clone();
            self.config_dirty = true;
        }
        self.playback.set_equalizer(settings);
    }

    /// The stored crossfade length in seconds; zero means gapless.
    pub fn crossfade_seconds(&self) -> f32 {
        self.config.crossfade_seconds
    }

    /// Set the crossfade length and schedule a save.
    ///
    /// A change takes effect from the next track change: the track already
    /// lined up is on the device or decoded already, so it keeps the transition
    /// it was prepared for.
    pub fn set_crossfade_seconds(&mut self, seconds: f32) {
        let seconds = if seconds.is_finite() {
            seconds.clamp(0.0, MAX_CROSSFADE_SECONDS)
        } else {
            0.0
        };
        if self.config.crossfade_seconds != seconds {
            self.config.crossfade_seconds = seconds;
            self.config_dirty = true;
        }
        self.playback.set_crossfade(seconds);
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

    /// The output device the user chose, or `None` for the system default.
    #[must_use]
    pub fn chosen_output_device(&self) -> Option<&str> {
        self.config.output_device.as_deref()
    }

    /// The output devices the system offers.
    ///
    /// Read once and remembered: asking the audio backend is a system call per
    /// device, and a settings screen is drawn many times a second.
    pub fn output_devices(&mut self) -> &[AudioDevice] {
        if self.output_devices.is_none() {
            self.output_devices = Some(crate::audio::available_devices());
        }

        self.output_devices.as_deref().unwrap_or_default()
    }

    /// Read the output devices again, for a device plugged in since startup.
    pub fn rescan_output_devices(&mut self) {
        self.output_devices = None;
        self.settings_error = None;
    }

    /// Play through `device`, remembering the choice; `None` means the default.
    ///
    /// The setting is only stored once the switch has actually worked, so a
    /// device that cannot be opened does not become the preference the next run
    /// tries to honour. The failure is kept in [`MusicCraftApp::settings_error`]
    /// for the settings screen to show.
    ///
    /// Returns whether the player is now on the requested device.
    pub fn set_output_device(&mut self, device: Option<String>) -> bool {
        match self.playback.switch_output(device.as_deref()) {
            Ok(()) => {
                tracing::info!(
                    "playing through {}",
                    device.as_deref().unwrap_or("the default output device")
                );
                if self.config.output_device != device {
                    self.config.output_device = device;
                    self.config_dirty = true;
                }
                self.settings_error = None;
                true
            }
            Err(error) => {
                tracing::warn!("the output device could not be opened: {error}");
                self.settings_error = Some(error.to_string());
                false
            }
        }
    }

    /// Why the last settings action failed, ready to show.
    #[must_use]
    pub fn settings_error(&self) -> Option<&str> {
        self.settings_error.as_deref()
    }

    /// Hide the settings failure.
    pub fn dismiss_settings_error(&mut self) {
        self.settings_error = None;
    }

    /// Everything the Diagnostics screen reports about this installation.
    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        let browse = &self.browse;

        Diagnostics {
            app_name: crate::APP_NAME.to_owned(),
            version: crate::APP_VERSION.to_owned(),
            config_path: self.config_path.clone(),
            data_directory: AppConfig::data_directory().ok(),
            database_path: self.database_path.clone(),
            log_path: crate::logging::default_log_path(),
            schema_version: self
                .library
                .as_ref()
                .and_then(|library| library.schema_version().ok()),
            tracks: browse.track_count(),
            albums: browse.album_count(),
            artists: browse.artist_count(),
            playlists: self.playlist_count(),
            missing: browse.missing_count(),
            folders: self.config.library_folders.len(),
            output: self.playback.output_description(),
            output_id: self.playback.output_id().map(str::to_owned),
            library_error: self.library_error.clone(),
            config_error: self.config_error.clone(),
        }
    }

    /// Ask SQLite whether the library database is sound, and remember the
    /// answer for the Diagnostics screen.
    ///
    /// The check reads the whole file, so it runs once per press rather than
    /// per frame. Nothing is repaired: a damaged database is reported with the
    /// way out — the recovery steps live in the message.
    pub fn check_library_integrity(&mut self) {
        let database = self.database_path.clone();

        let result = match self.library.as_ref() {
            Some(library) => library.check_integrity(),
            None => Err(AppError::NoConfigDir),
        };

        let okay = result.as_ref().is_ok_and(|report| report.is_ok());
        if okay {
            tracing::info!("the library database passed its integrity check");
        } else {
            tracing::warn!("the library database reported a problem: {result:?}");
        }

        self.integrity = Some(diagnostics::integrity_message(&result, database.as_ref()));
    }

    /// What the last database check said, ready to show.
    #[must_use]
    pub fn library_integrity(&self) -> Option<&str> {
        self.integrity.as_deref()
    }

    /// Hide the last database check's report.
    pub fn dismiss_integrity(&mut self) {
        self.integrity = None;
    }

    /// Write the settings back to disk when something changed.
    ///
    /// Failures are reported through the top bar rather than panicking: losing
    /// a preference is annoying, not fatal, and the in-memory value is still
    /// correct for this session.
    pub(super) fn persist(&mut self) {
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
