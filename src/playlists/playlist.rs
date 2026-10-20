//! The playlist domain types.
//!
//! A playlist is described twice: as a summary (name and totals) for the sidebar
//! and as the full ordered track list. Both are derived from the same rows, so
//! the sidebar never has to load every track of every playlist.

use std::time::Duration;

use rusqlite::Row;

use crate::library::{PlaylistId, Track};

/// A playlist's name and totals, without its tracks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistSummary {
    /// Row identifier.
    pub id: PlaylistId,
    /// User-chosen name, unique case-insensitively.
    pub name: String,
    /// How many entries the playlist holds.
    pub track_count: usize,
    /// Total playing time, or `None` when no entry reports a duration.
    pub duration: Option<Duration>,
    /// Whether the playlist is filled by rules rather than by hand.
    ///
    /// Views read this to decide what to offer: a smart playlist has no
    /// membership to reorder, remove from, or add to, so those controls give
    /// way to editing the rules that fill it.
    pub smart: bool,
}

impl PlaylistSummary {
    /// Build a summary from a `(id, name, count, total_ms, is_smart)` row.
    ///
    /// # Errors
    ///
    /// Returns an error when a column is missing or the wrong type.
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let total: Option<i64> = row.get(3)?;

        Ok(Self {
            id: PlaylistId(row.get(0)?),
            name: row.get(1)?,
            track_count: usize::try_from(row.get::<_, i64>(2)?).unwrap_or(usize::MAX),
            duration: total
                .and_then(|millis| u64::try_from(millis).ok())
                .map(Duration::from_millis),
            smart: row.get::<_, i64>(4)? != 0,
        })
    }

    /// Whether the playlist holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.track_count == 0
    }

    /// The name, ready for the sidebar.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.trim()
    }

    /// What the playlist's totals say when there is room to show them.
    ///
    /// `1 track — 1:01`, or `No tracks yet` when there is nothing in it. Both
    /// the entry in the sidebar and the heading of the open playlist say it, so
    /// it is phrased once here rather than in either view.
    #[must_use]
    pub fn totals(&self) -> String {
        match self.duration {
            Some(duration) => format!(
                "{} — {}",
                self.count_of_tracks(),
                crate::app::format_duration(duration)
            ),
            None if self.is_empty() => "No tracks yet".to_owned(),
            None => self.count_of_tracks(),
        }
    }

    /// `1 track` / `3 tracks`, the phrasing the rest of the interface uses.
    #[must_use]
    fn count_of_tracks(&self) -> String {
        format!(
            "{} {}",
            self.track_count,
            if self.track_count == 1 {
                "track"
            } else {
                "tracks"
            }
        )
    }
}

/// A playlist together with its tracks, in play order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    /// The playlist's name and totals.
    pub summary: PlaylistSummary,
    /// The tracks, in the order they will be played.
    pub tracks: Vec<Track>,
}

impl Playlist {
    /// The playlist's identifier.
    #[must_use]
    pub fn id(&self) -> PlaylistId {
        self.summary.id
    }

    /// The playlist's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.summary.name
    }

    /// How many tracks the playlist holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Whether the playlist holds no tracks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// The tracks, in play order.
    #[must_use]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Total playing time, or `None` when no track reports one.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        let known: Vec<Duration> = self
            .tracks
            .iter()
            .filter_map(|track| track.duration)
            .collect();

        if known.is_empty() {
            return None;
        }

        Some(known.iter().sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::TrackId;

    fn summary(track_count: usize) -> PlaylistSummary {
        PlaylistSummary {
            id: PlaylistId(1),
            name: "Evening".to_owned(),
            track_count,
            duration: None,
            smart: false,
        }
    }

    #[test]
    fn an_empty_summary_reports_itself_empty() {
        assert!(summary(0).is_empty());
        assert!(!summary(2).is_empty());
    }

    #[test]
    fn a_playlist_reports_its_own_name_and_length() {
        let playlist = Playlist {
            summary: summary(2),
            tracks: vec![
                Track::from_path("/music/a.mp3"),
                Track::from_path("/music/b.mp3"),
            ],
        };

        assert_eq!(playlist.id(), PlaylistId(1));
        assert_eq!(playlist.name(), "Evening");
        assert_eq!(playlist.len(), 2);
        assert!(!playlist.is_empty());
        assert_eq!(playlist.tracks().len(), 2);
    }

    #[test]
    fn a_playlist_without_durations_reports_no_total() {
        let playlist = Playlist {
            summary: summary(1),
            tracks: vec![Track::from_path("/music/a.mp3")],
        };

        assert_eq!(playlist.duration(), None);
    }

    #[test]
    fn a_playlist_sums_the_durations_it_knows() {
        let mut first = Track::from_path("/music/a.mp3");
        first.duration = Some(Duration::from_secs(30));
        let second = Track::from_path("/music/b.mp3");

        let playlist = Playlist {
            summary: summary(2),
            tracks: vec![first, second],
        };

        assert_eq!(playlist.duration(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn a_track_identifier_round_trips_through_a_summary() {
        let track = Track {
            id: Some(TrackId(7)),
            ..Track::from_path("/music/a.mp3")
        };

        assert_eq!(track.id, Some(TrackId(7)));
    }
}
