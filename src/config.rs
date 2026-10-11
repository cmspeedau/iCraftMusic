//! Persistent application settings.
//!
//! Settings live in a single TOML file under the platform's per-user config
//! directory (`~/.config/musiccraft/config.toml` on Linux). Loading never
//! panics: a missing or malformed file falls back to defaults and reports the
//! failure back to the caller so it can be surfaced to the user.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

/// Everything MusicCraft remembers between runs.
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            volume: DEFAULT_VOLUME,
            library_folders: Vec::new(),
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
        };

        config.save(&path).expect("save succeeds");
        let loaded = AppConfig::load(&path).expect("load succeeds");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, config);
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
}
