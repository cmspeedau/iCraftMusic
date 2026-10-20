//! The transport, the queue, and the ways tracks enter them.

use super::{MusicCraftApp, NowPlaying, TransportState, format_duration, queue_item};
use crate::audio::{Playback, PlaybackEvent, QueueItem, RepeatMode};
use crate::library::Track;
use std::path::{Path, PathBuf};
use std::time::Duration;

impl MusicCraftApp {
    /// The state of the transport, as the playback bar should draw it.
    #[must_use]
    pub fn transport(&self) -> TransportState {
        if !self.playback.is_available() {
            TransportState::Unavailable
        } else if self.playback.is_playing() {
            TransportState::Playing
        } else if self.playback.is_paused() {
            TransportState::Paused
        } else {
            TransportState::Stopped
        }
    }

    /// The track the playback bar should show, if the queue has one selected.
    ///
    /// A queue entry carries a title and a path but no picture, so the cover
    /// comes from the library row indexed at that path — taken from the rows the
    /// last `refresh` loaded rather than queried from the database on every
    /// frame. A track that is not indexed (one played from the command line)
    /// simply has no cover.
    #[must_use]
    pub fn now_playing(&self) -> Option<NowPlaying> {
        let track = self.playback.current()?;
        let position = self
            .playback
            .queue_position()
            .map(|(index, total)| format!("Track {index} of {total}"));
        let duration = self.playback.duration().map(format_duration);
        let artwork = self
            .browse
            .tracks()
            .iter()
            .find(|row| row.path.as_path() == track.path())
            .and_then(|row| row.artwork_path.clone());

        Some(NowPlaying {
            title: track.title().to_owned(),
            position,
            duration,
            artwork,
        })
    }

    /// How far into the current track playback has reached.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.playback.position()
    }

    /// The length of the current track, when the decoder reported one.
    #[must_use]
    pub fn track_duration(&self) -> Option<Duration> {
        self.playback.duration()
    }

    /// How far through the current track playback is, in `0.0..=1.0`.
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.playback.progress()
    }

    /// The audio side, for views that only need to read it.
    #[must_use]
    pub fn playback(&self) -> &Playback {
        &self.playback
    }

    /// The audio side, for views that drive the transport.
    pub fn playback_mut(&mut self) -> &mut Playback {
        &mut self.playback
    }

    /// Start or pause, whichever the current state calls for.
    pub fn toggle_playback(&mut self) {
        self.playback.toggle();
    }

    /// Skip to the next track.
    pub fn next_track(&mut self) {
        self.playback.next();
    }

    /// Go back to the previous track.
    pub fn previous_track(&mut self) {
        self.playback.previous();
    }

    /// Stop playback and unload the track.
    pub fn stop_playback(&mut self) {
        self.playback.stop();
    }

    /// Seek to a fraction of the current track.
    pub fn seek_to_fraction(&mut self, fraction: f32) {
        self.playback.seek_fraction(fraction);
    }

    /// Change how the queue behaves at the end of a track.
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.playback.set_repeat(mode);

        // Shuffle and repeat are ways of listening rather than properties of one
        // queue, so the choice is remembered and applied to the next album too.
        if self.config.repeat != mode {
            self.config.repeat = mode;
            self.config_dirty = true;
        }
    }

    /// Advance the repeat mode to the next one, remembering the choice.
    ///
    /// Routed through [`MusicCraftApp::set_repeat_mode`] rather than writing
    /// the setting here: a switch that cycled the engine and left the stored
    /// mode behind would come back in the old mode on the next launch.
    pub fn cycle_repeat_mode(&mut self) -> RepeatMode {
        let next = self.playback_mut().cycle_repeat();
        self.set_repeat_mode(next);

        next
    }

    /// Turn shuffle on or off, returning the new state.
    pub fn toggle_shuffle(&mut self) -> bool {
        let shuffle = self.playback.toggle_shuffle();

        if self.config.shuffle != shuffle {
            self.config.shuffle = shuffle;
            self.config_dirty = true;
        }

        shuffle
    }

    /// Move a queued track to another position, returning whether it moved.
    ///
    /// Nothing about the transport changes: this is the Queue page reordering
    /// the list, not the user skipping.
    pub fn move_in_queue(&mut self, from: usize, to: usize) -> bool {
        self.playback.move_in_queue(from, to)
    }

    /// Take a queued track out, returning whether it came out.
    ///
    /// The track being played is not removable; the Queue view greys the button
    /// out rather than offering an action the player would refuse.
    pub fn remove_from_queue(&mut self, index: usize) -> bool {
        self.playback.remove_from_queue(index).is_some()
    }

    /// Empty the queue, stopping playback so nothing plays from a spent list.
    pub fn clear_queue(&mut self) {
        self.playback.clear_queue();
    }

    /// Jump to a queued track and start it.
    pub fn play_queue_index(&mut self, index: usize) {
        self.playback.play_index(index);
    }

    /// Let the player notice that a track ended, and advance if it did.
    ///
    /// The engine cannot call back into the interface, so this runs once per
    /// frame from [`eframe::App::ui`]. It is also where a track that has just
    /// been started gets counted, because the player records the start and can
    /// reach no database itself.
    pub fn poll_playback(&mut self) -> Option<PlaybackEvent> {
        let event = self.playback.poll();
        let started = self.playback.take_started();

        if !started.is_empty() {
            self.record_plays(&started);
        }

        event
    }

    /// Count one play for each of `paths`, for the rows that exist.
    ///
    /// A path with no row — a file opened from the command line, or one whose
    /// track was removed from the library — is skipped rather than reported:
    /// playing an un-indexed file is a normal thing to do and has no count to
    /// belong to. Returns how many rows were counted.
    ///
    /// The library is re-read afterwards so the Plays column shows the new
    /// number in the same frame the track started, which is the whole point of
    /// showing it beside the row.
    pub fn record_plays(&mut self, paths: &[PathBuf]) -> usize {
        let Some(library) = self.library.as_ref() else {
            return 0;
        };

        let mut counted = 0;

        for path in paths {
            let Some(id) = self
                .browse
                .tracks()
                .iter()
                .find(|track| track.path.as_path() == path.as_path())
                .and_then(|track| track.id)
            else {
                continue;
            };

            match library.record_play(id) {
                Ok(Some(_)) => counted += 1,
                Ok(None) => {}
                Err(error) => {
                    // A play count is not worth interrupting playback over: the
                    // track is playing, and the count is the thing that failed.
                    tracing::warn!("could not count a play for {id}: {error}");
                }
            }
        }

        if counted > 0
            && let Err(error) = self.refresh()
        {
            self.library_error = Some(error.to_string());
        }

        counted
    }

    /// The failure the player wants the user to see, ready to display.
    #[must_use]
    pub fn playback_error(&self) -> Option<String> {
        self.playback.error().map(ToString::to_string)
    }

    /// Hide the playback failure message.
    pub fn dismiss_playback_error(&mut self) {
        self.playback.dismiss_error();
    }

    /// The text in the library's "play a file or folder" field.
    #[must_use]
    pub fn path_input(&self) -> &str {
        &self.path_input
    }

    /// The play field's text, for the view to edit.
    pub fn path_input_mut(&mut self) -> &mut String {
        &mut self.path_input
    }

    /// Play whatever the user typed into the play field, if anything.
    ///
    /// The field is cleared once something was added, so the next entry starts
    /// from an empty line.
    pub fn play_path_input(&mut self) -> usize {
        let text = self.path_input.trim().to_owned();
        if text.is_empty() {
            return 0;
        }

        let added = self.open_path(Path::new(&text));
        if added > 0 {
            self.path_input.clear();
        }

        added
    }

    /// Loads one file or folder into the queue, and plays it when idle.
    pub fn open_path(&mut self, path: &Path) -> usize {
        let added = self.playback.enqueue_path(path);
        self.start_if_idle(added);

        added
    }

    /// Loads several files or folders, and plays the first when idle.
    ///
    /// The command line uses this so `musiccraft album/` starts playing
    /// immediately, and folders are scanned one level deep.
    pub fn open_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;
        for path in paths {
            added += self.playback.enqueue_path(&path);
        }
        self.start_if_idle(added);

        added
    }

    /// Starts the selected track when something was just added and nothing is
    /// playing, so selecting a folder begins at its first track.
    fn start_if_idle(&mut self, added: usize) {
        if added > 0 && !self.playback.is_playing() {
            self.playback.play_current();
        }
    }

    /// Replace the queue with `tracks` and start playing the one at `start`.
    ///
    /// The queue is rebuilt from what the user is looking at, so playing the
    /// fourth row of a sorted, filtered table plays that row and what follows
    /// it, in the order shown on screen.
    pub fn play_tracks(&mut self, tracks: &[Track], start: usize) -> usize {
        let items: Vec<QueueItem> = tracks.iter().map(queue_item).collect();
        let count = items.len();
        self.playback.play_items(items, start);

        count
    }

    /// Play the selected tracks, starting at the first one.
    ///
    /// Returns how many tracks were queued, so the view can tell "nothing was
    /// selected" from "playing".
    pub fn play_selection(&mut self) -> usize {
        let selected: Vec<Track> = self.browse.selected_tracks().into_iter().cloned().collect();

        self.play_tracks(&selected, 0)
    }

    /// Add the selected tracks to the end of the queue, without playing them.
    pub fn queue_selection(&mut self) -> usize {
        let selected: Vec<Track> = self.browse.selected_tracks().into_iter().cloned().collect();

        self.queue_tracks(&selected)
    }

    /// Add tracks to the end of the queue, without playing them.
    ///
    /// Returns how many entries were added, which is zero for an empty slice.
    pub fn queue_tracks(&mut self, tracks: &[Track]) -> usize {
        let items: Vec<QueueItem> = tracks.iter().map(queue_item).collect();

        self.playback.queue_items(items)
    }

    /// Add every track of an album to the end of the queue.
    pub fn queue_album(&mut self, album: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_album(album)
            .into_iter()
            .cloned()
            .collect();

        self.queue_tracks(&tracks)
    }

    /// Add every track credited to an artist to the end of the queue.
    pub fn queue_artist(&mut self, artist: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_artist(artist)
            .into_iter()
            .cloned()
            .collect();

        self.queue_tracks(&tracks)
    }

    /// Play every track of an album, in album order.
    pub fn play_album(&mut self, album: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_album(album)
            .into_iter()
            .cloned()
            .collect();

        self.play_tracks(&tracks, 0)
    }

    /// Play every track credited to an artist.
    pub fn play_artist(&mut self, artist: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_artist(artist)
            .into_iter()
            .cloned()
            .collect();

        self.play_tracks(&tracks, 0)
    }
}
