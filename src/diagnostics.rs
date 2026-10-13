//! What the application knows about itself, and how it answers "is anything
//! wrong?".
//!
//! Two things live here. [`Diagnostics`] is a snapshot of the installation —
//! where the settings, the database, the cover art and the log are, what
//! version of the schema the database is at, and what the audio side is playing
//! through. It is gathered by the application, which owns all of it, and
//! rendered by the Settings screen as plain lines a user can read out when
//! asking for help.
//!
//! [`integrity_message`] turns the database check into the one sentence the
//! Diagnostics screen shows, which is deliberately a pure function: the
//! wording of "your library is fine" and "your library is damaged, here is what
//! to do" is worth a test, and the failure paths that produce them cannot be
//! forced from a test that goes through SQLite.

use std::path::PathBuf;

use crate::database::IntegrityReport;
use crate::error::AppError;

/// How many problem lines a check's summary shows before it stops listing them.
///
/// A damaged database can report hundreds of problems, and a wall of them tells
/// the user less than the first few plus a count.
const PROBLEMS_SHOWN: usize = 3;

/// A snapshot of the installation, ready to be shown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    /// Application name.
    pub app_name: String,
    /// Application version, as `Cargo.toml` declares it.
    pub version: String,
    /// Where `config.toml` lives, when the platform provides a location.
    pub config_path: Option<PathBuf>,
    /// Where the database, the cover-art cache and the log live.
    pub data_directory: Option<PathBuf>,
    /// The library database file.
    pub database_path: Option<PathBuf>,
    /// The log file this run is writing to.
    pub log_path: Option<PathBuf>,
    /// Schema version the open database is at.
    pub schema_version: Option<i64>,
    /// Indexed tracks, albums and artists.
    pub tracks: usize,
    pub albums: usize,
    pub artists: usize,
    /// Playlists in the database.
    pub playlists: usize,
    /// Tracks whose files have gone.
    pub missing: usize,
    /// Folders being watched.
    pub folders: usize,
    /// The output device description, e.g. `48.0 kHz · 2 ch`.
    pub output: Option<String>,
    /// The identifier of the device in use, when the backend reports one.
    pub output_id: Option<String>,
    /// Why the library is unavailable, when it is.
    pub library_error: Option<String>,
    /// Why the settings file could not be read, when it could not.
    pub config_error: Option<String>,
}

impl Diagnostics {
    /// The snapshot as label-and-value lines, in the order they are shown.
    ///
    /// A path that does not apply on this platform is left out rather than
    /// shown as `None`: an empty line in a diagnostics list reads like a
    /// missing file, which is exactly the sort of red herring this screen
    /// exists to avoid.
    #[must_use]
    pub fn lines(&self) -> Vec<(String, String)> {
        let mut lines = vec![(
            "Application".to_owned(),
            format!("{} {}", self.app_name, self.version),
        )];

        if let Some(path) = &self.config_path {
            lines.push(("Settings file".to_owned(), path.display().to_string()));
        }

        if let Some(path) = &self.data_directory {
            lines.push(("Data directory".to_owned(), path.display().to_string()));
        }

        if let Some(path) = &self.database_path {
            lines.push(("Library database".to_owned(), path.display().to_string()));
        }

        if let Some(path) = &self.log_path {
            lines.push(("Log file".to_owned(), path.display().to_string()));
        }

        if let Some(version) = self.schema_version {
            lines.push(("Library schema".to_owned(), format!("version {version}")));
        }

        lines.push((
            "Indexed".to_owned(),
            format!(
                "{} tracks · {} albums · {} artists",
                self.tracks, self.albums, self.artists
            ),
        ));
        lines.push(("Playlists".to_owned(), self.playlists.to_string()));
        lines.push(("Watched folders".to_owned(), self.folders.to_string()));

        if self.missing > 0 {
            lines.push((
                "Missing files".to_owned(),
                format!("{} indexed tracks have no file on disk", self.missing),
            ));
        }

        match &self.output {
            Some(description) => lines.push((
                "Audio output".to_owned(),
                match &self.output_id {
                    Some(id) => format!("{description} ({id})"),
                    None => description.clone(),
                },
            )),
            None => lines.push((
                "Audio output".to_owned(),
                "no output device could be opened".to_owned(),
            )),
        }

        if let Some(error) = &self.library_error {
            lines.push(("Library".to_owned(), error.clone()));
        }

        if let Some(error) = &self.config_error {
            lines.push(("Settings".to_owned(), error.clone()));
        }

        lines
    }
}

/// The sentence the Diagnostics screen shows after checking the database.
///
/// A damaged library is the one failure this application cannot repair on its
/// own — the honest answer is what is wrong, where the file is, and what to do
/// about it — so the advice is part of the message rather than a separate
/// screen nobody opens.
#[must_use]
pub fn integrity_message(
    result: &Result<IntegrityReport, AppError>,
    database: Option<&PathBuf>,
) -> String {
    let where_it_is = database
        .map(|path| format!(" ({})", path.display()))
        .unwrap_or_default();

    match result {
        Ok(report) if report.is_ok() => {
            "The library database is sound: every table and index checks out.".to_owned()
        }
        Ok(report) => {
            let shown: Vec<String> = report
                .problems
                .iter()
                .take(PROBLEMS_SHOWN)
                .cloned()
                .collect();
            let rest = report.problems.len().saturating_sub(shown.len());

            let mut message = format!(
                "The library database is damaged{}: {}.",
                where_it_is,
                shown.join("; ")
            );

            if rest > 0 {
                message.push_str(&format!(" (and {rest} more)"));
            }

            message.push_str(
                " Nothing has been changed. Move the file aside and rescan your folders to rebuild \
                 the library from your music; a copy taken before the last upgrade sits beside it \
                 as `*.pre-v*.bak`.",
            );

            message
        }
        Err(error) => format!(
            "The library database could not be checked{}: {error}. It may not be a SQLite database \
             at all; moving it aside and rescanning rebuilds the library from your music.",
            where_it_is
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> Diagnostics {
        Diagnostics {
            app_name: "MusicCraft".to_owned(),
            version: "0.1.0".to_owned(),
            config_path: Some(PathBuf::from("/home/you/.config/musiccraft/config.toml")),
            data_directory: Some(PathBuf::from("/home/you/.local/share/musiccraft")),
            database_path: Some(PathBuf::from(
                "/home/you/.local/share/musiccraft/library.sqlite",
            )),
            log_path: Some(PathBuf::from(
                "/home/you/.local/share/musiccraft/logs/musiccraft.log",
            )),
            schema_version: Some(3),
            tracks: 18_432,
            albums: 1_204,
            artists: 402,
            playlists: 6,
            missing: 0,
            folders: 2,
            output: Some("48.0 kHz · 2 ch".to_owned()),
            output_id: Some("alsa:default".to_owned()),
            library_error: None,
            config_error: None,
        }
    }

    /// The value shown for a label, or `None` when the line is absent.
    fn value<'a>(lines: &'a [(String, String)], label: &str) -> Option<&'a str> {
        lines
            .iter()
            .find(|(name, _)| name == label)
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn the_snapshot_names_the_build_and_every_place_data_lives() {
        let lines = snapshot().lines();

        assert_eq!(value(&lines, "Application"), Some("MusicCraft 0.1.0"));
        assert_eq!(
            value(&lines, "Settings file"),
            Some("/home/you/.config/musiccraft/config.toml")
        );
        assert_eq!(
            value(&lines, "Library database"),
            Some("/home/you/.local/share/musiccraft/library.sqlite")
        );
        assert_eq!(
            value(&lines, "Log file"),
            Some("/home/you/.local/share/musiccraft/logs/musiccraft.log")
        );
        assert_eq!(value(&lines, "Library schema"), Some("version 3"));
    }

    #[test]
    fn the_snapshot_counts_the_library() {
        let lines = snapshot().lines();

        assert_eq!(
            value(&lines, "Indexed"),
            Some("18432 tracks · 1204 albums · 402 artists")
        );
        assert_eq!(value(&lines, "Playlists"), Some("6"));
        assert_eq!(value(&lines, "Watched folders"), Some("2"));
        assert_eq!(
            value(&lines, "Audio output"),
            Some("48.0 kHz · 2 ch (alsa:default)")
        );
    }

    #[test]
    fn paths_that_do_not_apply_are_left_out_rather_than_shown_empty() {
        let lines = Diagnostics {
            app_name: "MusicCraft".to_owned(),
            version: "0.1.0".to_owned(),
            ..Diagnostics::default()
        }
        .lines();

        assert!(value(&lines, "Settings file").is_none());
        assert!(value(&lines, "Library database").is_none());
        assert_eq!(
            value(&lines, "Audio output"),
            Some("no output device could be opened"),
            "the audio line always says something: silence there reads like a bug"
        );
    }

    #[test]
    fn a_library_that_did_not_open_says_so_in_its_own_line() {
        let lines = Diagnostics {
            library_error: Some("the library could not be opened: disk is full".to_owned()),
            ..snapshot()
        }
        .lines();

        assert_eq!(
            value(&lines, "Library"),
            Some("the library could not be opened: disk is full")
        );
    }

    #[test]
    fn a_missing_count_only_appears_when_something_is_missing() {
        assert!(value(&snapshot().lines(), "Missing files").is_none());

        let lines = Diagnostics {
            missing: 3,
            ..snapshot()
        }
        .lines();
        assert!(
            value(&lines, "Missing files").is_some_and(|value| value.contains('3')),
            "the missing files are worth naming once there are any"
        );
    }

    #[test]
    fn a_sound_database_is_reported_as_sound() {
        let report = IntegrityReport { problems: vec![] };

        let message = integrity_message(&Ok(report), None);

        assert!(message.contains("sound"), "{message}");
        assert!(
            !message.contains("damaged"),
            "a healthy database must not be described as damaged: {message}"
        );
    }

    #[test]
    fn a_damaged_database_names_the_file_and_says_what_to_do() {
        let report = IntegrityReport {
            problems: vec![
                "row 4 missing from index tracks_artist_idx".to_owned(),
                "row 9 missing".to_owned(),
            ],
        };
        let path = PathBuf::from("/data/library.sqlite");

        let message = integrity_message(&Ok(report), Some(&path));

        assert!(message.contains("/data/library.sqlite"), "{message}");
        assert!(message.contains("missing from index"), "{message}");
        assert!(
            message.contains("rescan"),
            "the user is told how to rebuild: {message}"
        );
        assert!(
            message.contains("Nothing has been changed"),
            "…and that the file has not been touched: {message}"
        );
    }

    #[test]
    fn a_long_list_of_problems_is_summarised_rather_than_dumped() {
        let report = IntegrityReport {
            problems: (0..40).map(|index| format!("problem {index}")).collect(),
        };

        let message = integrity_message(&Ok(report), None);

        assert!(message.contains("problem 0"), "{message}");
        assert!(
            message.contains("and 37 more"),
            "the rest are counted, not listed: {message}"
        );
        assert!(
            !message.contains("problem 39"),
            "a wall of problems helps nobody: {message}"
        );
    }

    #[test]
    fn a_database_that_cannot_be_checked_at_all_says_so() {
        let failure = Err(AppError::NoConfigDir);

        let message = integrity_message(&failure, None);

        assert!(message.contains("could not be checked"), "{message}");
        assert!(
            message.contains("rescan"),
            "even an unreadable file has a way forward: {message}"
        );
    }
}
