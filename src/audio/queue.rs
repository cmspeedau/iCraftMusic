//! The play queue: an ordered list of tracks plus the transport rules that
//! decide what plays next.
//!
//! The queue owns no audio state. It answers the questions the transport asks
//! ("what comes after this track?", "what did I play before?") so the answers
//! can be unit tested without an audio device.

use std::path::{Path, PathBuf};

/// How the queue behaves when the current track ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// Stop after the last track.
    #[default]
    Off,
    /// Start again from the first track.
    All,
    /// Play the current track again.
    One,
}

impl RepeatMode {
    /// The next mode in the repeat cycle, in the order the UI button cycles.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }

    /// A human readable name for the mode, used in tooltips.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Repeat off",
            Self::All => "Repeat all",
            Self::One => "Repeat one",
        }
    }

    /// Whether the mode is anything other than [`RepeatMode::Off`].
    #[must_use]
    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

/// A single entry in the play queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    path: PathBuf,
    title: String,
}

impl QueueItem {
    /// Builds an entry from a path and an explicit display title.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
        }
    }

    /// Builds an entry from a path, deriving the title from the file name.
    ///
    /// When the file name is not valid UTF-8 the lossy form is used, so the
    /// entry always has something displayable.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let title = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Self { path, title }
    }

    /// The file this entry points at.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The display title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// An ordered list of tracks with a cursor and the repeat/shuffle rules
/// applied to it.
#[derive(Debug)]
pub struct PlayQueue {
    items: Vec<QueueItem>,
    current: Option<usize>,
    repeat: RepeatMode,
    shuffle: bool,
    /// Track order in shuffle mode, newest first, for "previous".
    history: Vec<usize>,
    /// Indices already played in the current shuffle pass.
    played: Vec<usize>,
    /// Xorshift state; seeded from the clock so passes differ between runs.
    rng: u64,
}

impl Default for PlayQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current: None,
            repeat: RepeatMode::Off,
            shuffle: false,
            history: Vec::new(),
            played: Vec::new(),
            rng: seed_from_clock(),
        }
    }

    /// Creates a queue holding `items`, positioned on the first entry.
    #[must_use]
    pub fn from_items(items: Vec<QueueItem>) -> Self {
        let mut queue = Self::new();
        queue.set_items(items);
        queue
    }

    /// Replaces every entry and rewinds the cursor to the first track.
    pub fn set_items(&mut self, items: Vec<QueueItem>) {
        self.items = items;
        self.current = if self.items.is_empty() { None } else { Some(0) };
        self.history.clear();
        self.played.clear();
    }

    /// Every entry, in queue order.
    #[must_use]
    pub fn items(&self) -> &[QueueItem] {
        &self.items
    }

    /// The number of entries in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the queue holds no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The entry the cursor is on, if any.
    #[must_use]
    pub fn current(&self) -> Option<&QueueItem> {
        self.current.and_then(|index| self.items.get(index))
    }

    /// The cursor position as an index into [`PlayQueue::items`].
    #[must_use]
    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    /// The cursor as a one-based "(3 of 12)" pair for display.
    #[must_use]
    pub fn position(&self) -> Option<(usize, usize)> {
        self.current.map(|index| (index + 1, self.items.len()))
    }

    /// Moves the cursor to `index`, remembering the track we came from.
    ///
    /// Returns the selected entry, or [`None`] when `index` is out of range.
    pub fn select(&mut self, index: usize) -> Option<&QueueItem> {
        if index >= self.items.len() {
            return None;
        }
        if self.current != Some(index) {
            self.remember(index);
        }
        self.current = Some(index);
        self.current()
    }

    /// The queue index of `path`, if the queue holds it.
    #[must_use]
    pub fn contains(&self, path: &Path) -> Option<usize> {
        self.items.iter().position(|item| item.path == path)
    }

    /// Appends an entry and returns its index.
    ///
    /// The cursor does not move; callers decide whether the new track starts.
    pub fn push(&mut self, item: QueueItem) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    /// Removes the entry at `index`, keeping the cursor on a sensible track.
    pub fn remove(&mut self, index: usize) -> Option<QueueItem> {
        if index >= self.items.len() {
            return None;
        }
        let removed = self.items.remove(index);
        shift_after_removal(&mut self.history, index);
        shift_after_removal(&mut self.played, index);
        self.current = match self.current {
            Some(current) if current == index => {
                if self.items.is_empty() {
                    None
                } else {
                    Some(index.min(self.items.len() - 1))
                }
            }
            Some(current) if current > index => Some(current - 1),
            other => other,
        };
        Some(removed)
    }

    /// Moves the entry at `from` to position `to`, keeping the cursor on the
    /// track it was on.
    ///
    /// Reordering is a playlist edit, not a transport action: the entry that was
    /// playing stays the current one (the cursor follows it to its new index),
    /// and so do the shuffle bookkeeping lists. Returns the entry at its new
    /// position, or [`None`] when either index is out of range; an out-of-range
    /// reorder changes nothing at all.
    pub fn move_item(&mut self, from: usize, to: usize) -> Option<&QueueItem> {
        if from >= self.items.len() || to >= self.items.len() {
            return None;
        }
        if from == to {
            return self.items.get(to);
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        self.current = self
            .current
            .map(|current| shift_after_move(current, from, to));
        for index in &mut self.history {
            *index = shift_after_move(*index, from, to);
        }
        for index in &mut self.played {
            *index = shift_after_move(*index, from, to);
        }
        self.items.get(to)
    }

    /// Empties the queue and clears the cursor.
    pub fn clear(&mut self) {
        self.items.clear();
        self.current = None;
        self.history.clear();
        self.played.clear();
    }

    /// Moves to the next track according to the repeat and shuffle rules.
    ///
    /// Returns [`None`] when playback should stop: the last track of a
    /// non-repeating queue has finished.
    pub fn advance(&mut self) -> Option<&QueueItem> {
        let index = self.next_index()?;
        self.remember(index);
        self.current = Some(index);
        self.current()
    }

    /// Moves to the previously played track, staying put at the start of a
    /// linear queue.
    pub fn previous(&mut self) -> Option<&QueueItem> {
        if self.items.is_empty() {
            self.current = None;
            return None;
        }
        if self.shuffle {
            if let Some(index) = self.history.pop() {
                self.current = Some(index.min(self.items.len() - 1));
                return self.current();
            }
            return self.current();
        }
        self.current = match self.current {
            None => Some(0),
            Some(0) if self.repeat == RepeatMode::All => Some(self.items.len() - 1),
            Some(0) => Some(0),
            Some(index) => Some(index - 1),
        };
        self.current()
    }

    /// The current repeat mode.
    #[must_use]
    pub fn repeat(&self) -> RepeatMode {
        self.repeat
    }

    /// Sets the repeat mode.
    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    /// Advances the repeat mode and returns the new one.
    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.repeat = self.repeat.next();
        self.repeat
    }

    /// Whether shuffle is on.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    /// Turns shuffle on or off, keeping the cursor and restarting the pass.
    pub fn toggle_shuffle(&mut self) -> bool {
        self.shuffle = !self.shuffle;
        self.played.clear();
        self.history.clear();
        self.shuffle
    }

    /// The index the transport should move to next, without applying it.
    fn next_index(&mut self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        if self.repeat == RepeatMode::One
            && let Some(current) = self.current
        {
            return Some(current);
        }
        if self.shuffle {
            return self.next_shuffled();
        }
        match self.current {
            None => Some(0),
            Some(current) if current + 1 < self.items.len() => Some(current + 1),
            Some(_) if self.repeat == RepeatMode::All => Some(0),
            Some(_) => None,
        }
    }

    /// Picks a random track that has not been played in the current pass.
    fn next_shuffled(&mut self) -> Option<usize> {
        let len = self.items.len();
        let current = self.current;
        let mut candidates = self.unplayed_candidates(len, current);
        if candidates.is_empty() {
            // The pass is over: start a new one when repeating, otherwise stop.
            let repeating = self.repeat == RepeatMode::All || current.is_none();
            if !repeating {
                return None;
            }
            self.played.clear();
            if let Some(current) = current
                && len > 1
            {
                self.played.push(current);
            }
            candidates = self.unplayed_candidates(len, current);
        }
        if candidates.is_empty() {
            return current.or(Some(0));
        }
        let pick = (self.next_rand() % candidates.len() as u64) as usize;
        Some(candidates[pick])
    }

    /// Indices not yet played in this shuffle pass, excluding `current`.
    fn unplayed_candidates(&self, len: usize, current: Option<usize>) -> Vec<usize> {
        (0..len)
            .filter(|index| !self.played.contains(index) && Some(*index) != current)
            .collect()
    }

    /// Records the track we are leaving so [`PlayQueue::previous`] can find it,
    /// and the track we are entering so the shuffle pass knows it was played.
    ///
    /// Both lists exist for shuffle mode only, so a linear queue pays nothing.
    fn remember(&mut self, index: usize) {
        if !self.shuffle {
            return;
        }
        if let Some(current) = self.current {
            if current != index {
                self.history.push(current);
            }
            if !self.played.contains(&current) {
                self.played.push(current);
            }
        }
        if !self.played.contains(&index) {
            self.played.push(index);
        }
    }

    /// Advances the xorshift state.
    fn next_rand(&mut self) -> u64 {
        let mut state = self.rng;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.rng = state;
        state
    }
}

/// Drops `removed` from a list of indices and shifts the ones after it down.
fn shift_after_removal(indices: &mut Vec<usize>, removed: usize) {
    indices.retain(|index| *index != removed);
    for index in indices.iter_mut() {
        if *index > removed {
            *index -= 1;
        }
    }
}

/// Rewrites one queue index after the entry at `from` moved to `to`.
///
/// The list is treated as a remove-then-insert: everything between the two
/// positions slides one step towards where the gap was, and the moved entry
/// lands on `to`.
fn shift_after_move(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}

/// A non-zero seed for the shuffle generator.
fn seed_from_clock() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15);
    // xorshift cannot recover from zero.
    if nanos == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_of(titles: &[&str]) -> PlayQueue {
        let items = titles
            .iter()
            .map(|title| QueueItem::new(format!("/music/{title}.mp3"), *title))
            .collect();
        PlayQueue::from_items(items)
    }

    fn current_title(queue: &PlayQueue) -> Option<&str> {
        queue.current().map(QueueItem::title)
    }

    /// The queue's titles in order, for asserting on a reorder.
    fn titles(queue: &PlayQueue) -> Vec<&str> {
        queue.items().iter().map(QueueItem::title).collect()
    }

    #[test]
    fn starts_on_the_first_track() {
        let queue = queue_of(&["a", "b", "c"]);

        assert_eq!(current_title(&queue), Some("a"));
        assert_eq!(queue.position(), Some((1, 3)));
        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());
    }

    #[test]
    fn an_empty_queue_has_nothing_to_advance_to() {
        let mut queue = PlayQueue::new();

        assert!(queue.current().is_none());
        assert!(queue.position().is_none());
        assert!(queue.advance().is_none());
    }

    #[test]
    fn advance_walks_the_queue_then_stops() {
        let mut queue = queue_of(&["a", "b", "c"]);

        assert_eq!(queue.advance().map(QueueItem::title), Some("b"));
        assert_eq!(queue.advance().map(QueueItem::title), Some("c"));
        assert!(queue.advance().is_none(), "the queue does not repeat");
        assert_eq!(current_title(&queue), Some("c"));
    }

    #[test]
    fn repeat_all_wraps_around() {
        let mut queue = queue_of(&["a", "b"]);
        queue.set_repeat(RepeatMode::All);

        assert_eq!(queue.advance().map(QueueItem::title), Some("b"));
        assert_eq!(queue.advance().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn repeat_one_stays_on_the_same_track() {
        let mut queue = queue_of(&["a", "b"]);
        queue.set_repeat(RepeatMode::One);

        assert_eq!(queue.advance().map(QueueItem::title), Some("a"));
        assert_eq!(queue.advance().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn repeat_cycles_off_all_one() {
        let mut queue = queue_of(&["a"]);

        assert_eq!(queue.repeat(), RepeatMode::Off);
        assert_eq!(queue.cycle_repeat(), RepeatMode::All);
        assert_eq!(queue.cycle_repeat(), RepeatMode::One);
        assert_eq!(queue.cycle_repeat(), RepeatMode::Off);
    }

    #[test]
    fn repeat_labels_describe_the_mode() {
        assert_eq!(RepeatMode::Off.label(), "Repeat off");
        assert!(!RepeatMode::Off.is_enabled());
        assert!(RepeatMode::One.is_enabled());
    }

    #[test]
    fn previous_walks_back_and_stops_at_the_start() {
        let mut queue = queue_of(&["a", "b", "c"]);

        queue.advance();
        queue.advance();
        assert_eq!(current_title(&queue), Some("c"));
        assert_eq!(queue.previous().map(QueueItem::title), Some("b"));
        assert_eq!(queue.previous().map(QueueItem::title), Some("a"));
        assert_eq!(queue.previous().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn previous_wraps_when_repeating() {
        let mut queue = queue_of(&["a", "b", "c"]);
        queue.set_repeat(RepeatMode::All);

        assert_eq!(queue.previous().map(QueueItem::title), Some("c"));
    }

    #[test]
    fn select_moves_the_cursor_and_rejects_bad_indices() {
        let mut queue = queue_of(&["a", "b", "c"]);

        assert_eq!(queue.select(2).map(QueueItem::title), Some("c"));
        assert_eq!(queue.current_index(), Some(2));
        assert!(queue.select(3).is_none());
        assert_eq!(
            current_title(&queue),
            Some("c"),
            "a bad index changes nothing"
        );
    }

    #[test]
    fn select_then_previous_returns_to_the_played_track() {
        let mut queue = queue_of(&["a", "b"]);
        queue.toggle_shuffle();

        queue.select(1);
        assert_eq!(queue.previous().map(QueueItem::title), Some("a"));
    }

    #[test]
    fn shuffle_plays_each_track_once_before_repeating() {
        let mut queue = queue_of(&["a", "b", "c", "d"]);
        queue.toggle_shuffle();
        assert!(queue.shuffle());

        let mut seen = vec![current_title(&queue).unwrap().to_owned()];
        while let Some(item) = queue.advance() {
            seen.push(item.title().to_owned());
        }

        assert_eq!(
            seen.len(),
            4,
            "a non-repeating shuffle stops after one pass"
        );
        let mut sorted = seen.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn shuffled_queue_keeps_going_when_repeating() {
        let mut queue = queue_of(&["a", "b", "c"]);
        queue.toggle_shuffle();
        queue.set_repeat(RepeatMode::All);

        for _ in 0..12 {
            assert!(queue.advance().is_some(), "a repeating shuffle never stops");
        }
    }

    #[test]
    fn a_single_track_survives_shuffle() {
        let mut queue = queue_of(&["only"]);
        queue.toggle_shuffle();
        queue.set_repeat(RepeatMode::All);

        assert_eq!(queue.advance().map(QueueItem::title), Some("only"));
    }

    #[test]
    fn push_and_remove_keep_the_cursor_on_a_real_track() {
        let mut queue = queue_of(&["a", "b"]);
        queue.select(1);

        let index = queue.push(QueueItem::from_path("/music/c.mp3"));
        assert_eq!(index, 2);
        assert_eq!(current_title(&queue), Some("b"), "pushing does not jump");

        let removed = queue.remove(1).expect("the entry exists");
        assert_eq!(removed.title(), "b");
        assert_eq!(current_title(&queue), Some("c"), "the cursor slides down");
        assert_eq!(queue.len(), 2);

        assert!(queue.remove(5).is_none());
    }

    #[test]
    fn removing_the_last_track_clears_the_cursor() {
        let mut queue = queue_of(&["a"]);

        assert!(queue.remove(0).is_some());
        assert!(queue.current().is_none());
        assert!(queue.is_empty());
        assert!(queue.advance().is_none());
    }

    #[test]
    fn removing_before_the_cursor_keeps_the_same_track_playing() {
        let mut queue = queue_of(&["a", "b", "c"]);
        queue.select(2);

        queue.remove(0);
        assert_eq!(current_title(&queue), Some("c"));
        assert_eq!(queue.current_index(), Some(1));
    }

    #[test]
    fn moving_a_track_reorders_the_queue_and_follows_the_cursor() {
        let mut queue = queue_of(&["a", "b", "c", "d"]);
        queue.select(2);

        assert_eq!(queue.move_item(2, 0).map(QueueItem::title), Some("c"));
        assert_eq!(titles(&queue), ["c", "a", "b", "d"]);
        assert_eq!(
            current_title(&queue),
            Some("c"),
            "the track being played is still the current one after a reorder"
        );
        assert_eq!(queue.current_index(), Some(0));

        // Moving it back where it came from leaves the queue as it started.
        assert_eq!(queue.move_item(0, 2).map(QueueItem::title), Some("c"));
        assert_eq!(titles(&queue), ["a", "b", "c", "d"]);
        assert_eq!(queue.current_index(), Some(2));
    }

    #[test]
    fn moving_a_track_past_the_cursor_leaves_the_cursor_behind() {
        let mut queue = queue_of(&["a", "b", "c", "d"]);
        queue.select(1);

        queue.move_item(3, 0);
        assert_eq!(titles(&queue), ["d", "a", "b", "c"]);
        assert_eq!(current_title(&queue), Some("b"));
        assert_eq!(queue.current_index(), Some(2));
    }

    #[test]
    fn move_item_rejects_bad_indices_and_ignores_a_no_op() {
        let mut queue = queue_of(&["a", "b"]);

        assert!(queue.move_item(0, 2).is_none());
        assert!(queue.move_item(7, 0).is_none());
        assert_eq!(
            titles(&queue),
            ["a", "b"],
            "a rejected move changes nothing"
        );

        assert_eq!(queue.move_item(1, 1).map(QueueItem::title), Some("b"));
        assert_eq!(titles(&queue), ["a", "b"]);
    }

    #[test]
    fn moving_a_track_rewrites_the_shuffle_history() {
        let mut queue = queue_of(&["a", "b", "c"]);
        assert!(queue.toggle_shuffle());
        queue.select(2);
        queue.select(1);

        // "c" was left behind at index 2, so it is what Previous comes back to.
        queue.move_item(2, 0);

        assert_eq!(queue.previous().map(QueueItem::title), Some("c"));
    }

    #[test]
    fn clear_empties_everything() {
        let mut queue = queue_of(&["a", "b"]);
        queue.advance();

        queue.clear();
        assert!(queue.is_empty());
        assert!(queue.current().is_none());
        assert!(queue.previous().is_none());
    }

    #[test]
    fn contains_finds_a_queued_path() {
        let queue = queue_of(&["a", "b"]);

        assert_eq!(queue.contains(Path::new("/music/b.mp3")), Some(1));
        assert_eq!(queue.contains(Path::new("/music/z.mp3")), None);
    }

    #[test]
    fn titles_come_from_the_file_name() {
        let item = QueueItem::from_path("/music/Artist - Song.flac");
        assert_eq!(item.title(), "Artist - Song");
        assert_eq!(item.path(), Path::new("/music/Artist - Song.flac"));

        let explicit = QueueItem::new("/music/track01.wav", "Opening Theme");
        assert_eq!(explicit.title(), "Opening Theme");
    }

    #[test]
    fn set_items_rewinds_and_keeps_the_mode() {
        let mut queue = queue_of(&["a", "b", "c"]);
        queue.advance();
        queue.set_repeat(RepeatMode::All);
        queue.toggle_shuffle();

        queue.set_items(vec![QueueItem::new("/music/x.mp3", "x")]);
        assert_eq!(current_title(&queue), Some("x"));
        assert_eq!(queue.repeat(), RepeatMode::All);
        assert!(queue.shuffle());
    }
}
