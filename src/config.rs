//! Persistent application settings.
//!
//! Settings live in a single TOML file under the platform's per-user config
//! directory (`~/.config/icraftmusic/config.toml` on Linux). Loading never
//! panics: a missing or malformed file falls back to defaults and reports the
//! failure back to the caller so it can be surfaced to the user.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audio::{EqualizerSettings, MAX_CROSSFADE_SECONDS, RepeatMode};
use crate::error::{AppError, Result};

/// Volume applied when no preference has been stored yet.
pub const DEFAULT_VOLUME: f32 = 0.8;

/// Name of the settings file inside the config directory.
const CONFIG_FILE_NAME: &str = "config.toml";

/// Visual theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Light text on a dark background.
    #[default]
    Dark,
    /// Dark text on a light background.
    Light,
    /// Follow the operating system.
    System,
}

impl ThemeMode {
    /// Every theme, in the order shown in the settings view.
    pub const ALL: [ThemeMode; 3] = [Self::Dark, Self::Light, Self::System];

    /// Label shown in the user interface.
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::System => "System",
        }
    }
}

/// Everything iCraftMusic remembers between runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Selected theme.
    pub theme: ThemeMode,
    /// Default playback volume in the range `0.0..=1.0`.
    pub volume: f32,
    /// Folders the user has added to the music library.
    ///
    /// The list is persisted from Stage 0 onwards, but folder scanning is
    /// implemented in Stage 3.
    pub library_folders: Vec<PathBuf>,
    /// The output device to play through, as the audio backend identifies it.
    ///
    /// `None` means "whatever the desktop calls its default", which is what a
    /// user who never opened Settings expects and what follows their system
    /// settings. A stored identifier is only honoured when that device is still
    /// on the machine.
    pub output_device: Option<String>,
    /// Whether playback shuffles the queue.
    ///
    /// Remembered across runs because it is a way of listening rather than a
    /// property of one queue: a user who shuffles their library expects the next
    /// album they play to be shuffled too.
    pub shuffle: bool,
    /// What playback does when the current track ends.
    pub repeat: RepeatMode,
    /// How long one track fades into the next, in seconds.
    ///
    /// Zero means the tracks change over gaplessly — the next is queued behind
    /// the current one and starts the instant it ends. Anything above zero
    /// overlaps the two by that many seconds instead.
    pub crossfade_seconds: f32,
    /// The graphic equalizer applied to every track.
    pub equalizer: EqualizerSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            volume: DEFAULT_VOLUME,
            library_folders: Vec::new(),
            output_device: None,
            shuffle: false,
            repeat: RepeatMode::default(),
            crossfade_seconds: 0.0,
            equalizer: EqualizerSettings::default(),
        }
    }
}

impl AppConfig {
    /// Location of the settings file for the current user.
    ///
    /// Returns [`AppError::NoConfigDir`] on platforms without a usable config
    /// directory rather than guessing a path.
    pub fn default_path() -> Result<PathBuf> {
        let dirs =
            directories::ProjectDirs::from("", "", crate::APP_NAME).ok_or(AppError::NoConfigDir)?;
        Ok(dirs.config_dir().join(CONFIG_FILE_NAME))
    }

    /// Read and validate the settings stored at `path`.
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| AppError::io(path, source))?;
        Self::from_toml(path, &text)
    }

    /// Parse settings from `text`, treating `path` as the origin for errors.
    pub fn from_toml(path: &Path, text: &str) -> Result<Self> {
        let mut config: Self = toml::from_str(text).map_err(|source| AppError::Toml {
            path: path.to_path_buf(),
            source,
        })?;
        config.sanitize();
        Ok(config)
    }

    /// Load settings from `path`, falling back to defaults on any failure.
    ///
    /// The failure is returned rather than swallowed so the caller can report
    /// it; a missing file is not a failure and yields `None`.
    pub fn load_or_default(path: &Path) -> (Self, Option<AppError>) {
        if !path.exists() {
            return (Self::default(), None);
        }

        match Self::load(path) {
            Ok(config) => (config, None),
            Err(error) => (Self::default(), Some(error)),
        }
    }

    /// Write the settings to `path`, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|source| AppError::Serialize { source })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::io(parent, source))?;
        }

        fs::write(path, text).map_err(|source| AppError::io(path, source))?;
        Ok(())
    }

    /// Clamp values that would otherwise produce a broken interface.
    fn sanitize(&mut self) {
        if !self.volume.is_finite() {
            self.volume = DEFAULT_VOLUME;
        }
        self.volume = self.volume.clamp(0.0, 1.0);

        if !self.crossfade_seconds.is_finite() {
            self.crossfade_seconds = 0.0;
        }
        self.crossfade_seconds = self.crossfade_seconds.clamp(0.0, MAX_CROSSFADE_SECONDS);

        self.equalizer.sanitize();
    }

    /// Location of the per-user data directory.
    ///
    /// The library database and the cached cover art live here, apart from the
    /// settings file, so a user can back up or delete the library without
    /// losing their preferences.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NoConfigDir`] on platforms without a usable data
    /// directory rather than guessing a path.
    pub fn data_directory() -> Result<PathBuf> {
        let dirs =
            directories::ProjectDirs::from("", "", crate::APP_NAME).ok_or(AppError::NoConfigDir)?;
        Ok(dirs.data_dir().to_path_buf())
    }

    /// Start watching `folder`, returning whether it was not already there.
    ///
    /// Duplicates are refused rather than merged, because scanning the same
    /// folder twice would index every file under it twice.
    pub fn add_library_folder(&mut self, folder: impl Into<PathBuf>) -> bool {
        let folder = folder.into();
        if folder.as_os_str().is_empty() || self.library_folders.contains(&folder) {
            return false;
        }

        self.library_folders.push(folder);
        true
    }

    /// Stop watching `folder`, returning whether it was being watched.
    ///
    /// Only the watching stops: the tracks it contributed stay indexed until
    /// the user forgets them, and their files are untouched. A folder removed
    /// by mistake is put back by choosing it again.
    pub fn remove_library_folder(&mut self, folder: &Path) -> bool {
        let before = self.library_folders.len();
        self.library_folders.retain(|watched| watched != folder);

        self.library_folders.len() != before
    }

    /// The media home a brand-new install starts watching, if the platform
    /// reports a home directory.
    ///
    /// This is `~/iCraftMusic` — the folder the application owns, the way
    /// iTunes owns *iTunes Media*. It is computed rather than stored, which is
    /// what lets the Settings **Reset** button return the library to it no
    /// matter where the user had moved the home.
    pub fn default_media_home() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|dirs| dirs.home_dir().join(crate::APP_NAME))
    }

    /// The library's media home: its first watched folder, if it has one.
    ///
    /// The home is always element zero of [`AppConfig::library_folders`], so
    /// the Settings page can show one "Media folder location" while the list
    /// beneath it names the extra folders added alongside it.
    #[must_use]
    pub fn media_home(&self) -> Option<&Path> {
        self.library_folders.first().map(PathBuf::as_path)
    }

    /// Make `folder` the library's media home, keeping other watched folders.
    ///
    /// The old home stops being watched (it becomes the moved-from folder the
    /// move either re-points away from or, once files have been copied, can
    /// offer to clean up), and any copy of the new home that was watched as an
    /// extra folder is folded into the home so the list stays unique.
    pub fn set_media_home(&mut self, folder: PathBuf) {
        if self.library_folders.is_empty() {
            self.library_folders.push(folder);
            return;
        }

        self.library_folders[0] = folder.clone();
        let mut unique: Vec<PathBuf> = Vec::with_capacity(self.library_folders.len());
        for watched in self.library_folders.drain(..) {
            if !unique.contains(&watched) {
                unique.push(watched);
            }
        }
        self.library_folders = unique;
    }

    /// Return the media home to where a first boot would have placed it.
    ///
    /// Returns `false` when the platform has no home directory to return to;
    /// the watched folders are then left exactly as they were.
    pub fn reset_media_home(&mut self) -> bool {
        match Self::default_media_home() {
            Some(home) => {
                self.set_media_home(home);
                true
            }
            None => false,
        }
    }

    /// Seed the media home on a brand-new install, returning whether it seeded.
    ///
    /// Only an empty list is seeded: a user who has already chosen folders —
    /// or who deliberately watches nothing — is left exactly as they are. The
    /// folder itself is created later, by the application, and only when it is
    /// still this default (a folder the user picked that is missing is a
    /// mistake to report, not a thing to `mkdir` behind their back).
    pub fn ensure_media_home(&mut self) -> bool {
        if !self.library_folders.is_empty() {
            return false;
        }

        match Self::default_media_home() {
            Some(home) => {
                self.library_folders.push(home);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "musiccraft-config-{name}-{}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn defaults_are_sane() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemeMode::Dark);
        assert_eq!(config.volume, DEFAULT_VOLUME);
        assert!(config.library_folders.is_empty());
    }

    #[test]
    fn the_media_home_is_a_folder_called_after_the_application_in_the_home() {
        let Some(home) = AppConfig::default_media_home() else {
            return; // a platform with no home directory says so, and that is fine
        };

        assert_eq!(
            home.file_name().and_then(|name| name.to_str()),
            Some(crate::APP_NAME),
            "the default home is ~/{}",
            crate::APP_NAME
        );
    }

    #[test]
    fn a_first_boot_is_seeded_with_the_home_only_where_one_exists() {
        let mut config = AppConfig::default();

        let seeded = config.ensure_media_home();

        assert_eq!(seeded, AppConfig::default_media_home().is_some());
        if seeded {
            assert_eq!(
                config.media_home(),
                AppConfig::default_media_home().as_deref()
            );
        } else {
            assert!(config.library_folders.is_empty());
        }
    }

    #[test]
    fn seeding_leaves_a_config_that_already_watches_a_folder_alone() {
        let mut config = AppConfig::default();
        config.add_library_folder("/music/chosen");

        assert!(
            !config.ensure_media_home(),
            "a returning user is never re-seeded"
        );
        assert_eq!(config.media_home(), Some(Path::new("/music/chosen")));
    }

    #[test]
    fn setting_the_media_home_moves_it_and_keeps_the_other_folders() {
        let mut config = AppConfig::default();
        config.add_library_folder("/first/home");
        config.add_library_folder("/extra/folder");

        config.set_media_home(PathBuf::from("/second/home"));

        assert_eq!(config.media_home(), Some(Path::new("/second/home")));
        assert!(
            !config
                .library_folders
                .contains(&PathBuf::from("/first/home")),
            "the old home stops being watched when the home moves"
        );
        assert!(
            config
                .library_folders
                .contains(&PathBuf::from("/extra/folder"))
        );
    }

    #[test]
    fn setting_the_media_home_folds_in_a_folder_already_watched() {
        let mut config = AppConfig::default();
        config.add_library_folder("/first/home");
        config.add_library_folder("/new/home");

        config.set_media_home(PathBuf::from("/new/home"));

        assert_eq!(config.library_folders, vec![PathBuf::from("/new/home")]);
    }

    #[test]
    fn missing_file_falls_back_to_defaults_without_error() {
        let path = temp_path("missing");
        let _ = fs::remove_file(&path);

        let (config, error) = AppConfig::load_or_default(&path);

        assert_eq!(config, AppConfig::default());
        assert!(error.is_none(), "a missing file is not an error");
    }

    #[test]
    fn malformed_file_reports_an_error_and_keeps_defaults() {
        let path = temp_path("malformed");
        fs::write(&path, "theme = [not valid").expect("write fixture");

        let (config, error) = AppConfig::load_or_default(&path);
        let _ = fs::remove_file(&path);

        assert_eq!(config, AppConfig::default());
        assert!(matches!(error, Some(AppError::Toml { .. })));
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = temp_path("roundtrip");
        let config = AppConfig {
            theme: ThemeMode::Light,
            volume: 0.35,
            library_folders: vec![PathBuf::from("/home/example/Music")],
            output_device: Some("alsa:default".to_owned()),
            shuffle: true,
            repeat: RepeatMode::All,
            ..AppConfig::default()
        };

        config.save(&path).expect("save succeeds");
        let loaded = AppConfig::load(&path).expect("load succeeds");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, config);
    }

    #[test]
    fn crossfade_and_equalizer_survive_a_round_trip() {
        let path = temp_path("effects");
        let config = AppConfig {
            crossfade_seconds: 4.5,
            equalizer: EqualizerSettings::preset("Rock").expect("Rock is a preset"),
            ..AppConfig::default()
        };

        config.save(&path).expect("save succeeds");
        let loaded = AppConfig::load(&path).expect("load succeeds");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, config, "the audio effects are remembered");
        assert!(loaded.equalizer.enabled);
    }

    #[test]
    fn out_of_range_effects_are_clamped() {
        let path = PathBuf::from("config.toml");
        let text = r#"
            volume = 0.5
            crossfade_seconds = 99.0
            [equalizer]
            enabled = true
            preamp_db = 100.0
            bands = [40.0, -40.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        "#;

        let config = AppConfig::from_toml(&path, text).expect("parses");

        assert_eq!(config.crossfade_seconds, MAX_CROSSFADE_SECONDS);
        assert_eq!(config.equalizer.preamp_db, crate::audio::MAX_GAIN_DB);
        assert_eq!(config.equalizer.bands[0], crate::audio::MAX_GAIN_DB);
        assert_eq!(config.equalizer.bands[1], crate::audio::MIN_GAIN_DB);
    }

    #[test]
    fn a_settings_file_without_the_effects_still_loads() {
        // A config.toml written before these settings existed has no such keys.
        let path = PathBuf::from("config.toml");
        let config = AppConfig::from_toml(&path, "volume = 0.5\n").expect("parses");

        assert_eq!(
            config.crossfade_seconds, 0.0,
            "an absent crossfade means gapless"
        );
        assert!(
            !config.equalizer.enabled,
            "an absent equalizer means off and flat"
        );
        assert!(config.equalizer.is_flat());
    }

    #[test]
    fn a_settings_file_without_the_listening_modes_still_loads() {
        let path = PathBuf::from("config.toml");
        let config = AppConfig::from_toml(&path, "volume = 0.5\n").expect("parses");

        assert!(!config.shuffle, "shuffle starts off");
        assert_eq!(config.repeat, RepeatMode::Off, "and so does repeat");
    }

    #[test]
    fn out_of_range_volume_is_clamped() {
        let path = PathBuf::from("config.toml");

        let too_loud = AppConfig::from_toml(&path, "volume = 12.0\n").expect("parses");
        assert_eq!(too_loud.volume, 1.0);

        let too_quiet = AppConfig::from_toml(&path, "volume = -3.0\n").expect("parses");
        assert_eq!(too_quiet.volume, 0.0);
    }

    #[test]
    fn partial_file_keeps_defaults_for_missing_keys() {
        let path = PathBuf::from("config.toml");
        let config = AppConfig::from_toml(&path, "theme = \"light\"\n").expect("parses");

        assert_eq!(config.theme, ThemeMode::Light);
        assert_eq!(config.volume, DEFAULT_VOLUME);
    }

    #[test]
    fn default_path_ends_with_config_file_name() {
        if let Ok(path) = AppConfig::default_path() {
            assert_eq!(
                path.file_name().and_then(|n| n.to_str()),
                Some(CONFIG_FILE_NAME)
            );
        }
    }

    #[test]
    fn the_data_directory_is_separate_from_the_settings_file() {
        let Ok(data) = AppConfig::data_directory() else {
            return;
        };

        // `directories` lower-cases the application name on Linux, so the check
        // is on the last component rather than on the whole path.
        let last = data
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a directory name");
        assert_eq!(last.to_lowercase(), crate::APP_NAME.to_lowercase());

        if let Ok(settings) = AppConfig::default_path() {
            assert_ne!(
                data,
                settings.parent().expect("a parent directory"),
                "the database is not kept beside the settings"
            );
        }
    }

    #[test]
    fn a_folder_is_only_watched_once() {
        let mut config = AppConfig::default();

        assert!(config.add_library_folder("/music"));
        assert!(!config.add_library_folder("/music"), "no duplicates");
        assert!(config.add_library_folder("/other"));
        assert_eq!(config.library_folders.len(), 2);

        assert!(
            !config.add_library_folder(""),
            "an empty path is not a folder"
        );
    }

    #[test]
    fn a_watched_folder_can_be_dropped_again() {
        let mut config = AppConfig::default();
        config.add_library_folder("/music");
        config.add_library_folder("/other");

        assert!(config.remove_library_folder(Path::new("/music")));
        assert_eq!(config.library_folders, vec![PathBuf::from("/other")]);

        assert!(
            !config.remove_library_folder(Path::new("/music")),
            "removing what is not there changes nothing"
        );
        assert_eq!(config.library_folders.len(), 1);
    }

    #[test]
    fn the_chosen_output_device_survives_a_round_trip() {
        let path = temp_path("device");
        let config = AppConfig {
            output_device: Some("alsa:hdmi".to_owned()),
            ..AppConfig::default()
        };

        config.save(&path).expect("save succeeds");
        let loaded = AppConfig::load(&path).expect("load succeeds");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, config);
        assert_eq!(loaded.output_device.as_deref(), Some("alsa:hdmi"));
    }

    #[test]
    fn settings_written_before_the_device_was_chosen_keep_the_default_output() {
        // An older config.toml has no such key at all.
        let path = PathBuf::from("config.toml");
        let config = AppConfig::from_toml(&path, "theme = \"light\"\nvolume = 0.5\n")
            .expect("a file without the key still parses");

        assert_eq!(
            config.output_device, None,
            "no stored device means the system default"
        );
    }
}
