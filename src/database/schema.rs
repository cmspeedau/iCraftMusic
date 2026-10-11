//! The library's SQLite schema and its migrations.
//!
//! The schema is versioned so a database written by an older build can be
//! brought forward without being thrown away. Each migration runs once, inside
//! a transaction, and the version it carries is recorded in `schema_version`
//! before the next one starts — so an interrupted upgrade leaves the database at
//! the last version it fully applied.

use rusqlite::Connection;

use crate::error::{AppError, Result};

/// The schema version this build writes.
pub const SCHEMA_VERSION: i64 = 2;

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

/// Bring `connection` up to [`SCHEMA_VERSION`].
///
/// Running this on an up-to-date database is a no-op, so it is safe to call on
/// every start.
///
/// # Errors
///
/// Returns [`AppError::Database`] when a migration cannot be applied; the
/// migration's transaction is rolled back and the recorded version is left
/// untouched.
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
}
