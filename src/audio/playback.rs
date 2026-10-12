//! The playback facade the rest of the application talks to.
//!
//! [`Playback`] joins the two halves of the audio module: the [`AudioEngine`],
//! which owns the output device, and the [`PlayQueue`], which owns the track
//! list and the transport rules. Everything the UI needs — start, pause, skip,
//! seek, volume, repeat, shuffle — is one method here, and each of them records
//! a failure for the UI to display instead of throwing it at the caller.
//!
//! Keeping the facade infallible matters for the interface: a frame cannot
//! return an error, and a player that dies because one file would not decode is
//! worse than one that shows a message and moves on.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{AudioEngine, AudioError, PlayQueue, QueueItem, RepeatMode, is_supported};

/// Something the queue did that the interface should know about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackEvent {
    /// The loaded track reached its end and the queue advanced.
    TrackFinished,
}

/// The device, the queue and the current transport state, as one object.
pub struct Playback {
    engine: AudioEngine,
    queue: PlayQueue,
    /// The last failure, kept until the interface shows or dismisses it.
    error: Option<AudioError>,
}

impl Playback {
    /// Opens the default output device and prepares an empty queue.
    ///
    /// A device that cannot be opened is not fatal: the player keeps working
    /// with no sound, records why, and reports it through
    /// [`Playback::error`]. Browsing the queue and changing settings still
    /// work, which is what a user on a headless machine or without a sound
    /// card needs.
    #[must_use]
    pub fn start(volume: f32) -> Self {
        match AudioEngine::open() {
            Ok(engine) => Self::with_engine(engine, volume),
            Err(error) => {
                let mut playback = Self::with_engine(AudioEngine::without_device(), volume);
                playback.error = Some(error);
                playback
            }
        }
    }

    /// Builds a player around an engine that is already open.
    #[must_use]
    pub fn with_engine(engine: AudioEngine, volume: f32) -> Self {
        let mut playback = Self {
            engine,
            queue: PlayQueue::new(),
            error: None,
        };
        playback.engine.set_volume(volume);
        playback
    }

    /// The track list and its transport rules.
    #[must_use]
    pub fn queue(&self) -> &PlayQueue {
        &self.queue
    }

    /// Whether an output device is open and sound is possible.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.engine.is_available()
    }

    /// A description of the open device, e.g. `48.0 kHz · 2 ch`.
    #[must_use]
    pub fn output_description(&self) -> Option<String> {
        self.engine.output_description()
    }

    /// The most recent failure, if the interface has not dismissed it.
    #[must_use]
    pub fn error(&self) -> Option<&AudioError> {
        self.error.as_ref()
    }

    /// Takes the most recent failure for display, clearing it.
    pub fn take_error(&mut self) -> Option<AudioError> {
        self.error.take()
    }

    /// Forgets the most recent failure without showing it.
    pub fn dismiss_error(&mut self) {
        self.error = None;
    }

    /// The volume the engine applies to the current track, in `0.0..=1.0`.
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.engine.volume()
    }

    /// Sets the playback volume, clamped to the accepted range.
    pub fn set_volume(&mut self, volume: f32) {
        self.engine.set_volume(volume);
    }

    /// How the queue behaves when a track ends.
    #[must_use]
    pub fn repeat(&self) -> RepeatMode {
        self.queue.repeat()
    }

    /// Sets the repeat mode.
    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.queue.set_repeat(mode);
    }

    /// Steps the repeat mode on to the next one and returns it.
    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.queue.cycle_repeat()
    }

    /// Whether the queue plays in random order.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.queue.shuffle()
    }

    /// Turns shuffle on or off and returns the new state.
    pub fn toggle_shuffle(&mut self) -> bool {
        self.queue.toggle_shuffle()
    }

    /// Adds every supported file in `paths` to the end of the queue.
    ///
    /// Files the engine cannot decode are skipped rather than refused, because
    /// a folder usually contains more than audio and the user asked to play
    /// what they selected, not to be told about a cover image. Returns how many
    /// tracks were added and leaves [`Playback::error`] set to the last skipped
    /// file so the interface can mention it.
    ///
    /// When the queue had no current track, the cursor moves onto the first
    /// track added, so the play button starts playing the selection the user
    /// just made.
    pub fn enqueue(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let was_idle = self.queue.current_index().is_none();
        let first_new = self.queue.len();
        let mut added = 0;
        let mut skipped: Option<PathBuf> = None;

        for path in paths {
            if !is_supported(&path) {
                skipped = Some(path);
                continue;
            }
            self.queue.push(QueueItem::from_path(path));
            added += 1;
        }

        if was_idle && added > 0 {
            // Nothing was selected before, so point the cursor at the first
            // track just added; the play button then starts the selection.
            self.queue.select(first_new);
        }

        if added > 0 {
            // A new track list means the old failure is no longer relevant.
            self.error = None;
        } else if let Some(path) = skipped {
            self.error = Some(AudioError::UnsupportedFormat { path });
        }

        added
    }

    /// Loads one path, expanding a folder into the audio files it holds.
    ///
    /// The play field and the command line both hand over whatever path the
    /// user picked, which may be a single file or a folder; the queue itself
    /// only ever holds files. Returns how many tracks were added, so the caller
    /// can tell "nothing playable here" from "queued".
    pub fn enqueue_path(&mut self, path: &Path) -> usize {
        if path.is_dir() {
            let entries = read_folder(path);
            self.enqueue(entries)
        } else {
            self.enqueue([path.to_path_buf()])
        }
    }

    /// Replaces the queue with `items` and starts playing the one at `index`.
    ///
    /// The library view plays a track the database describes rather than one a
    /// folder walk turned up, so it hands the queue the entries it already has:
    /// the title comes from the tags, and the order is the order the user saw
    /// on screen. An out-of-range `index` plays nothing, which is what a stale
    /// click after a rescan should do.
    pub fn play_items(&mut self, items: Vec<QueueItem>, index: usize) {
        self.queue.set_items(items);
        if self.queue.items().is_empty() {
            self.error = Some(AudioError::EmptyQueue);
            return;
        }

        self.error = None;
        self.play_index(index);
    }

    /// Appends `items` to the end of the queue, returning how many were added.
    ///
    /// This is "add to queue" as opposed to "play": nothing is loaded, and the
    /// track that is already playing is left alone. When the queue was empty,
    /// the cursor moves to the first entry added so the play button starts on
    /// the selection the user just made.
    pub fn queue_items(&mut self, items: impl IntoIterator<Item = QueueItem>) -> usize {
        let was_idle = self.queue.current_index().is_none();
        let first_new = self.queue.len();
        let mut added = 0;

        for item in items {
            if !is_supported(item.path()) {
                continue;
            }
            self.queue.push(item);
            added += 1;
        }

        if was_idle && added > 0 {
            self.queue.select(first_new);
        }

        added
    }

    /// Whether a track is loaded into the device.
    #[must_use]
    pub fn has_track(&self) -> bool {
        self.engine.has_track()
    }

    /// Whether the device is playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.engine.is_playing()
    }

    /// Whether a track is loaded but suspended.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.engine.is_paused()
    }

    /// The track the queue points at, whether or not it is loaded.
    #[must_use]
    pub fn current(&self) -> Option<&QueueItem> {
        self.queue.current()
    }

    /// The index of the current track and the number of tracks.
    #[must_use]
    pub fn queue_position(&self) -> Option<(usize, usize)> {
        self.queue.position()
    }

    /// How far into the loaded track playback has reached.
    #[must_use]
    pub fn position(&self) -> Duration {
        self.engine.position()
    }

    /// The loaded track's duration, when the decoder reported one.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.engine.duration()
    }

    /// Playback progress as a fraction in `0.0..=1.0`.
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.engine.progress()
    }

    /// Loads and plays the track at `index`, replacing whatever played before.
    ///
    /// Out-of-range indices are ignored, so a stale click cannot start the
    /// wrong track.
    pub fn play_index(&mut self, index: usize) {
        if self.queue.select(index).is_none() {
            return;
        }
        self.play_current();
    }

    /// Moves a queued track to another position in the queue.
    ///
    /// Reordering is not a transport action: whatever is playing keeps playing
    /// and the cue follows it to its new place. Returns `false` when either
    /// index is out of range, in which case nothing moves.
    pub fn move_in_queue(&mut self, from: usize, to: usize) -> bool {
        self.queue.move_item(from, to).is_some()
    }

    /// Takes a track out of the queue, returning it when it came out.
    ///
    /// The track being played is not removable: dropping it would leave the
    /// transport on a track the queue no longer holds. Remove the others, or
    /// skip past the current one first.
    pub fn remove_from_queue(&mut self, index: usize) -> Option<QueueItem> {
        if self.queue.current_index() == Some(index) {
            return None;
        }
        self.queue.remove(index)
    }

    /// Empties the queue and stops, so the transport never outlives the list.
    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.stop();
    }

    /// Loads and plays the track the queue currently points at.
    ///
    /// This is also the entry point for a simple "play this file": build a
    /// queue of one with [`Playback::enqueue`] and call this.
    pub fn play_current(&mut self) {
        let Some(item) = self.queue.current() else {
            self.error = Some(AudioError::EmptyQueue);
            return;
        };
        let path = item.path().to_path_buf();
        match self.engine.load(&path) {
            Ok(_) => self.error = None,
            Err(error) => self.error = Some(error),
        }
    }

    /// Starts playback, or pauses it when something is already playing.
    ///
    /// With nothing loaded yet but a queue to draw from, the first track is
    /// started, so the play button works from a cold start.
    pub fn toggle(&mut self) {
        if self.engine.is_playing() {
            self.engine.pause();
            return;
        }
        if !self.engine.has_track() {
            self.play_current();
            return;
        }
        if let Err(error) = self.engine.resume() {
            self.error = Some(error);
        }
    }

    /// Moves to the next track and plays it.
    ///
    /// At the end of a linear queue there is nothing to play: playback stops
    /// and the last track stays visible, which is what the playback bar should
    /// show in that state.
    pub fn next(&mut self) {
        if self.queue.advance().is_some() {
            self.play_current();
        } else {
            self.engine.stop();
        }
    }

    /// Moves to the previous track and plays it.
    pub fn previous(&mut self) {
        if self.queue.previous().is_some() {
            self.play_current();
        }
    }

    /// Stops playback, keeping the current track in the queue.
    pub fn stop(&mut self) {
        self.engine.stop();
    }

    /// Seeks to `position` within the loaded track.
    pub fn seek(&mut self, position: Duration) {
        if let Err(error) = self.engine.seek(position) {
            self.error = Some(error);
        }
    }

    /// Seeks to a fraction of the loaded track, as a progress bar reports.
    ///
    /// Does nothing when the duration is unknown, since there is no point to
    /// seek to.
    pub fn seek_fraction(&mut self, fraction: f32) {
        let Some(duration) = self.engine.duration() else {
            return;
        };
        let fraction = fraction.clamp(0.0, 1.0);
        self.seek(duration.mul_f32(fraction));
    }

    /// Advances the queue when the loaded track reaches its end.
    ///
    /// Call this once per frame. The engine reports the end exactly once per
    /// track, so the queue advances one track per natural end and never skips
    /// ahead.
    pub fn poll(&mut self) -> Option<PlaybackEvent> {
        if !self.engine.poll_finished() {
            return None;
        }
        self.next();
        Some(PlaybackEvent::TrackFinished)
    }
}

/// The audio files inside `folder`, sorted by name so an album plays in order.
///
/// The listing is one level deep, and unreadable folders yield nothing rather
/// than an error: a directory the process may not open should leave the queue
/// alone instead of failing the whole selection.
fn read_folder(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_supported(path))
        .collect();
    paths.sort();

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A player with no output device, so the tests run without a sound card.
    fn silent(paths: &[&str]) -> Playback {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);
        let added = playback.enqueue(paths.iter().map(PathBuf::from));
        assert_eq!(added, paths.len(), "the fixtures should be playable files");
        playback
    }

    #[test]
    fn a_player_without_a_device_still_holds_a_queue() {
        let playback = silent(&["/music/a.mp3", "/music/b.flac"]);

        assert!(!playback.is_available());
        assert_eq!(playback.queue().len(), 2);
        assert_eq!(playback.current().map(QueueItem::title), Some("a"));
        assert!(!playback.has_track());
        assert_eq!(playback.position(), Duration::ZERO);
        assert_eq!(playback.progress(), 0.0);
    }

    #[test]
    fn unplayable_files_are_skipped_and_reported() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);

        let added = playback.enqueue(["cover.png", "song.mp3"].map(PathBuf::from));

        assert_eq!(added, 1, "only the audio file should be queued");
        assert_eq!(playback.queue().len(), 1);
        assert_eq!(playback.current().map(QueueItem::title), Some("song"));
        // A successful add clears the record of the skipped file.
        assert!(playback.error().is_none());
    }

    #[test]
    fn a_queue_of_only_unplayable_files_reports_the_last_one() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);

        let added = playback.enqueue(["cover.png"].map(PathBuf::from));

        assert_eq!(added, 0);
        assert!(matches!(
            playback.error(),
            Some(AudioError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn playing_without_a_device_records_the_device_error() {
        let mut playback = silent(&["/music/a.mp3"]);

        playback.play_current();

        assert!(matches!(
            playback.error(),
            Some(AudioError::OutputUnavailable)
        ));
        assert!(!playback.is_playing());
    }

    #[test]
    fn the_transport_controls_move_through_the_queue_without_a_device() {
        let mut playback = silent(&["/music/a.mp3", "/music/b.mp3", "/music/c.mp3"]);

        playback.next();
        assert_eq!(playback.current().map(QueueItem::title), Some("b"));
        assert_eq!(
            playback.queue_position(),
            Some((2, 3)),
            "positions are one-based"
        );

        playback.previous();
        assert_eq!(playback.current().map(QueueItem::title), Some("a"));

        // Past the end of a linear queue there is nothing left to play.
        playback.next();
        playback.next();
        assert_eq!(playback.current().map(QueueItem::title), Some("c"));
        assert!(!playback.has_track());
    }

    #[test]
    fn repeat_one_keeps_the_same_track() {
        let mut playback = silent(&["/music/a.mp3", "/music/b.mp3"]);
        playback.set_repeat(RepeatMode::One);

        playback.next();

        assert_eq!(playback.current().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn volume_reaches_the_engine_and_is_clamped() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);

        playback.set_volume(2.0);
        assert_eq!(playback.volume(), 1.0);
        playback.set_volume(0.25);
        assert_eq!(playback.volume(), 0.25);
    }

    #[test]
    fn shuffle_and_repeat_are_owned_by_the_queue() {
        let mut playback = silent(&["/music/a.mp3", "/music/b.mp3"]);

        assert!(!playback.shuffle());
        assert!(playback.toggle_shuffle());
        assert!(playback.shuffle());

        let first = playback.repeat();
        assert_ne!(playback.cycle_repeat(), first);
    }

    #[test]
    fn an_empty_queue_cannot_start_playback() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);

        playback.play_current();
        assert!(matches!(playback.error(), Some(AudioError::EmptyQueue)));

        // The play button is harmless with nothing to play.
        playback.toggle();
        assert!(!playback.is_playing());
    }

    #[test]
    fn polling_without_a_loaded_track_does_nothing() {
        let mut playback = silent(&["/music/a.mp3"]);

        assert_eq!(playback.poll(), None);
        assert_eq!(playback.current().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn seeking_without_a_duration_is_ignored() {
        let mut playback = silent(&["/music/a.mp3"]);

        playback.seek_fraction(0.5);

        // No duration was reported, so there is no position to seek to and no
        // failure worth showing.
        assert!(playback.error().is_none());
    }

    #[test]
    fn errors_can_be_taken_once_and_are_then_gone() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 1.0);
        playback.play_current();

        assert!(playback.take_error().is_some());
        assert!(playback.error().is_none(), "taking the error clears it");

        playback.dismiss_error();
        assert!(playback.error().is_none());
    }

    #[test]
    fn playing_a_stale_index_is_ignored() {
        let mut playback = silent(&["/music/a.mp3"]);

        playback.play_index(7);

        assert_eq!(playback.current().map(QueueItem::title), Some("a"));
    }
}
