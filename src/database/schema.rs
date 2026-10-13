//! The library's SQLite schema and its migrations.
//!
//! The schema is versioned so a database written by an older build can be
//! brought forward without being thrown away. Each migration runs once, inside
//! a transaction, and the version it carries is recorded in `schema_version`
//! before the next one starts — so an interrupted upgrade leaves the database at
//! the last version it fully applied.

use rusqlite::Connection;

use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// The schema version this build writes.
pub const SCHEMA_VERSION: i64 = 4;

/// One step of the schema's history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    /// The version the migration produces.
    pub version: i64,
    /// A short description, for logs.
    pub description: &'static str,
    /// The statements to run.
    pub sql: &'static str,
}

/// Every migration, oldest first.
///
/// Never edit a migration that has shipped: add a new one instead, or existing
/// libraries will disagree with fresh ones.
#[must_use]
pub fn migrations() -> &'static [Migration] {
    &[
        Migration {
            version: 1,
            description: "tracks, playlists and playlist membership",
            sql: SQL_V1,
        },
        Migration {
            version: 2,
            description: "playlist names are unique ignoring case",
            sql: SQL_V2,
        },
        Migration {
            version: 3,
            description: "a track can be edited without its file being rewritten",
            sql: SQL_V3,
        },
        Migration {
            version: 4,
            description: "a track remembers how many times it has been played",
            sql: SQL_V4,
        },
    ]
}

const SQL_V1: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version    INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tracks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT    NOT NULL UNIQUE,
    title        TEXT    NOT NULL,
    artist       TEXT,
    album        TEXT,
    album_artist TEXT,
    genre        TEXT,
    track_number INTEGER,
    disc_number  INTEGER,
    duration_ms  INTEGER,
    artwork_path TEXT,
    size_bytes   INTEGER,
    modified_at  INTEGER,
    missing      INTEGER NOT NULL DEFAULT 0,
    added_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS tracks_artist_idx ON tracks (artist COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS tracks_album_idx  ON tracks (album COLLATE NOCASE, album_artist COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL UNIQUE,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists (id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks (id)    ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, position)
);

CREATE INDEX IF NOT EXISTS playlist_tracks_track_idx ON playlist_tracks (track_id);
";

const SQL_V2: &str = "
CREATE UNIQUE INDEX IF NOT EXISTS playlists_name_idx ON playlists (name COLLATE NOCASE);
";

const SQL_V3: &str = "
ALTER TABLE tracks ADD COLUMN tags_dirty INTEGER NOT NULL DEFAULT 0;
";

const SQL_V4: &str = "
ALTER TABLE tracks ADD COLUMN play_count INTEGER NOT NULL DEFAULT 0;
";

/// What `PRAGMA integrity_check` said about a database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    /// One line per problem found, empty when the database is sound.
    pub problems: Vec<String>,
}

impl IntegrityReport {
    /// Whether the database passed the check.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.problems.is_empty()
    }

    /// A one-line description, for a log line or a button's report.
    #[must_use]
    pub fn summary(&self) -> String {
        match self.problems.len() {
            0 => "the library database is sound".to_owned(),
            1 => "the library database reported one problem".to_owned(),
            count => format!("the library database reported {count} problems"),
        }
    }
}

/// Ask SQLite to check the database's own consistency.
///
/// The check reads every page and every index, so it is a deliberate action
/// rather than something done on every start: it is what the user runs when
/// something looks wrong, and what tells them whether a corrupt file is the
/// reason. A database that cannot be read at all comes back as the database
/// error rather than as a report.
///
/// # Errors
///
/// Returns [`AppError::Database`] when the check itself cannot run.
pub fn integrity_check(connection: &Connection) -> Result<IntegrityReport> {
    let mut statement = connection
        .prepare("PRAGMA integrity_check")
        .map_err(AppError::database)?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(AppError::database)?;

    let lines: Vec<String> = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::database)?;

    // A sound database answers with a single row reading `ok`; anything else is
    // a problem to show the user, whatever it says.
    Ok(IntegrityReport {
        problems: lines.into_iter().filter(|line| line != "ok").collect(),
    })
}

/// Where a copy of the database at `path` would be written.
///
/// The copy sits beside the original so it is found by anyone who goes looking
/// for the library, and it never overwrites an earlier copy: each migration
/// that needs one gets a file of its own, which is what makes the oldest copy —
/// taken from the version the user actually ran — still available.
#[must_use]
pub fn backup_path(path: &Path, version: i64) -> PathBuf {
    let mut candidate = path
        .to_path_buf()
        .with_extension(format!("{}.pre-v{version}.bak", extension_of(path)));

    let mut serial = 2;
    while candidate.exists() {
        candidate = path.to_path_buf().with_extension(format!(
            "{}.pre-v{version}-{serial}.bak",
            extension_of(path)
        ));
        serial += 1;
    }

    candidate
}

/// The extension of `path` as a string, or `sqlite` when it has none.
fn extension_of(path: &Path) -> String {
    path.extension().map_or_else(
        || "sqlite".to_owned(),
        |extension| extension.to_string_lossy().into_owned(),
    )
}

/// Copy the database to `destination`, leaving the original in place.
///
/// `VACUUM INTO` is used rather than a file copy because it asks SQLite itself
/// to write a consistent database: a copy taken with `fs::copy` while a
/// write-ahead log is in play can miss the pages that are still in the log.
///
/// # Errors
///
/// Returns [`AppError::Database`] when the copy cannot be written — a full or
/// read-only directory, or a destination that already exists.
pub fn backup_to(connection: &Connection, destination: &Path) -> Result<()> {
    connection
        .execute("VACUUM INTO ?1", [destination.to_string_lossy()])
        .map(|_| ())
        .map_err(AppError::database)
}

/// Bring `connection` up to [`SCHEMA_VERSION`], taking a copy first.
///
/// The copy is only taken when there is something to lose: a database written
/// by an older build, with migrations still to apply. A fresh database has
/// nothing worth keeping, and an up-to-date one is not rewritten.
///
/// # Errors
///
/// Returns [`AppError::Database`] when a migration cannot be applied; the
/// migration's transaction is rolled back, its recorded version is left
/// untouched, and a copy of the database as it was sits beside it.
pub fn apply(connection: &mut Connection) -> Result<()> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(AppError::database)?;

    // The version table predates the migrations, so it is created separately.
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                 version    INTEGER PRIMARY KEY,
                 applied_at TEXT NOT NULL DEFAULT (datetime('now'))
             );",
        )
        .map_err(AppError::database)?;

    let current = current_version(connection)?;
    tracing::debug!("library schema is at version {current}");

    if current > 0
        && current < SCHEMA_VERSION
        && let Err(error) = back_up(connection, current)
    {
        // Not being able to take a copy is not a reason to refuse to open the
        // library — the user's music is still there and the migration is
        // reversible by hand — so this is reported and the upgrade goes ahead.
        tracing::warn!("could not back up the library before migrating: {error}");
    }

    for migration in migrations() {
        if migration.version <= current {
            continue;
        }

        let transaction = connection.transaction().map_err(AppError::database)?;
        transaction
            .execute_batch(migration.sql)
            .map_err(AppError::database)?;
        transaction
            .execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                [migration.version],
            )
            .map_err(AppError::database)?;
        transaction.commit().map_err(AppError::database)?;

        tracing::info!(
            "applied library migration {} ({})",
            migration.version,
            migration.description
        );
    }

    Ok(())
}

/// The highest version recorded in the database, or zero for a fresh file.
///
/// # Errors
///
/// Returns [`AppError::Database`] when the version row cannot be read.
pub fn current_version(connection: &Connection) -> Result<i64> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(AppError::database)
}

/// Copy a database file beside itself, before a migration rewrites it.
///
/// Returns the path the copy was written to, or `None` when there is no file to
/// copy — an in-memory database is thrown away with the process anyway.
///
/// # Errors
///
/// Returns [`AppError::Database`] when the copy cannot be written.
fn back_up(connection: &Connection, version: i64) -> Result<Option<PathBuf>> {
    let Some(source) = connection.path() else {
        return Ok(None);
    };
    let source = Path::new(source);

    let destination = backup_path(source, version);
    backup_to(connection, &destination)?;

    tracing::info!(
        "copied the library database to {} before migrating from version {version}",
        destination.display()
    );

    Ok(Some(destination))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applying_migrations_sets_the_current_version() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");

        apply(&mut connection).expect("migrations run");

        assert_eq!(
            current_version(&connection).expect("version"),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn applying_migrations_twice_changes_nothing() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");

        apply(&mut connection).expect("first run");
        apply(&mut connection).expect("second run");

        assert_eq!(
            current_version(&connection).expect("version"),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn every_migration_has_a_unique_ascending_version() {
        let versions: Vec<i64> = migrations()
            .iter()
            .map(|migration| migration.version)
            .collect();
        let mut sorted = versions.clone();
        sorted.sort_unstable();
        sorted.dedup();

        assert_eq!(versions, sorted);
        assert_eq!(versions.last().copied(), Some(SCHEMA_VERSION));
    }

    #[test]
    fn every_track_column_the_code_reads_exists_after_migrating() {
        // `TRACK_COLUMNS` names the columns the mappers read by position, and a
        // migration that forgot one would otherwise show up as a runtime query
        // failure rather than as a failing build.
        let mut connection = Connection::open_in_memory().expect("in-memory database");
        apply(&mut connection).expect("migrations run");

        let statement = connection
            .prepare("SELECT * FROM tracks")
            .expect("prepare the row");
        let names: Vec<String> = statement
            .column_names()
            .into_iter()
            .map(str::to_owned)
            .collect();

        for column in ["tags_dirty", "play_count"] {
            assert!(
                names.iter().any(|name| name == column),
                "the migrated table has no {column} column: {names:?}"
            );
        }
    }

    #[test]
    fn a_database_from_first_release_gains_a_zero_play_count() {
        // Migration 4 adds the column, so existing rows must come out at zero
        // rather than at a missing-column error.
        let directory = scratch("migrate-play-count");
        let path = directory.join("library.sqlite");

        {
            let connection = Connection::open(&path).expect("open");
            connection.execute_batch(SQL_V1).expect("first schema");
            connection.execute_batch(SQL_V2).expect("second schema");
            connection.execute_batch(SQL_V3).expect("third schema");
            connection
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS schema_version (
                         version    INTEGER PRIMARY KEY,
                         applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                     );",
                )
                .expect("version table");
            connection
                .execute("INSERT INTO schema_version (version) VALUES (3)", [])
                .expect("record the old version");
            connection
                .execute(
                    "INSERT INTO tracks (path, title) VALUES ('/music/a.mp3', 'Roygbiv')",
                    [],
                )
                .expect("track");
        }

        let mut connection = Connection::open(&path).expect("reopen");
        apply(&mut connection).expect("migrations run");

        let (title, plays): (String, i64) = connection
            .query_row("SELECT title, play_count FROM tracks", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("the row survived the migration");

        assert_eq!(title, "Roygbiv");
        assert_eq!(plays, 0, "an unplayed track starts at zero");
    }

    #[test]
    fn the_schema_has_the_expected_tables() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");
        apply(&mut connection).expect("migrations run");

        for table in ["tracks", "playlists", "playlist_tracks"] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table lookup");

            assert_eq!(count, 1, "missing table {table}");
        }
    }

    #[test]
    fn a_path_can_only_be_indexed_once() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");
        apply(&mut connection).expect("migrations run");

        let insert = "INSERT INTO tracks (path, title) VALUES (?1, ?2)";
        connection
            .execute(insert, ("/music/a.mp3", "Roygbiv"))
            .expect("first insert");

        let duplicate = connection.execute(insert, ("/music/a.mp3", "Again"));

        assert!(
            duplicate.is_err(),
            "the unique path index must reject a duplicate"
        );
    }

    #[test]
    fn playlist_membership_is_removed_with_its_playlist() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");
        apply(&mut connection).expect("migrations run");

        connection
            .execute(
                "INSERT INTO tracks (path, title) VALUES ('/music/a.mp3', 'A')",
                [],
            )
            .expect("track");
        connection
            .execute("INSERT INTO playlists (name) VALUES ('Evening')", [])
            .expect("playlist");
        connection
            .execute(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (1, 1, 0)",
                [],
            )
            .expect("membership");

        connection
            .execute("DELETE FROM playlists WHERE id = 1", [])
            .expect("delete playlist");

        let remaining: i64 = connection
            .query_row("SELECT COUNT(*) FROM playlist_tracks", [], |row| row.get(0))
            .expect("count");

        assert_eq!(remaining, 0);
    }

    #[test]
    fn a_sound_database_passes_the_integrity_check() {
        let mut connection = Connection::open_in_memory().expect("in-memory database");
        apply(&mut connection).expect("migrations run");

        let report = integrity_check(&connection).expect("check runs");

        assert!(report.is_ok(), "a fresh database is sound: {report:?}");
        assert!(!report.summary().is_empty());
    }

    #[test]
    fn the_integrity_check_reports_a_file_that_cannot_be_used() {
        // A file that is not a database at all: the check has to come back as a
        // failure rather than as a panic or as a clean bill of health.
        let directory = scratch("corrupt");
        let path = directory.join("library.sqlite");
        std::fs::write(&path, b"this is not a SQLite file, whatever it claims").expect("fixture");

        let connection = Connection::open(&path).expect("the file opens");
        let report = integrity_check(&connection);
        assert!(report.is_err(), "a corrupt file must not report as sound");
    }

    #[test]
    fn a_copy_of_the_database_holds_what_the_original_held() {
        let directory = scratch("backup");
        let path = directory.join("library.sqlite");
        let copy = directory.join("library-copy.sqlite");

        {
            let mut connection = Connection::open(&path).expect("open");
            apply(&mut connection).expect("migrations run");
            connection
                .execute(
                    "INSERT INTO tracks (path, title) VALUES ('/music/a.mp3', 'Roygbiv')",
                    [],
                )
                .expect("track");
            backup_to(&connection, &copy).expect("copy");
        }

        let copied = Connection::open(&copy).expect("open the copy");
        let (path_column, title): (String, String) = copied
            .query_row("SELECT path, title FROM tracks", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("the copy holds the row");

        assert_eq!(path_column, "/music/a.mp3");
        assert_eq!(title, "Roygbiv");
        assert_eq!(
            current_version(&copied).expect("version"),
            SCHEMA_VERSION,
            "the copy is a database of the same schema"
        );
    }

    #[test]
    fn an_older_database_is_copied_beside_itself_before_it_is_migrated() {
        let directory = scratch("migrate-backup");
        let path = directory.join("library.sqlite");

        // Exactly what the build that shipped migrations 1 and 2 left behind.
        {
            let connection = Connection::open(&path).expect("open");
            connection.execute_batch(SQL_V1).expect("first schema");
            connection.execute_batch(SQL_V2).expect("second schema");
            connection
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS schema_version (
                         version    INTEGER PRIMARY KEY,
                         applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                     );",
                )
                .expect("version table");
            connection
                .execute("INSERT INTO schema_version (version) VALUES (2)", [])
                .expect("record the old version");
            connection
                .execute(
                    "INSERT INTO tracks (path, title) VALUES ('/music/a.mp3', 'Roygbiv')",
                    [],
                )
                .expect("track");
        }

        // The path the copy would take, worked out before the migration runs —
        // afterwards the name is taken and `backup_path` moves on to the next
        // one.
        let expected = backup_path(&path, 2);

        let mut connection = Connection::open(&path).expect("reopen");
        apply(&mut connection).expect("migrations run");

        assert_eq!(
            current_version(&connection).expect("version"),
            SCHEMA_VERSION
        );
        assert!(
            expected.is_file(),
            "no copy was taken at {expected:?}: {:?}",
            std::fs::read_dir(&directory)
                .expect("the directory")
                .flatten()
                .map(|entry| entry.file_name())
                .collect::<Vec<_>>()
        );

        // The copy is what the user was running, so going back to it is
        // possible even though the live database has moved on.
        let copied = Connection::open(&expected).expect("open the copy");
        assert_eq!(current_version(&copied).expect("version"), 2);
        let rows: i64 = copied
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .expect("count");
        assert_eq!(rows, 1, "the copy holds the music that was indexed");

        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn a_fresh_database_is_not_copied() {
        let directory = scratch("fresh-no-backup");
        let path = directory.join("library.sqlite");

        {
            let mut connection = Connection::open(&path).expect("open");
            apply(&mut connection).expect("migrations run");
        }

        let backups: Vec<String> = std::fs::read_dir(&directory)
            .expect("the directory")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".bak"))
            .collect();

        assert!(
            backups.is_empty(),
            "a database with nothing in it is not worth copying: {backups:?}"
        );

        std::fs::remove_dir_all(&directory).ok();
    }

    /// A scratch directory of this test's own.
    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-schema-{}-{name}", std::process::id()));
        std::fs::remove_dir_all(&directory).ok();
        std::fs::create_dir_all(&directory).expect("the scratch directory");

        directory
    }
}
