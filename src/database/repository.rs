//! Storing the library in SQLite.
//!
//! Everything in this module goes through [`LibraryRepository`], which owns the
//! one connection the application writes with. Two rules shape the code:
//!
//! - **Deleting a row never touches the disk.** `remove_track` forgets a file;
//!   it does not delete it. Every path that forgets a track says so in its name
//!   or its documentation.
//! - **Storing a track is idempotent.** A track is identified by its path, so
//!   re-importing a folder updates rows instead of adding duplicates, and the
//!   caller is told which of the two happened.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::database::schema::{self, IntegrityReport};
use crate::error::{AppError, Result};
use crate::library::track::{Album, Artist, PlaylistId, Track, TrackFields, TrackId};

/// The columns of a `tracks` row, in the order [`row_to_track`] expects.
const TRACK_COLUMNS: &str = "id, path, title, artist, album, album_artist, genre, track_number, \
     disc_number, duration_ms, artwork_path, size_bytes, modified_at, missing, tags_dirty, \
     play_count";

/// What storing a track did to the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertOutcome {
    /// The path was not indexed before.
    Inserted,
    /// The path was indexed and its stored details changed.
    Updated,
    /// The path was indexed and nothing had changed.
    Unchanged,
}

/// How many tracks an import inserted, changed and left alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpsertSummary {
    /// Tracks that were not in the database before.
    pub inserted: usize,
    /// Tracks whose stored details changed.
    pub updated: usize,
    /// Tracks that were already correct.
    pub unchanged: usize,
}

impl UpsertSummary {
    /// How many tracks were written or rewritten.
    #[must_use]
    pub fn changed(&self) -> usize {
        self.inserted + self.updated
    }
}

/// The library database.
pub struct LibraryRepository {
    connection: Connection,
}

impl LibraryRepository {
    /// Open (or create) the database at `path`, applying any pending migrations.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the file cannot be opened or the
    /// schema cannot be brought up to date.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| AppError::io(parent, error))?;
        }

        let mut connection = Connection::open(path).map_err(AppError::database)?;
        schema::apply(&mut connection)?;

        Ok(Self { connection })
    }

    /// Open a private database held entirely in memory.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the schema cannot be applied.
    pub fn open_in_memory() -> Result<Self> {
        let mut connection = Connection::open_in_memory().map_err(AppError::database)?;
        schema::apply(&mut connection)?;

        Ok(Self { connection })
    }

    /// The underlying connection, for the playlist module's own statements.
    #[must_use]
    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }

    /// The underlying connection, mutably, for statements that need a
    /// transaction.
    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    /// Store one track, keyed by its path.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the row cannot be read or written.
    pub fn upsert_track(&mut self, track: &Track) -> Result<UpsertOutcome> {
        let path = normalise(&track.path);
        merge(&self.connection, &path, track)
    }

    /// Store many tracks in one transaction.
    ///
    /// A scan of a large folder either lands completely or not at all, which
    /// keeps the database consistent with the folder it describes.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the transaction fails.
    pub fn upsert_tracks(&mut self, tracks: &[Track]) -> Result<UpsertSummary> {
        let transaction = self.connection.transaction().map_err(AppError::database)?;
        let mut summary = UpsertSummary::default();

        for track in tracks {
            let path = normalise(&track.path);

            match merge(&transaction, &path, track)? {
                UpsertOutcome::Inserted => summary.inserted += 1,
                UpsertOutcome::Updated => summary.updated += 1,
                UpsertOutcome::Unchanged => summary.unchanged += 1,
            }
        }

        transaction.commit().map_err(AppError::database)?;

        tracing::debug!(
            "stored {} tracks: {} inserted, {} updated, {} unchanged",
            tracks.len(),
            summary.inserted,
            summary.updated,
            summary.unchanged
        );

        Ok(summary)
    }

    /// Look a track up by its path.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn track_by_path(&self, path: impl AsRef<Path>) -> Result<Option<Track>> {
        let path = normalise(path.as_ref());

        self.connection
            .query_row(
                &format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE path = ?1"),
                params![path.to_string_lossy()],
                row_to_track,
            )
            .optional()
            .map_err(AppError::database)
    }

    /// Look a track up by its identifier.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn track_by_id(&self, id: TrackId) -> Result<Option<Track>> {
        self.connection
            .query_row(
                &format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE id = ?1"),
                params![id.get()],
                row_to_track,
            )
            .optional()
            .map_err(AppError::database)
    }

    /// Every indexed track, ordered by artist, album and position.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn tracks(&self) -> Result<Vec<Track>> {
        self.query_tracks(
            &format!(
                "SELECT {TRACK_COLUMNS} FROM tracks ORDER BY \
                 COALESCE(album_artist, artist, '') COLLATE NOCASE, \
                 COALESCE(album, '') COLLATE NOCASE, \
                 COALESCE(disc_number, 1), COALESCE(track_number, 0), \
                 title COLLATE NOCASE, path"
            ),
            &[],
        )
    }

    /// The tracks on one album.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn tracks_for_album(&self, album: &str) -> Result<Vec<Track>> {
        self.query_tracks(
            &format!(
                "SELECT {TRACK_COLUMNS} FROM tracks WHERE album = ?1 COLLATE NOCASE \
                 ORDER BY COALESCE(disc_number, 1), COALESCE(track_number, 0), title COLLATE NOCASE"
            ),
            params![album],
        )
    }

    /// The tracks credited to one artist.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn tracks_for_artist(&self, artist: &str) -> Result<Vec<Track>> {
        self.query_tracks(
            &format!(
                "SELECT {TRACK_COLUMNS} FROM tracks WHERE artist = ?1 COLLATE NOCASE \
                 ORDER BY COALESCE(album, '') COLLATE NOCASE, \
                 COALESCE(disc_number, 1), COALESCE(track_number, 0)"
            ),
            params![artist],
        )
    }

    /// The tracks whose files have vanished.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn missing_tracks(&self) -> Result<Vec<Track>> {
        self.query_tracks(
            &format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE missing = 1 ORDER BY path"),
            params![],
        )
    }

    /// How many tracks are indexed, including missing ones.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn track_count(&self) -> Result<usize> {
        let count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .map_err(AppError::database)?;

        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    /// Record that a track has just been started, and return its new count.
    ///
    /// The count lives only in the library: no audio tag carries it, so a
    /// rescan cannot undo it, and the increment is a single `UPDATE` rather
    /// than a read-modify-write so two starts can never lose one another's
    /// work.
    ///
    /// Returns the count after the increment, or `None` when no row has that
    /// identifier — a track played from the command line has no row to count
    /// against, and starting it is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the update fails.
    pub fn record_play(&self, id: TrackId) -> Result<Option<u64>> {
        let changed = self
            .connection
            .execute(
                "UPDATE tracks SET play_count = play_count + 1 WHERE id = ?1",
                params![id.get()],
            )
            .map_err(AppError::database)?;

        if changed == 0 {
            return Ok(None);
        }

        self.connection
            .query_row(
                "SELECT play_count FROM tracks WHERE id = ?1",
                params![id.get()],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| Some(count.try_into().unwrap_or_default()))
            .map_err(AppError::database)
    }

    /// Ask SQLite whether the library database is internally consistent.
    ///
    /// This is the recovery half of the repository: it is what the user runs
    /// from Settings → Diagnostics when the library misbehaves, and it is
    /// deliberately not run on every start, because it reads the whole file.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the check cannot run at all.
    pub fn check_integrity(&self) -> Result<IntegrityReport> {
        schema::integrity_check(&self.connection)
    }

    /// The schema version the open database is actually at.
    ///
    /// Read from the file rather than taken from [`SCHEMA_VERSION`], because
    /// the point of the diagnostics is to say what this database is, not what
    /// this build expects.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the version row cannot be read.
    pub fn schema_version(&self) -> Result<i64> {
        schema::current_version(&self.connection)
    }

    /// Every album, with its track count and total playing time.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn albums(&self) -> Result<Vec<Album>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT album, album_artist, COUNT(*), SUM(duration_ms) FROM tracks \
                 WHERE album IS NOT NULL \
                 GROUP BY album COLLATE NOCASE, album_artist COLLATE NOCASE \
                 ORDER BY album COLLATE NOCASE",
            )
            .map_err(AppError::database)?;

        let rows = statement
            .query_map([], |row| {
                let total: Option<i64> = row.get(3)?;
                Ok(Album {
                    name: row.get(0)?,
                    album_artist: row.get(1)?,
                    track_count: usize::try_from(row.get::<_, i64>(2)?).unwrap_or(usize::MAX),
                    duration: total
                        .and_then(|millis| u64::try_from(millis).ok())
                        .map(Duration::from_millis),
                })
            })
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }

    /// Every artist, with how many albums and tracks they contribute.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn artists(&self) -> Result<Vec<Artist>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT artist, COUNT(DISTINCT album), COUNT(*) FROM tracks \
                 WHERE artist IS NOT NULL \
                 GROUP BY artist COLLATE NOCASE \
                 ORDER BY artist COLLATE NOCASE",
            )
            .map_err(AppError::database)?;

        let rows = statement
            .query_map([], |row| {
                Ok(Artist {
                    name: row.get(0)?,
                    album_count: usize::try_from(row.get::<_, i64>(1)?).unwrap_or(usize::MAX),
                    track_count: usize::try_from(row.get::<_, i64>(2)?).unwrap_or(usize::MAX),
                })
            })
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }

    /// Mark a track as missing or present.
    ///
    /// Returns whether a row was found and changed.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the update fails.
    pub fn mark_missing(&self, path: impl AsRef<Path>, missing: bool) -> Result<bool> {
        let path = normalise(path.as_ref());
        set_missing(&self.connection, &path, missing)
    }

    /// Re-check every indexed file and flag the ones that are gone.
    ///
    /// This reads the filesystem, not the database, so it can be slow on a large
    /// library; the interface is expected to call it after a scan or on demand
    /// rather than per frame. Rows are never removed here — a file that comes
    /// back is picked up by the next scan.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the update fails.
    pub fn mark_disappeared(&mut self) -> Result<usize> {
        let paths: Vec<(i64, String, bool)> = {
            let mut statement = self
                .connection
                .prepare("SELECT id, path, missing FROM tracks")
                .map_err(AppError::database)?;

            let rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)? != 0,
                    ))
                })
                .map_err(AppError::database)?;

            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(AppError::database)?
        };

        let transaction = self.connection.transaction().map_err(AppError::database)?;
        let mut changed = 0;

        for (id, path, was_missing) in paths {
            let is_missing = !Path::new(&path).is_file();

            if is_missing != was_missing {
                transaction
                    .execute(
                        "UPDATE tracks SET missing = ?2, updated_at = datetime('now') WHERE id = ?1",
                        params![id, i64::from(is_missing)],
                    )
                    .map_err(AppError::database)?;

                if is_missing {
                    changed += 1;
                }
            }
        }

        transaction.commit().map_err(AppError::database)?;
        Ok(changed)
    }

    /// Forget a track, without touching the file on disk.
    ///
    /// The audio file is left exactly as it is; only the library row goes away.
    /// Returns whether a row was actually deleted.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the delete fails.
    pub fn remove_track(&self, id: TrackId) -> Result<bool> {
        let removed = self
            .connection
            .execute("DELETE FROM tracks WHERE id = ?1", params![id.get()])
            .map_err(AppError::database)?;

        if removed > 0 {
            tracing::info!("forgot track {id} from the library");
        }

        Ok(removed > 0)
    }

    /// Forget every track whose file has vanished.
    ///
    /// Again, only the rows are removed.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the delete fails.
    pub fn remove_missing(&self) -> Result<usize> {
        self.connection
            .execute("DELETE FROM tracks WHERE missing = 1", [])
            .map_err(AppError::database)
    }

    /// Apply the user's edits to one row.
    ///
    /// `written` says whether the same values were also written into the file's
    /// own tags: when they were, the row is allowed to stop advertising itself
    /// as edited, and when they were not, [`Track::tags_dirty`] records that the
    /// library and the file disagree. `artwork` is the cover the library should
    /// now show, which the caller has already copied into the artwork cache;
    /// passing `None` records that there is none. Nothing here touches the disk
    /// — this is the database half of an edit, and the caller decides whether
    /// the file half ever happens.
    ///
    /// Returns whether a row was found and changed.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the update fails.
    pub fn apply_edit(
        &self,
        id: TrackId,
        fields: &TrackFields,
        artwork: Option<&Path>,
        written: bool,
    ) -> Result<bool> {
        let changed = self
            .connection
            .execute(
                "UPDATE tracks SET
                     title = ?2, artist = ?3, album = ?4, album_artist = ?5, genre = ?6,
                     track_number = ?7, disc_number = ?8, artwork_path = ?9, tags_dirty = ?10,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    id.get(),
                    fields.title,
                    fields.artist,
                    fields.album,
                    fields.album_artist,
                    fields.genre,
                    fields.track_number,
                    fields.disc_number,
                    artwork.map(|path| path.to_string_lossy().into_owned()),
                    i64::from(!written),
                ],
            )
            .map_err(AppError::database)?;

        Ok(changed > 0)
    }

    /// How many playlists exist.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn playlist_count(&self) -> Result<usize> {
        let count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM playlists", [], |row| row.get(0))
            .map_err(AppError::database)?;

        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    /// The identifier of a playlist by name, if it exists.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn playlist_id(&self, name: &str) -> Result<Option<PlaylistId>> {
        self.connection
            .query_row(
                "SELECT id FROM playlists WHERE name = ?1 COLLATE NOCASE",
                params![name],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map(|id| id.map(PlaylistId))
            .map_err(AppError::database)
    }

    /// Run a track query and collect its rows.
    fn query_tracks(&self, sql: &str, parameters: &[&dyn rusqlite::ToSql]) -> Result<Vec<Track>> {
        let mut statement = self.connection.prepare(sql).map_err(AppError::database)?;
        let rows = statement
            .query_map(parameters, row_to_track)
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }
}

impl std::fmt::Debug for LibraryRepository {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LibraryRepository")
            .field("tracks", &self.track_count().ok())
            .field("playlists", &self.playlist_count().ok())
            .finish_non_exhaustive()
    }
}

/// Canonicalise a path so the same file is never indexed twice.
///
/// Falls back to the path as given when the file cannot be resolved — a file
/// that vanished should still be found by the name it was stored under.
fn normalise(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Store one scanned track, keyed by its path.
///
/// Both the single-track and the batch write paths come through here, so the
/// rule for what a scan does to a row the user has edited lives in exactly one
/// place.
///
/// The rule: **a scan never overwrites an edit while the file itself has not
/// changed**. A row marked [`Track::tags_dirty`] whose file still has the size
/// and modification time the last scan recorded keeps the tags the user typed
/// and only has its file-derived facts refreshed. Once the file *has* changed
/// — retagged in another program, re-encoded, replaced — the file is the truth
/// again and the row follows it, which is what makes a rescan able to pick up
/// tags set outside this application.
fn merge(connection: &Connection, path: &Path, track: &Track) -> Result<UpsertOutcome> {
    let existing: Option<Track> = connection
        .query_row(
            &format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE path = ?1"),
            params![path.to_string_lossy()],
            row_to_track,
        )
        .optional()
        .map_err(AppError::database)?;

    let Some(existing) = existing else {
        insert_track(connection, path, track)?;
        return Ok(UpsertOutcome::Inserted);
    };

    let mut incoming = track.clone();
    incoming.path = path.to_path_buf();

    if existing.tags_dirty && !file_changed(&existing, &incoming) {
        let mut merged = existing.clone();
        merged.duration = incoming.duration;
        merged.size_bytes = incoming.size_bytes;
        merged.modified_at = incoming.modified_at;
        merged.missing = false;

        if merged.same_content(&existing) && !existing.missing {
            return Ok(UpsertOutcome::Unchanged);
        }

        update_file_facts(connection, &merged)?;
        return Ok(UpsertOutcome::Updated);
    }

    if existing.same_content(&incoming) {
        // The file may have come back since it was marked missing.
        if existing.missing {
            set_missing(connection, &incoming.path, false)?;
            return Ok(UpsertOutcome::Updated);
        }
        return Ok(UpsertOutcome::Unchanged);
    }

    update_track(connection, &incoming)?;
    Ok(UpsertOutcome::Updated)
}

/// Whether the file itself has been written since the row was last scanned.
///
/// Size and modification time are what a scan can see cheaply, and a retag, a
/// re-encode or a replacement all move at least one of them.
fn file_changed(existing: &Track, incoming: &Track) -> bool {
    existing.size_bytes != incoming.size_bytes || existing.modified_at != incoming.modified_at
}

/// Build a track from a `tracks` row.
///
/// Shared with the playlist queries so both read a track the same way.
pub(crate) fn row_to_track(row: &Row<'_>) -> rusqlite::Result<Track> {
    let duration: Option<i64> = row.get(9)?;

    Ok(Track {
        id: Some(TrackId(row.get(0)?)),
        path: PathBuf::from(row.get::<_, String>(1)?),
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        album_artist: row.get(5)?,
        genre: row.get(6)?,
        track_number: row
            .get::<_, Option<i64>>(7)?
            .and_then(|value| u32::try_from(value).ok()),
        disc_number: row
            .get::<_, Option<i64>>(8)?
            .and_then(|value| u32::try_from(value).ok()),
        duration: duration
            .and_then(|millis| u64::try_from(millis).ok())
            .map(Duration::from_millis),
        artwork_path: row.get::<_, Option<String>>(10)?.map(PathBuf::from),
        size_bytes: row
            .get::<_, Option<i64>>(11)?
            .and_then(|value| u64::try_from(value).ok()),
        modified_at: row.get(12)?,
        missing: row.get::<_, i64>(13)? != 0,
        tags_dirty: row.get::<_, i64>(14)? != 0,
        play_count: row.get::<_, i64>(15)?.try_into().unwrap_or_default(),
    })
}

/// Insert a new row.
fn insert_track(connection: &Connection, path: &Path, track: &Track) -> Result<()> {
    connection
        .execute(
            "INSERT INTO tracks (
                 path, title, artist, album, album_artist, genre, track_number, disc_number,
                 duration_ms, artwork_path, size_bytes, modified_at, missing
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0)",
            params![
                path.to_string_lossy(),
                track.title,
                track.artist,
                track.album,
                track.album_artist,
                track.genre,
                track.track_number,
                track.disc_number,
                track
                    .duration
                    .map(|value| i64::try_from(value.as_millis()).unwrap_or(i64::MAX)),
                track
                    .artwork_path
                    .as_ref()
                    .map(|value| value.to_string_lossy().into_owned()),
                track.size_bytes.map(|value| value as i64),
                track.modified_at,
            ],
        )
        .map(|_| ())
        .map_err(AppError::database)
}

/// Rewrite the tag-derived fields of an existing row.
///
/// This is the scan's write path: the row is made to agree with the file, which
/// includes dropping [`Track::tags_dirty`] — once the file has been re-read,
/// the library and the file say the same thing again.
fn update_track(connection: &Connection, track: &Track) -> Result<()> {
    connection
        .execute(
            "UPDATE tracks SET
                 title = ?2, artist = ?3, album = ?4, album_artist = ?5, genre = ?6,
                 track_number = ?7, disc_number = ?8, duration_ms = ?9, artwork_path = ?10,
                 size_bytes = ?11, modified_at = ?12, missing = 0, tags_dirty = 0,
                 updated_at = datetime('now')
             WHERE path = ?1",
            params![
                track.path.to_string_lossy(),
                track.title,
                track.artist,
                track.album,
                track.album_artist,
                track.genre,
                track.track_number,
                track.disc_number,
                track
                    .duration
                    .map(|value| i64::try_from(value.as_millis()).unwrap_or(i64::MAX)),
                track
                    .artwork_path
                    .as_ref()
                    .map(|value| value.to_string_lossy().into_owned()),
                track.size_bytes.map(|value| value as i64),
                track.modified_at,
            ],
        )
        .map(|_| ())
        .map_err(AppError::database)
}

/// Refresh the facts a scan can see about a file, leaving the row's tags alone.
///
/// This is the second half of the merge rule: an edited row keeps its title,
/// artist and the rest, but still learns that its file grew, was rewritten, or
/// came back after having gone missing.
fn update_file_facts(connection: &Connection, track: &Track) -> Result<()> {
    connection
        .execute(
            "UPDATE tracks SET duration_ms = ?2, size_bytes = ?3, modified_at = ?4, missing = 0, \
                 updated_at = datetime('now')
             WHERE path = ?1",
            params![
                track.path.to_string_lossy(),
                track
                    .duration
                    .map(|value| i64::try_from(value.as_millis()).unwrap_or(i64::MAX)),
                track.size_bytes.map(|value| value as i64),
                track.modified_at,
            ],
        )
        .map(|_| ())
        .map_err(AppError::database)
}

/// Set the `missing` flag for one path, reporting whether a row changed.
fn set_missing(connection: &Connection, path: &Path, missing: bool) -> Result<bool> {
    let changed = connection
        .execute(
            "UPDATE tracks SET missing = ?2, updated_at = datetime('now') WHERE path = ?1",
            params![path.to_string_lossy(), i64::from(missing)],
        )
        .map_err(AppError::database)?;

    Ok(changed > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;

    fn repository() -> LibraryRepository {
        LibraryRepository::open_in_memory().expect("in-memory repository")
    }

    /// A track whose tags are all set, so updates are easy to observe.
    fn tagged(path: &str) -> Track {
        let mut track = Track::from_path(path);
        track.title = "Roygbiv".to_owned();
        track.artist = Some("Boards of Canada".to_owned());
        track.album = Some("Music Has the Right to Children".to_owned());
        track.album_artist = Some("Boards of Canada".to_owned());
        track.genre = Some("Electronic".to_owned());
        track.track_number = Some(3);
        track.disc_number = Some(1);
        track.duration = Some(Duration::from_secs(153));
        track.size_bytes = Some(1234);
        track.modified_at = Some(1_700_000_000);
        track
    }

    #[test]
    fn storing_a_new_track_reports_an_insert() {
        let mut repository = repository();

        let outcome = repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("upsert");

        assert_eq!(outcome, UpsertOutcome::Inserted);
        assert_eq!(repository.track_count().expect("count"), 1);
    }

    #[test]
    fn a_round_tripped_track_keeps_every_field() {
        let mut repository = repository();
        let track = tagged("/music/a.mp3");
        repository.upsert_track(&track).expect("upsert");

        let stored = repository
            .track_by_path("/music/a.mp3")
            .expect("lookup")
            .expect("the track is there");

        assert_eq!(stored.title, track.title);
        assert_eq!(stored.artist, track.artist);
        assert_eq!(stored.album, track.album);
        assert_eq!(stored.genre, track.genre);
        assert_eq!(stored.track_number, track.track_number);
        assert_eq!(stored.disc_number, track.disc_number);
        assert_eq!(stored.duration, track.duration);
        assert_eq!(stored.size_bytes, track.size_bytes);
        assert_eq!(stored.modified_at, track.modified_at);
        assert!(stored.id.is_some());
        assert!(!stored.missing);
    }

    #[test]
    fn storing_the_same_track_twice_changes_nothing() {
        let mut repository = repository();
        let track = tagged("/music/a.mp3");

        repository.upsert_track(&track).expect("first");
        let outcome = repository.upsert_track(&track).expect("second");

        assert_eq!(outcome, UpsertOutcome::Unchanged);
        assert_eq!(repository.track_count().expect("count"), 1);
    }

    #[test]
    fn changed_tags_report_an_update() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("first");

        let mut changed = tagged("/music/a.mp3");
        changed.title = "Olson".to_owned();
        let outcome = repository.upsert_track(&changed).expect("second");

        assert_eq!(outcome, UpsertOutcome::Updated);
        assert_eq!(
            repository
                .track_by_path("/music/a.mp3")
                .expect("lookup")
                .expect("present")
                .title,
            "Olson"
        );
        assert_eq!(repository.track_count().expect("count"), 1);
    }

    #[test]
    fn a_path_is_never_indexed_twice() {
        let mut repository = repository();

        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("duplicate");

        assert_eq!(repository.track_count().expect("count"), 1);
    }

    #[test]
    fn an_unknown_path_looks_up_as_nothing() {
        let repository = repository();

        assert!(
            repository
                .track_by_path("/music/absent.mp3")
                .expect("lookup")
                .is_none()
        );
    }

    #[test]
    fn missing_tracks_can_be_flagged_and_found() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");

        assert!(repository.mark_missing("/music/a.mp3", true).expect("mark"));
        assert_eq!(repository.missing_tracks().expect("missing").len(), 1);
        assert_eq!(repository.track_count().expect("count"), 1);

        assert!(
            repository
                .mark_missing("/music/a.mp3", false)
                .expect("unmark")
        );
        assert!(repository.missing_tracks().expect("missing").is_empty());
    }

    #[test]
    fn a_returned_file_is_no_longer_missing_after_a_rescan() {
        let mut repository = repository();
        let track = tagged("/music/a.mp3");
        repository.upsert_track(&track).expect("insert");
        repository.mark_missing("/music/a.mp3", true).expect("mark");

        let outcome = repository.upsert_track(&track).expect("rescan");

        assert_eq!(outcome, UpsertOutcome::Updated);
        assert!(repository.missing_tracks().expect("missing").is_empty());
    }

    #[test]
    fn forgetting_a_track_leaves_the_file_alone() {
        let path =
            std::env::temp_dir().join(format!("musiccraft-forget-{}.mp3", std::process::id()));
        std::fs::write(&path, b"audio bytes").expect("scratch file");

        let mut repository = repository();
        repository
            .upsert_track(&tagged(&path.to_string_lossy()))
            .expect("insert");
        let stored = repository
            .track_by_path(&path)
            .expect("lookup")
            .expect("present");

        assert!(
            repository
                .remove_track(stored.id.expect("identifier"))
                .expect("remove")
        );
        assert_eq!(repository.track_count().expect("count"), 0);
        assert!(
            path.is_file(),
            "the audio file must survive being forgotten"
        );
        assert_eq!(
            std::fs::read(&path).expect("still readable"),
            b"audio bytes"
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn forgetting_an_unknown_track_reports_nothing_removed() {
        let repository = repository();

        assert!(!repository.remove_track(TrackId(404)).expect("remove"));
    }

    #[test]
    fn a_disappeared_file_is_flagged_by_a_rescan_of_the_disk() {
        let path =
            std::env::temp_dir().join(format!("musiccraft-gone-{}.flac", std::process::id()));
        std::fs::write(&path, b"audio bytes").expect("scratch file");

        let mut repository = repository();
        repository
            .upsert_track(&tagged(&path.to_string_lossy()))
            .expect("insert");

        assert_eq!(repository.mark_disappeared().expect("rescan"), 0);

        std::fs::remove_file(&path).expect("delete the file");
        assert_eq!(repository.mark_disappeared().expect("rescan"), 1);
        assert_eq!(repository.track_count().expect("count"), 1);

        // Forgetting it is a separate, deliberate step.
        assert_eq!(repository.remove_missing().expect("forget"), 1);
        assert_eq!(repository.track_count().expect("count"), 0);
    }

    #[test]
    fn an_edit_is_stored_and_marks_the_row_as_ahead_of_its_file() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");

        let mut fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );
        fields.title = "Olson".to_owned();
        fields.track_number = Some(2);

        assert!(
            repository
                .apply_edit(id, &fields, None, false)
                .expect("edit")
        );

        let stored = repository
            .track_by_id(id)
            .expect("lookup")
            .expect("present");
        assert_eq!(stored.title, "Olson");
        assert_eq!(stored.track_number, Some(2));
        assert!(
            stored.tags_dirty,
            "the library says the file has not been told yet"
        );
    }

    #[test]
    fn an_edit_can_point_the_row_at_a_chosen_cover() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");
        let fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );
        let cover = PathBuf::from("/cache/chosen.png");

        repository
            .apply_edit(id, &fields, Some(&cover), false)
            .expect("edit");

        assert_eq!(
            repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present")
                .artwork_path,
            Some(cover)
        );
    }

    #[test]
    fn writing_the_edit_to_the_file_clears_the_flag() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");
        let fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );

        repository
            .apply_edit(id, &fields, None, false)
            .expect("edit");
        repository
            .apply_edit(id, &fields, None, true)
            .expect("write");

        assert!(
            !repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present")
                .tags_dirty
        );
    }

    #[test]
    fn a_rescan_of_a_changed_file_takes_the_row_back_to_the_files_own_tags() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");

        let mut fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );
        fields.title = "Olly".to_owned();
        repository
            .apply_edit(id, &fields, None, false)
            .expect("edit");

        // The file changed since the scan, so the scan wins and the edit goes.
        let mut rescanned = tagged("/music/a.mp3");
        rescanned.size_bytes = Some(4321);
        rescanned.modified_at = Some(1_800_000_000);
        let outcome = repository.upsert_track(&rescanned).expect("rescan");

        assert_eq!(outcome, UpsertOutcome::Updated);
        assert_eq!(
            repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present")
                .title,
            "Roygbiv",
            "the file is the source of truth once it changes"
        );
    }

    #[test]
    fn a_rescan_of_an_unchanged_file_leaves_a_hand_edit_alone() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");

        let mut fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );
        fields.title = "Olly".to_owned();
        repository
            .apply_edit(id, &fields, None, false)
            .expect("edit");

        let outcome = repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("rescan");

        assert_eq!(outcome, UpsertOutcome::Unchanged);
        let stored = repository
            .track_by_id(id)
            .expect("lookup")
            .expect("present");
        assert_eq!(stored.title, "Olly", "an edit survives a plain rescan");
        assert!(stored.tags_dirty);
    }

    #[test]
    fn an_edit_that_found_no_cover_records_that_there_is_none() {
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("insert");
        let id = stored_id(&repository, "/music/a.mp3");
        let fields = TrackFields::from_track(
            &repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present"),
        );
        let cover = PathBuf::from("/cache/chosen.png");

        repository
            .apply_edit(id, &fields, Some(&cover), false)
            .expect("with cover");
        repository
            .apply_edit(id, &fields, None, false)
            .expect("without cover");

        assert_eq!(
            repository
                .track_by_id(id)
                .expect("lookup")
                .expect("present")
                .artwork_path,
            None,
            "the row shows the cover the editor showed"
        );
    }

    #[test]
    fn editing_an_unknown_row_reports_nothing_changed() {
        let repository = repository();

        assert!(
            !repository
                .apply_edit(TrackId(404), &TrackFields::default(), None, false)
                .expect("edit")
        );
    }

    /// The identifier of a stored row, for the edit tests.
    fn stored_id(repository: &LibraryRepository, path: &str) -> TrackId {
        repository
            .track_by_path(path)
            .expect("lookup")
            .expect("present")
            .id
            .expect("a stored row has an identifier")
    }

    #[test]
    fn albums_and_artists_are_derived_from_the_tracks() {
        let mut repository = repository();

        let mut first = tagged("/music/a.mp3");
        first.title = "Roygbiv".to_owned();
        let mut second = tagged("/music/b.mp3");
        second.title = "Olson".to_owned();

        repository.upsert_tracks(&[first, second]).expect("insert");

        let albums = repository.albums().expect("albums");
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Music Has the Right to Children");
        assert_eq!(albums[0].album_artist, Some("Boards of Canada".to_owned()));
        assert_eq!(albums[0].track_count, 2);
        assert_eq!(albums[0].duration, Some(Duration::from_secs(306)));

        let artists = repository.artists().expect("artists");
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].name, "Boards of Canada");
        assert_eq!(artists[0].track_count, 2);
        assert_eq!(artists[0].album_count, 1);
    }

    #[test]
    fn a_track_with_no_tags_still_lists() {
        let mut repository = repository();
        repository
            .upsert_track(&Track::from_path("/music/untagged.mp3"))
            .expect("insert");

        assert!(repository.albums().expect("albums").is_empty());
        assert!(repository.artists().expect("artists").is_empty());
        assert_eq!(repository.tracks().expect("tracks").len(), 1);
    }

    #[test]
    fn a_batch_import_summarises_what_it_did() {
        let mut repository = repository();

        let first = repository
            .upsert_tracks(&[tagged("/music/a.mp3"), tagged("/music/b.mp3")])
            .expect("first batch");
        assert_eq!(first.inserted, 2);
        assert_eq!(first.changed(), 2);

        let mut changed = tagged("/music/a.mp3");
        changed.title = "Olson".to_owned();
        let second = repository
            .upsert_tracks(&[changed, tagged("/music/b.mp3")])
            .expect("second batch");

        assert_eq!(second.inserted, 0);
        assert_eq!(second.updated, 1);
        assert_eq!(second.unchanged, 1);
    }

    #[test]
    fn tracks_for_an_album_come_back_in_order() {
        let mut repository = repository();

        let mut third = tagged("/music/c.mp3");
        third.title = "Third".to_owned();
        third.track_number = Some(3);
        let mut first = tagged("/music/a.mp3");
        first.title = "First".to_owned();
        first.track_number = Some(1);

        repository.upsert_tracks(&[third, first]).expect("insert");

        let titles: Vec<String> = repository
            .tracks_for_album("Music Has the Right to Children")
            .expect("album tracks")
            .into_iter()
            .map(|track| track.title)
            .collect();

        assert_eq!(titles, vec!["First".to_owned(), "Third".to_owned()]);
    }

    #[test]
    fn the_schema_version_is_recorded_when_the_repository_opens() {
        let repository = repository();

        assert_eq!(
            schema::current_version(repository.connection()).expect("version"),
            schema::SCHEMA_VERSION
        );
    }

    #[test]
    fn a_track_starts_with_no_plays_and_counts_each_one() {
        let repository = repository();
        let id = store(&repository, "/music/a.mp3");

        assert_eq!(
            repository
                .track_by_id(id)
                .expect("lookup")
                .expect("row")
                .play_count,
            0,
            "a track that has never been played reads zero"
        );

        assert_eq!(repository.record_play(id).expect("first play"), Some(1));
        assert_eq!(repository.record_play(id).expect("second play"), Some(2));
        assert_eq!(
            repository
                .track_by_id(id)
                .expect("lookup")
                .expect("row")
                .play_count,
            2
        );
    }

    #[test]
    fn a_play_count_survives_a_rescan_of_the_file() {
        // The count is the library's own, not the file's, so re-reading the
        // tags must not reset it — that is the whole reason it is a column and
        // not something written into a tag.
        let mut repository = repository();
        repository
            .upsert_track(&tagged("/music/a.mp3"))
            .expect("upsert");
        let id = stored_id(&repository, "/music/a.mp3");
        repository.record_play(id).expect("play");
        repository.record_play(id).expect("play");

        let mut changed = tagged("/music/a.mp3");
        changed.title = "Olson".to_owned();
        changed.size_bytes = Some(9999);
        changed.modified_at = Some(1_700_000_999);
        repository.upsert_track(&changed).expect("rescan");

        let stored = repository.track_by_id(id).expect("lookup").expect("row");
        assert_eq!(stored.title, "Olson", "the file won, as it should");
        assert_eq!(stored.play_count, 2, "but the count is not the file's");
    }

    #[test]
    fn counting_a_play_for_a_row_that_does_not_exist_reports_nothing() {
        let repository = repository();

        assert_eq!(repository.record_play(TrackId(404)).expect("play"), None);
    }

    /// Store a bare row at `path` and give back its identifier.
    fn store(repository: &LibraryRepository, path: &str) -> TrackId {
        repository
            .connection()
            .execute(
                "INSERT INTO tracks (path, title) VALUES (?1, 'A track')",
                params![path],
            )
            .expect("insert");
        stored_id(repository, path)
    }

    #[test]
    fn a_playlist_name_is_unique() {
        let repository = repository();
        repository
            .connection()
            .execute("INSERT INTO playlists (name) VALUES ('Evening')", [])
            .expect("first playlist");

        let duplicate = repository
            .connection()
            .execute("INSERT INTO playlists (name) VALUES ('Evening')", []);

        assert!(duplicate.is_err());
        assert_eq!(repository.playlist_count().expect("count"), 1);
    }
}
