//! The library database: the schema, its migrations, and the repository that
//! reads and writes tracks.
//!
//! The database is a cache of what the files on disk say. Nothing here is
//! authoritative on its own: deleting a row never deletes a file, and a fresh
//! scan can always rebuild the whole thing.

pub mod repository;
pub mod schema;

pub use repository::{LibraryRepository, UpsertOutcome, UpsertSummary};
pub use schema::{IntegrityReport, Migration, SCHEMA_VERSION, migrations};

use std::path::Path;

use crate::error::Result;

/// The database file's name inside the user's data directory.
pub const DATABASE_FILE: &str = "library.sqlite";

/// Open the library database stored under `data_directory`.
///
/// The directory is created if it does not exist.
///
/// # Errors
///
/// Returns [`AppError::Database`](crate::error::AppError::Database) when the
/// file cannot be opened, or [`AppError::Io`](crate::error::AppError::Io) when
/// the directory cannot be created.
pub fn open(data_directory: impl AsRef<Path>) -> Result<LibraryRepository> {
    let path = data_directory.as_ref().join(DATABASE_FILE);
    tracing::debug!("opening the library database at {}", path.display());

    LibraryRepository::open(path)
}

/// Open a private, throw-away database for tests and previews.
///
/// # Errors
///
/// Returns [`AppError::Database`](crate::error::AppError::Database) when the
/// schema cannot be applied.
pub fn open_in_memory() -> Result<LibraryRepository> {
    LibraryRepository::open_in_memory()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_database_lands_in_the_data_directory() {
        let directory = std::env::temp_dir().join(format!("musiccraft-db-{}", std::process::id()));

        {
            let repository = open(&directory).expect("open the database");
            assert_eq!(repository.track_count().expect("count"), 0);
        }

        assert!(
            directory.join(DATABASE_FILE).is_file(),
            "the database file should be created under the data directory"
        );

        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn reopening_keeps_the_tracks() {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-reopen-{}", std::process::id()));
        std::fs::remove_dir_all(&directory).ok();

        {
            let mut repository = open(&directory).expect("open");
            let mut track = crate::library::Track::from_path("/music/a.mp3");
            track.title = "Roygbiv".to_owned();
            repository.upsert_track(&track).expect("insert");
        }

        let repository = open(&directory).expect("reopen");
        assert_eq!(repository.track_count().expect("count"), 1);

        std::fs::remove_dir_all(&directory).ok();
    }
}
