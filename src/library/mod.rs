//! The music library: what files the user has, and what is in them.
//!
//! The module is split so each piece can be tested on its own:
//!
//! - [`Track`], [`Album`] and [`Artist`] are the domain models.
//! - [`metadata`] reads tags and cover art from a single file through `lofty`.
//! - [`scanner`] walks folders and turns them into tracks, on the calling
//!   thread or on a worker.
//! - [`artwork`] caches the cover art it finds.
//! - [`Library`] ties a scan to the database so a folder can be imported.

pub mod artwork;
pub mod metadata;
pub mod scanner;
pub mod track;

pub use metadata::AudioTags;
pub use scanner::{
    PROGRESS_INTERVAL, ScanError, ScanProgress, ScanReport, ScanRequest, ScanSession,
    collect_audio_files, scan, scan_with_cancel,
};
pub use track::{Album, Artist, PlaylistId, Track, TrackId, display_title};

use std::path::{Path, PathBuf};

use crate::database::LibraryRepository;
use crate::error::Result;

/// A scanned folder set together with the database it is stored in.
///
/// The type owns both halves so the interface only has to hold one value; it
/// never opens a second connection, because SQLite is happiest with one writer.
pub struct Library {
    repository: LibraryRepository,
    artwork_directory: PathBuf,
    folders: Vec<PathBuf>,
}

impl Library {
    /// Open (or create) the library database at `database_path`.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// file cannot be opened or migrated.
    pub fn open(
        database_path: impl AsRef<Path>,
        artwork_directory: impl Into<PathBuf>,
    ) -> Result<Self> {
        Ok(Self {
            repository: LibraryRepository::open(database_path)?,
            artwork_directory: artwork_directory.into(),
            folders: Vec::new(),
        })
    }

    /// An in-memory library, for tests and for the "no data directory" case.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// database cannot be created.
    pub fn in_memory(artwork_directory: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self {
            repository: LibraryRepository::open_in_memory()?,
            artwork_directory: artwork_directory.into(),
            folders: Vec::new(),
        })
    }

    /// The database behind the library.
    #[must_use]
    pub fn repository(&self) -> &LibraryRepository {
        &self.repository
    }

    /// The database behind the library, mutably, for playlist edits.
    pub fn repository_mut(&mut self) -> &mut LibraryRepository {
        &mut self.repository
    }

    /// The folders the user has asked to watch.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// Where extracted cover art is written.
    #[must_use]
    pub fn artwork_directory(&self) -> &Path {
        &self.artwork_directory
    }

    /// Replace the watched folders.
    pub fn set_folders(&mut self, folders: Vec<PathBuf>) {
        self.folders = folders;
    }

    /// The request that would scan the current folders.
    #[must_use]
    pub fn scan_request(&self) -> ScanRequest {
        ScanRequest {
            folders: self.folders.clone(),
            artwork_directory: self.artwork_directory.clone(),
            cache_artwork: true,
        }
    }

    /// Scan the watched folders on this thread and store what is found.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// imported tracks cannot be written; per-file read failures are reported
    /// inside the [`ScanReport`] instead.
    pub fn import(&mut self, observer: impl FnMut(&ScanProgress)) -> Result<ImportSummary> {
        let report = scan(&self.scan_request(), observer);
        self.store(report)
    }

    /// Write a finished scan into the database.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// transaction fails.
    pub fn store(&mut self, report: ScanReport) -> Result<ImportSummary> {
        let summary = self.repository.upsert_tracks(&report.tracks)?;
        let missing = self.repository.mark_disappeared()?;

        Ok(ImportSummary {
            scan: report,
            inserted: summary.inserted,
            updated: summary.updated,
            unchanged: summary.unchanged,
            missing,
        })
    }

    /// Every track in the library, ordered for display.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// query fails.
    pub fn tracks(&self) -> Result<Vec<Track>> {
        self.repository.tracks()
    }

    /// Every album in the library.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// query fails.
    pub fn albums(&self) -> Result<Vec<Album>> {
        self.repository.albums()
    }

    /// Every artist in the library.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// query fails.
    pub fn artists(&self) -> Result<Vec<Artist>> {
        self.repository.artists()
    }

    /// How many tracks are indexed.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`](crate::error::AppError::Database) when the
    /// query fails.
    pub fn track_count(&self) -> Result<usize> {
        self.repository.track_count()
    }
}

impl std::fmt::Debug for Library {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Library")
            .field("folders", &self.folders)
            .field("artwork_directory", &self.artwork_directory)
            .finish_non_exhaustive()
    }
}

/// What an import changed, alongside the scan that produced it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportSummary {
    /// The scan itself: what was found and what went wrong.
    pub scan: ScanReport,
    /// Tracks that were not in the database before.
    pub inserted: usize,
    /// Tracks whose stored fields changed.
    pub updated: usize,
    /// Tracks that were already up to date.
    pub unchanged: usize,
    /// Tracks whose files have disappeared since the last scan.
    pub missing: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str, files: &[&str]) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "musiccraft-library-{}-{}-{:?}",
            std::process::id(),
            name,
            std::thread::current().id()
        ));
        fs::remove_dir_all(&directory).ok();

        for file in files {
            let path = directory.join(file);
            fs::create_dir_all(path.parent().expect("parent")).expect("scratch directory");
            fs::write(&path, b"").expect("scratch file");
        }

        directory
    }

    #[test]
    fn importing_a_folder_puts_its_tracks_in_the_library() {
        let folder = scratch(
            "import",
            &["Boards of Canada/Roygbiv.mp3", "Aphex Twin/Xtal.flac"],
        );
        let mut library = Library::in_memory(folder.join("artwork")).expect("in-memory library");
        library.set_folders(vec![folder.clone()]);

        let summary = library.import(|_| {}).expect("import succeeds");

        assert_eq!(summary.inserted, 2);
        assert_eq!(summary.unchanged, 0);
        assert_eq!(library.track_count().expect("count"), 2);

        let titles: Vec<String> = library
            .tracks()
            .expect("tracks")
            .into_iter()
            .map(|track| track.title)
            .collect();
        assert_eq!(titles, vec!["Roygbiv".to_owned(), "Xtal".to_owned()]);

        fs::remove_dir_all(&folder).ok();
    }

    #[test]
    fn importing_twice_does_not_duplicate_tracks() {
        let folder = scratch("import-twice", &["a.mp3"]);
        let mut library = Library::in_memory(folder.join("artwork")).expect("in-memory library");
        library.set_folders(vec![folder.clone()]);

        library.import(|_| {}).expect("first import");
        let second = library.import(|_| {}).expect("second import");

        assert_eq!(second.inserted, 0);
        assert_eq!(second.unchanged, 1);
        assert_eq!(library.track_count().expect("count"), 1);

        fs::remove_dir_all(&folder).ok();
    }

    #[test]
    fn a_deleted_file_is_marked_missing_but_kept() {
        let folder = scratch("missing", &["a.mp3", "b.mp3"]);
        let mut library = Library::in_memory(folder.join("artwork")).expect("in-memory library");
        library.set_folders(vec![folder.clone()]);

        library.import(|_| {}).expect("first import");
        fs::remove_file(folder.join("b.mp3")).expect("delete a file");

        let summary = library.import(|_| {}).expect("second import");

        assert_eq!(summary.missing, 1);
        assert_eq!(library.track_count().expect("count"), 2);

        let missing: Vec<bool> = library
            .tracks()
            .expect("tracks")
            .into_iter()
            .map(|track| track.missing)
            .collect();
        assert_eq!(missing.iter().filter(|value| **value).count(), 1);

        fs::remove_dir_all(&folder).ok();
    }

    #[test]
    fn albums_and_artists_are_derived_from_the_tracks() {
        let folder = scratch("groups", &["a.mp3", "b.mp3"]);
        let mut library = Library::in_memory(folder.join("artwork")).expect("in-memory library");

        let mut first = Track::from_path(folder.join("a.mp3"));
        first.artist = Some("Aphex Twin".to_owned());
        first.album = Some("Selected Ambient Works".to_owned());

        let mut second = Track::from_path(folder.join("b.mp3"));
        second.artist = Some("Aphex Twin".to_owned());
        second.album = Some("Selected Ambient Works".to_owned());

        library
            .repository_mut()
            .upsert_track(&first)
            .expect("insert");
        library
            .repository_mut()
            .upsert_track(&second)
            .expect("insert");

        let albums = library.albums().expect("albums");
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Selected Ambient Works");
        assert_eq!(albums[0].track_count, 2);

        let artists = library.artists().expect("artists");
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].name, "Aphex Twin");
        assert_eq!(artists[0].track_count, 2);

        fs::remove_dir_all(&folder).ok();
    }
}
