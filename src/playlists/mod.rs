//! User-made playlists.
//!
//! A playlist is an ordered list of references to tracks that are already in the
//! library: the tracks stay where they are on disk, and deleting a playlist
//! never touches an audio file. Removing a track from the library takes it out
//! of every playlist, which the schema enforces with a foreign key rather than
//! leaving the interface to remember.
//!
//! The manager borrows the one database connection the application owns, so a
//! playlist edit and a scan can never be in flight against the same file at
//! once:
//!
//! ```no_run
//! # use musiccraft::database::LibraryRepository;
//! # use musiccraft::playlists::Playlists;
//! # fn main() -> musiccraft::error::Result<()> {
//! # let mut repository = LibraryRepository::open_in_memory()?;
//! let mut playlists = Playlists::new(&mut repository);
//! let evening = playlists.create("Evening")?;
//! playlists.rename(evening, "Late Evening")?;
//! # Ok(())
//! # }
//! ```

pub mod playlist;

pub use playlist::{Playlist, PlaylistSummary};

use rusqlite::{OptionalExtension, params};

use crate::database::LibraryRepository;
use crate::error::{AppError, Result};
use crate::library::{PlaylistId, Track, TrackId};

/// Playlist editing, borrowing the library database for the length of a call.
pub struct Playlists<'a> {
    repository: &'a mut LibraryRepository,
}

impl<'a> Playlists<'a> {
    /// Edit the playlists held in `repository`.
    pub fn new(repository: &'a mut LibraryRepository) -> Self {
        Self { repository }
    }

    /// Create an empty playlist and return its identifier.
    ///
    /// Names are compared case-insensitively, so "Evening" and "evening" are the
    /// same playlist.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the name is already taken or the row
    /// cannot be written. Use [`Playlists::find`] to check first.
    pub fn create(&mut self, name: &str) -> Result<PlaylistId> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::InvalidPlaylistName);
        }

        let connection = self.repository.connection_mut();
        connection
            .execute("INSERT INTO playlists (name) VALUES (?1)", params![name])
            .map_err(AppError::database)?;

        let id = PlaylistId(connection.last_insert_rowid());
        tracing::info!("created playlist {id} ({name})");

        Ok(id)
    }

    /// Create a playlist if its name is free, or return the existing one.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::InvalidPlaylistName`] for a blank name, and
    /// [`AppError::Database`] when the row cannot be written.
    pub fn create_or_find(&mut self, name: &str) -> Result<PlaylistId> {
        match self.find(name)? {
            Some(id) => Ok(id),
            None => self.create(name),
        }
    }

    /// The identifier of a playlist by name.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn find(&self, name: &str) -> Result<Option<PlaylistId>> {
        self.repository.playlist_id(name.trim())
    }

    /// Every playlist, each with how many tracks it holds and how long it runs.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn list(&self) -> Result<Vec<PlaylistSummary>> {
        let mut statement = self
            .repository
            .connection()
            .prepare(
                "SELECT p.id, p.name, COUNT(m.track_id), SUM(t.duration_ms) \
                 FROM playlists p \
                 LEFT JOIN playlist_tracks m ON m.playlist_id = p.id \
                 LEFT JOIN tracks t ON t.id = m.track_id \
                 GROUP BY p.id \
                 ORDER BY p.name COLLATE NOCASE",
            )
            .map_err(AppError::database)?;

        let rows = statement
            .query_map([], PlaylistSummary::from_row)
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }

    /// One playlist, with its tracks in order.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn playlist(&self, id: PlaylistId) -> Result<Option<Playlist>> {
        let Some(summary) = self.summary(id)? else {
            return Ok(None);
        };

        Ok(Some(Playlist {
            summary,
            tracks: self.tracks(id)?,
        }))
    }

    /// A playlist's name and totals.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn summary(&self, id: PlaylistId) -> Result<Option<PlaylistSummary>> {
        self.repository
            .connection()
            .query_row(
                "SELECT p.id, p.name, COUNT(m.track_id), SUM(t.duration_ms) \
                 FROM playlists p \
                 LEFT JOIN playlist_tracks m ON m.playlist_id = p.id \
                 LEFT JOIN tracks t ON t.id = m.track_id \
                 WHERE p.id = ?1 \
                 GROUP BY p.id",
                params![id.get()],
                PlaylistSummary::from_row,
            )
            .optional()
            .map_err(AppError::database)
    }

    /// A playlist's tracks, in play order.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn tracks(&self, id: PlaylistId) -> Result<Vec<Track>> {
        let mut statement = self
            .repository
            .connection()
            .prepare(
                "SELECT t.id, t.path, t.title, t.artist, t.album, t.album_artist, t.genre, \
                        t.track_number, t.disc_number, t.duration_ms, t.artwork_path, \
                        t.size_bytes, t.modified_at, t.missing, t.tags_dirty \
                 FROM playlist_tracks m JOIN tracks t ON t.id = m.track_id \
                 WHERE m.playlist_id = ?1 \
                 ORDER BY m.position",
            )
            .map_err(AppError::database)?;

        let rows = statement
            .query_map(params![id.get()], crate::database::repository::row_to_track)
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }

    /// Whether a playlist is known to the database.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn exists(&self, id: PlaylistId) -> Result<bool> {
        let found: Option<i64> = self
            .repository
            .connection()
            .query_row(
                "SELECT 1 FROM playlists WHERE id = ?1",
                params![id.get()],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::database)?;

        Ok(found.is_some())
    }

    /// Rename a playlist.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::InvalidPlaylistName`] for a blank name,
    /// [`AppError::PlaylistNotFound`] when the playlist is gone, and
    /// [`AppError::Database`] when the name is already taken or the row cannot
    /// be written.
    pub fn rename(&mut self, id: PlaylistId, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::InvalidPlaylistName);
        }

        let changed = self
            .repository
            .connection()
            .execute(
                "UPDATE playlists SET name = ?2, updated_at = datetime('now') WHERE id = ?1",
                params![id.get(), name],
            )
            .map_err(AppError::database)?;

        if changed == 0 {
            return Err(AppError::PlaylistNotFound(id));
        }

        Ok(())
    }

    /// Delete a playlist and its membership rows, leaving every audio file and
    /// library track alone.
    ///
    /// Returns whether a playlist was actually deleted.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the delete fails.
    pub fn delete(&mut self, id: PlaylistId) -> Result<bool> {
        // The cascade normally handles this, but running it explicitly keeps the
        // behaviour right on a connection opened without foreign keys on.
        let connection = self.repository.connection_mut();
        let transaction = connection.transaction().map_err(AppError::database)?;
        transaction
            .execute(
                "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
                params![id.get()],
            )
            .map_err(AppError::database)?;
        let removed = transaction
            .execute("DELETE FROM playlists WHERE id = ?1", params![id.get()])
            .map_err(AppError::database)?;
        transaction.commit().map_err(AppError::database)?;

        Ok(removed > 0)
    }

    /// Append a track to the end of a playlist.
    ///
    /// Adding the same track twice is allowed — a playlist may play a track more
    /// than once — and the same track is never duplicated as a *row* in the
    /// library.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::PlaylistNotFound`] when the playlist is gone,
    /// [`AppError::TrackHasNoRow`] when the track has not been stored yet, and
    /// [`AppError::Database`] when the rows cannot be written.
    pub fn add_track(&mut self, playlist: PlaylistId, track: TrackId) -> Result<usize> {
        if !self.exists(playlist)? {
            return Err(AppError::PlaylistNotFound(playlist));
        }

        self.require_track(track)?;

        let connection = self.repository.connection_mut();
        let position: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_tracks WHERE playlist_id = ?1",
                params![playlist.get()],
                |row| row.get(0),
            )
            .map_err(AppError::database)?;

        connection
            .execute(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
                params![playlist.get(), track.get(), position],
            )
            .map_err(AppError::database)?;

        touch(connection, playlist)?;

        Ok(usize::try_from(position).unwrap_or(usize::MAX))
    }

    /// Take the first occurrence of a track out of a playlist, compacting the
    /// positions behind it.
    ///
    /// Returns whether the track was in the playlist. The track itself stays in
    /// the library.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the rows cannot be written.
    pub fn remove_track(&mut self, playlist: PlaylistId, track: TrackId) -> Result<bool> {
        let connection = self.repository.connection_mut();
        let transaction = connection.transaction().map_err(AppError::database)?;

        let position: Option<i64> = transaction
            .query_row(
                "SELECT MIN(position) FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
                params![playlist.get(), track.get()],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::database)?
            .flatten();

        let Some(position) = position else {
            transaction.commit().map_err(AppError::database)?;
            return Ok(false);
        };

        transaction
            .execute(
                "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND position = ?2",
                params![playlist.get(), position],
            )
            .map_err(AppError::database)?;
        transaction
            .execute(
                "UPDATE playlist_tracks SET position = position - 1 \
                 WHERE playlist_id = ?1 AND position > ?2",
                params![playlist.get(), position],
            )
            .map_err(AppError::database)?;

        let changed = transaction
            .execute(
                "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
                params![playlist.get()],
            )
            .map_err(AppError::database)?;
        transaction.commit().map_err(AppError::database)?;

        Ok(changed > 0)
    }

    /// Move the track at `from` to `to`, shifting the tracks in between.
    ///
    /// Both indices are zero-based positions within the playlist. Indices past
    /// the end are clamped, so this can also be used to send a track to the
    /// back.
    ///
    /// Returns whether anything moved.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::PlaylistNotFound`] when the playlist is gone and
    /// [`AppError::Database`] when the rows cannot be written.
    pub fn move_track(&mut self, playlist: PlaylistId, from: usize, to: usize) -> Result<bool> {
        if !self.exists(playlist)? {
            return Err(AppError::PlaylistNotFound(playlist));
        }

        let connection = self.repository.connection_mut();
        let transaction = connection.transaction().map_err(AppError::database)?;

        let mut track_ids: Vec<i64> = {
            let mut statement = transaction
                .prepare(
                    "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position",
                )
                .map_err(AppError::database)?;

            let rows = statement
                .query_map(params![playlist.get()], |row| row.get::<_, i64>(0))
                .map_err(AppError::database)?;

            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(AppError::database)?
        };

        if from >= track_ids.len() || from == to.min(track_ids.len().saturating_sub(1)) {
            transaction.commit().map_err(AppError::database)?;
            return Ok(false);
        }

        let track = track_ids.remove(from);
        let target = to.min(track_ids.len());
        track_ids.insert(target, track);

        transaction
            .execute(
                "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
                params![playlist.get()],
            )
            .map_err(AppError::database)?;

        for (position, track_id) in track_ids.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
                    params![
                        playlist.get(),
                        track_id,
                        i64::try_from(position).unwrap_or(i64::MAX)
                    ],
                )
                .map_err(AppError::database)?;
        }

        let changed = transaction
            .execute(
                "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
                params![playlist.get()],
            )
            .map_err(AppError::database)?;
        transaction.commit().map_err(AppError::database)?;

        Ok(changed > 0)
    }

    /// Copy a playlist under a new name.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::PlaylistNotFound`] when the source is gone,
    /// [`AppError::InvalidPlaylistName`] for a blank name, and
    /// [`AppError::Database`] when the name is taken or the rows cannot be
    /// written.
    pub fn duplicate(&mut self, id: PlaylistId, name: &str) -> Result<PlaylistId> {
        let tracks = self.tracks(id)?;
        if !self.exists(id)? {
            return Err(AppError::PlaylistNotFound(id));
        }

        let created = self.create(name)?;

        let connection = self.repository.connection_mut();
        let transaction = connection.transaction().map_err(AppError::database)?;

        for (position, track) in tracks.iter().enumerate() {
            let Some(track_id) = track.id else {
                continue;
            };

            transaction
                .execute(
                    "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
                    params![
                        created.get(),
                        track_id.get(),
                        i64::try_from(position).unwrap_or(i64::MAX)
                    ],
                )
                .map_err(AppError::database)?;
        }

        transaction.commit().map_err(AppError::database)?;
        Ok(created)
    }

    /// Take every track out of a playlist, keeping the playlist itself.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::PlaylistNotFound`] when the playlist is gone and
    /// [`AppError::Database`] when the rows cannot be written.
    pub fn clear(&mut self, id: PlaylistId) -> Result<usize> {
        if !self.exists(id)? {
            return Err(AppError::PlaylistNotFound(id));
        }

        let connection = self.repository.connection_mut();
        let removed = connection
            .execute(
                "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
                params![id.get()],
            )
            .map_err(AppError::database)?;

        touch(connection, id)?;
        Ok(removed)
    }

    /// How many tracks a playlist holds.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn len(&self, id: PlaylistId) -> Result<usize> {
        let count: i64 = self
            .repository
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
                params![id.get()],
                |row| row.get(0),
            )
            .map_err(AppError::database)?;

        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    /// Whether a playlist holds no tracks at all.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn is_empty(&self, id: PlaylistId) -> Result<bool> {
        Ok(self.len(id)? == 0)
    }

    /// Every playlist that contains a track.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Database`] when the query fails.
    pub fn playlists_containing(&self, track: TrackId) -> Result<Vec<PlaylistSummary>> {
        let mut statement = self
            .repository
            .connection()
            .prepare(
                "SELECT p.id, p.name, COUNT(m.track_id), SUM(t.duration_ms) \
                 FROM playlists p \
                 LEFT JOIN playlist_tracks m ON m.playlist_id = p.id \
                 LEFT JOIN tracks t ON t.id = m.track_id \
                 WHERE p.id IN (SELECT playlist_id FROM playlist_tracks WHERE track_id = ?1) \
                 GROUP BY p.id \
                 ORDER BY p.name COLLATE NOCASE",
            )
            .map_err(AppError::database)?;

        let rows = statement
            .query_map(params![track.get()], PlaylistSummary::from_row)
            .map_err(AppError::database)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)
    }

    /// Fail with [`AppError::TrackHasNoRow`] if the track has no row yet.
    fn require_track(&self, track: TrackId) -> Result<()> {
        let found: Option<i64> = self
            .repository
            .connection()
            .query_row(
                "SELECT 1 FROM tracks WHERE id = ?1",
                params![track.get()],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::database)?;

        match found {
            Some(_) => Ok(()),
            None => Err(AppError::TrackHasNoRow(track)),
        }
    }
}

impl std::fmt::Debug for Playlists<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Playlists")
            .field("playlists", &self.list().map(|list| list.len()).ok())
            .finish_non_exhaustive()
    }
}

/// Note that a playlist changed.
fn touch(connection: &rusqlite::Connection, id: PlaylistId) -> Result<()> {
    connection
        .execute(
            "UPDATE playlists SET updated_at = datetime('now') WHERE id = ?1",
            params![id.get()],
        )
        .map(|_| ())
        .map_err(AppError::database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::library::Track;

    /// A repository holding `count` tracks, returned with their identifiers.
    fn library(count: usize) -> (LibraryRepository, Vec<TrackId>) {
        let mut repository = LibraryRepository::open_in_memory().expect("in-memory repository");
        let mut ids = Vec::new();

        for index in 0..count {
            let mut track = Track::from_path(format!("/music/{index}.mp3"));
            track.title = format!("Track {index}");
            track.duration = Some(Duration::from_secs(60));
            repository.upsert_track(&track).expect("insert");

            let stored = repository
                .track_by_path(&track.path)
                .expect("lookup")
                .expect("stored");
            ids.push(stored.id.expect("identifier"));
        }

        (repository, ids)
    }

    #[test]
    fn a_new_playlist_is_empty() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        let id = playlists.create("Evening").expect("create");

        assert!(playlists.exists(id).expect("exists"));
        assert!(playlists.is_empty(id).expect("empty"));
        assert_eq!(playlists.len(id).expect("len"), 0);
    }

    #[test]
    fn playlists_list_in_name_order() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        playlists.create("Sleep").expect("create");
        playlists.create("Awake").expect("create");

        let names: Vec<String> = playlists
            .list()
            .expect("list")
            .into_iter()
            .map(|summary| summary.name)
            .collect();

        assert_eq!(names, vec!["Awake".to_owned(), "Sleep".to_owned()]);
    }

    #[test]
    fn a_duplicate_name_is_rejected() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        playlists.create("Evening").expect("create");
        let duplicate = playlists.create("evening");

        assert!(duplicate.is_err());
    }

    #[test]
    fn a_blank_name_is_rejected() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        assert!(playlists.create("   ").is_err());
    }

    #[test]
    fn create_or_find_is_idempotent() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        let first = playlists.create_or_find("Evening").expect("first");
        let second = playlists.create_or_find("evening").expect("second");

        assert_eq!(first, second);
        assert_eq!(playlists.list().expect("list").len(), 1);
    }

    #[test]
    fn a_playlist_can_be_found_by_name() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);

        let id = playlists.create("Evening").expect("create");

        assert_eq!(playlists.find("Evening").expect("find"), Some(id));
        assert_eq!(playlists.find("Morning").expect("find"), None);
    }

    #[test]
    fn tracks_are_appended_in_order() {
        let (mut repository, ids) = library(3);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");

        assert_eq!(playlists.add_track(id, ids[2]).expect("add"), 0);
        assert_eq!(playlists.add_track(id, ids[0]).expect("add"), 1);
        assert_eq!(playlists.add_track(id, ids[1]).expect("add"), 2);

        let titles: Vec<String> = playlists
            .tracks(id)
            .expect("tracks")
            .into_iter()
            .map(|track| track.title)
            .collect();

        assert_eq!(
            titles,
            vec![
                "Track 2".to_owned(),
                "Track 0".to_owned(),
                "Track 1".to_owned()
            ]
        );
    }

    #[test]
    fn a_track_may_appear_twice() {
        let (mut repository, ids) = library(1);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");

        playlists.add_track(id, ids[0]).expect("first");
        playlists.add_track(id, ids[0]).expect("second");

        assert_eq!(playlists.len(id).expect("len"), 2);
    }

    #[test]
    fn the_summary_counts_and_times_a_playlist() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");
        playlists.add_track(id, ids[1]).expect("add");

        let summary = playlists.summary(id).expect("summary").expect("present");

        assert_eq!(summary.name, "Evening");
        assert_eq!(summary.track_count, 2);
        assert_eq!(summary.duration, Some(Duration::from_secs(120)));
    }

    #[test]
    fn adding_an_unstored_track_is_refused() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");

        let error = playlists.add_track(id, TrackId(99));

        assert!(matches!(error, Err(AppError::TrackHasNoRow(_))));
    }

    #[test]
    fn adding_to_an_unknown_playlist_is_refused() {
        let (mut repository, ids) = library(1);
        let mut playlists = Playlists::new(&mut repository);

        let error = playlists.add_track(PlaylistId(42), ids[0]);

        assert!(matches!(error, Err(AppError::PlaylistNotFound(_))));
    }

    #[test]
    fn removing_a_track_keeps_the_rest_in_order() {
        let (mut repository, ids) = library(3);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");

        for track in &ids {
            playlists.add_track(id, *track).expect("add");
        }

        assert!(playlists.remove_track(id, ids[1]).expect("remove"));

        let remaining: Vec<TrackId> = playlists
            .tracks(id)
            .expect("tracks")
            .into_iter()
            .filter_map(|track| track.id)
            .collect();

        assert_eq!(remaining, vec![ids[0], ids[2]]);
        assert_eq!(playlists.len(id).expect("len"), 2);
    }

    #[test]
    fn removing_a_track_that_is_not_there_reports_nothing() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");

        assert!(!playlists.remove_track(id, ids[1]).expect("remove"));
        assert!(playlists.remove_track(id, ids[0]).expect("remove"));
        assert!(!playlists.remove_track(id, ids[0]).expect("second remove"));
    }

    #[test]
    fn a_track_can_be_moved_towards_the_front() {
        let (mut repository, ids) = library(3);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        for track in &ids {
            playlists.add_track(id, *track).expect("add");
        }

        assert!(playlists.move_track(id, 2, 0).expect("move"));

        let order: Vec<TrackId> = playlists
            .tracks(id)
            .expect("tracks")
            .into_iter()
            .filter_map(|track| track.id)
            .collect();

        assert_eq!(order, vec![ids[2], ids[0], ids[1]]);
    }

    #[test]
    fn a_track_can_be_moved_to_the_back() {
        let (mut repository, ids) = library(3);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        for track in &ids {
            playlists.add_track(id, *track).expect("add");
        }

        assert!(playlists.move_track(id, 0, 99).expect("move"));

        let order: Vec<TrackId> = playlists
            .tracks(id)
            .expect("tracks")
            .into_iter()
            .filter_map(|track| track.id)
            .collect();

        assert_eq!(order, vec![ids[1], ids[2], ids[0]]);
    }

    #[test]
    fn moving_a_track_onto_itself_changes_nothing() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        for track in &ids {
            playlists.add_track(id, *track).expect("add");
        }

        assert!(!playlists.move_track(id, 1, 1).expect("move"));
        assert!(!playlists.move_track(id, 9, 0).expect("out of range"));
    }

    #[test]
    fn a_playlist_can_be_renamed() {
        let (mut repository, _) = library(0);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");

        playlists.rename(id, "Late Evening").expect("rename");

        assert_eq!(
            playlists
                .summary(id)
                .expect("summary")
                .expect("present")
                .name,
            "Late Evening"
        );
        assert!(playlists.rename(PlaylistId(404), "Nowhere").is_err());
    }

    #[test]
    fn a_playlist_can_be_duplicated() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");
        playlists.add_track(id, ids[1]).expect("add");

        let copy = playlists.duplicate(id, "Evening Copy").expect("duplicate");

        assert_ne!(copy, id);
        assert_eq!(playlists.len(copy).expect("len"), 2);

        // Editing the copy leaves the original alone.
        playlists.remove_track(copy, ids[0]).expect("remove");
        assert_eq!(playlists.len(id).expect("len"), 2);
    }

    #[test]
    fn clearing_a_playlist_keeps_the_playlist() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");

        assert_eq!(playlists.clear(id).expect("clear"), 1);
        assert!(playlists.exists(id).expect("exists"));
        assert!(playlists.is_empty(id).expect("empty"));
    }

    #[test]
    fn deleting_a_playlist_leaves_the_tracks_alone() {
        let (mut repository, ids) = library(1);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");

        assert!(playlists.delete(id).expect("delete"));

        assert!(!playlists.exists(id).expect("exists"));
        assert_eq!(repository.track_count().expect("tracks"), 1);
    }

    #[test]
    fn forgetting_a_track_takes_it_out_of_playlists() {
        let (mut repository, ids) = library(1);
        let mut playlists = Playlists::new(&mut repository);
        let id = playlists.create("Evening").expect("create");
        playlists.add_track(id, ids[0]).expect("add");

        // The borrow ends with the last use of `playlists`.
        repository.remove_track(ids[0]).expect("forget the track");

        let playlists = Playlists::new(&mut repository);
        assert!(playlists.is_empty(id).expect("empty"));
    }

    #[test]
    fn the_playlists_holding_a_track_can_be_asked_for() {
        let (mut repository, ids) = library(2);
        let mut playlists = Playlists::new(&mut repository);
        let evening = playlists.create("Evening").expect("create");
        let morning = playlists.create("Morning").expect("create");
        playlists.add_track(evening, ids[0]).expect("add");
        playlists.add_track(morning, ids[1]).expect("add");

        let names: Vec<String> = playlists
            .playlists_containing(ids[0])
            .expect("containing")
            .into_iter()
            .map(|summary| summary.name)
            .collect();

        assert_eq!(names, vec!["Evening".to_owned()]);
    }

    /// A playlist survives being closed and reopened, which is the one thing
    /// about playlists a user notices: they are still there tomorrow.
    ///
    /// The repository is a real file rather than an in-memory database, because
    /// an in-memory one is thrown away with its connection and would make this
    /// pass without proving anything.
    #[test]
    fn playlists_and_their_tracks_survive_a_restart() {
        let directory = crate::fixtures::scratch("playlist-restart");
        let database = directory.join("library.sqlite");

        let track_id = {
            let mut repository = LibraryRepository::open(&database).expect("open the library");
            let mut track = Track::from_path("/music/Roygbiv.mp3");
            track.title = "Roygbiv".to_owned();
            repository.upsert_track(&track).expect("store the track");
            let stored = repository
                .track_by_path("/music/Roygbiv.mp3")
                .expect("lookup")
                .expect("stored");
            let id = stored.id.expect("identifier");

            let mut playlists = Playlists::new(&mut repository);
            let evening = playlists.create("Evening").expect("create");
            playlists.add_track(evening, id).expect("add");

            id
        };

        // Everything is reopened the way a second run of the application does.
        let mut repository = LibraryRepository::open(&database).expect("reopen the library");
        let playlists = Playlists::new(&mut repository);

        let names: Vec<String> = playlists
            .list()
            .expect("list")
            .into_iter()
            .map(|summary| summary.name)
            .collect();
        assert_eq!(names, vec!["Evening".to_owned()]);

        let evening = playlists
            .find("Evening")
            .expect("find")
            .expect("the playlist is still there");
        let tracks = playlists.tracks(evening).expect("tracks");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, Some(track_id), "the same row is still in it");
        assert_eq!(tracks[0].title, "Roygbiv");

        std::fs::remove_dir_all(&directory).ok();
    }
}
