//! What the Library view is showing, and what the user has selected in it.
//!
//! The view itself is stateless: it reads the state held here and calls back
//! into [`crate::app::MusicCraftApp`] to act. Keeping the browsing state out of
//! the rendering code means the interesting behaviour — which rows a query
//! leaves visible, how a click with a modifier changes the selection — can be
//! tested without opening a window.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::library::track::{Album, Artist, Track, TrackId};
use crate::search::{Query, Sort, SortKey, filter_artists, filter_tracks};

/// Which page of the Library is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseKind {
    /// Every indexed track, as a sortable table.
    #[default]
    Tracks,
    /// The albums the tracks add up to, with their cover art.
    Albums,
    /// The artists credited on the indexed tracks.
    Artists,
}

impl BrowseKind {
    /// Every page, in the order the view offers them.
    pub const ALL: [BrowseKind; 3] = [Self::Tracks, Self::Albums, Self::Artists];

    /// Label shown on the page's entry in the sidebar.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Tracks => "Tracks",
            Self::Albums => "Albums",
            Self::Artists => "Artists",
        }
    }
}

/// How a click on a row should change the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClickMode {
    /// Select just this row.
    #[default]
    Replace,
    /// Add the row, or remove it when it was already selected.
    Toggle,
    /// Select everything between the anchor and this row.
    Range,
}

impl ClickMode {
    /// The mode the pressed modifier keys ask for.
    ///
    /// A range click (shift) takes precedence over an additive click
    /// (control or command), which is what a user of a table expects.
    #[must_use]
    pub fn from_modifiers(additive: bool, range: bool) -> Self {
        if range {
            Self::Range
        } else if additive {
            Self::Toggle
        } else {
            Self::Replace
        }
    }
}

/// The tracks the user has picked out of the current list.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    ids: BTreeSet<TrackId>,
    /// Where a shift-click ranges from.
    anchor: Option<TrackId>,
}

impl Selection {
    /// An empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many tracks are selected.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Whether `id` is selected.
    #[must_use]
    pub fn contains(&self, id: TrackId) -> bool {
        self.ids.contains(&id)
    }

    /// Every selected identifier, in library order.
    pub fn ids(&self) -> impl Iterator<Item = TrackId> + '_ {
        self.ids.iter().copied()
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.ids.clear();
        self.anchor = None;
    }

    /// Forget identifiers that are no longer in `visible`.
    ///
    /// A rescan or a new query removes rows; selecting tracks that are gone
    /// would leave "play the selection" with nothing to play.
    pub fn retain(&mut self, visible: &[TrackId]) {
        self.ids.retain(|id| visible.contains(id));

        if self.anchor.is_some_and(|anchor| !visible.contains(&anchor)) {
            self.anchor = None;
        }
    }

    /// Change the selection as a click on `id` asks.
    ///
    /// `visible` is the list as it is currently shown, which is what a range
    /// click is measured against.
    pub fn apply(&mut self, visible: &[TrackId], id: TrackId, mode: ClickMode) {
        match mode {
            ClickMode::Replace => {
                self.ids.clear();
                self.ids.insert(id);
                self.anchor = Some(id);
            }
            ClickMode::Toggle => {
                if !self.ids.remove(&id) {
                    self.ids.insert(id);
                }
                self.anchor = Some(id);
            }
            ClickMode::Range => {
                let Some(anchor) = self.anchor.filter(|anchor| visible.contains(anchor)) else {
                    // Nothing to range from, so behave like a plain click.
                    self.ids.clear();
                    self.ids.insert(id);
                    self.anchor = Some(id);
                    return;
                };

                let (Some(start), Some(end)) = (
                    visible.iter().position(|candidate| *candidate == anchor),
                    visible.iter().position(|candidate| *candidate == id),
                ) else {
                    return;
                };

                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };

                self.ids.clear();
                self.ids.extend(visible[start..=end].iter().copied());
            }
        }
    }
}

/// An album plus the cover art the interface should draw for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumCard {
    /// The album itself.
    pub album: Album,
    /// Cached cover art taken from one of the album's tracks, when any.
    pub artwork: Option<PathBuf>,
}

/// Which tracks of an album can supply its cover art.
///
/// The first track with cached artwork wins; the rest of the album shares it,
/// because cover art is attached to every track of an album by the tagger.
fn album_artwork(album: &Album, tracks: &[Track]) -> Option<PathBuf> {
    tracks
        .iter()
        .filter(|track| track.album.as_deref() == Some(album.name.as_str()))
        .find_map(|track| track.artwork_path.clone())
}

/// Everything the Library view needs to draw itself.
#[derive(Debug, Default)]
pub struct LibraryBrowse {
    kind: BrowseKind,
    query_text: String,
    sort: Sort,
    selection: Selection,
    tracks: Vec<Track>,
    albums: Vec<AlbumCard>,
    artists: Vec<Artist>,
}

impl LibraryBrowse {
    /// An empty library, showing tracks.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Which page is open.
    #[must_use]
    pub fn kind(&self) -> BrowseKind {
        self.kind
    }

    /// Switch pages.
    ///
    /// The selection is kept: a track picked in the table stays picked when the
    /// user looks at the album it belongs to.
    pub fn set_kind(&mut self, kind: BrowseKind) {
        self.kind = kind;
    }

    /// The raw text in the search field.
    #[must_use]
    pub fn query_text(&self) -> &str {
        &self.query_text
    }

    /// The search field's text, for the view to edit.
    pub fn query_text_mut(&mut self) -> &mut String {
        &mut self.query_text
    }

    /// Replace the search text.
    pub fn set_query_text(&mut self, text: impl Into<String>) {
        self.query_text = text.into();
    }

    /// The normalised query matching and filtering use.
    #[must_use]
    pub fn query(&self) -> Query {
        Query::new(&self.query_text)
    }

    /// The column and direction the track table is ordered by.
    #[must_use]
    pub fn sort(&self) -> Sort {
        self.sort
    }

    /// Order the track table by `sort`.
    pub fn set_sort(&mut self, sort: Sort) {
        self.sort = sort;
    }

    /// Sort by `key`, flipping the direction when it is already the active one.
    pub fn toggle_sort(&mut self, key: SortKey) {
        self.sort = self.sort.clicked(key);
    }

    /// The current selection.
    #[must_use]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// The current selection, mutably, for the view's click handling.
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Replace everything a rescan or a fresh read of the database produced.
    ///
    /// Album cards are derived here rather than by the database because the
    /// cover art a card shows depends on the tracks, not on the album row.
    pub fn replace(&mut self, tracks: Vec<Track>, albums: Vec<Album>, artists: Vec<Artist>) {
        self.albums = albums
            .into_iter()
            .map(|album| AlbumCard {
                artwork: album_artwork(&album, &tracks),
                album,
            })
            .collect();

        self.tracks = tracks;
        self.artists = artists;

        let ids = self.visible_track_ids();
        self.selection.retain(&ids);
    }

    /// Every indexed track.
    #[must_use]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Every album.
    #[must_use]
    pub fn albums(&self) -> &[AlbumCard] {
        &self.albums
    }

    /// Every artist.
    #[must_use]
    pub fn artists(&self) -> &[Artist] {
        &self.artists
    }

    /// How many tracks are indexed, whether or not the query hides them.
    #[must_use]
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// How many albums are indexed.
    #[must_use]
    pub fn album_count(&self) -> usize {
        self.albums.len()
    }

    /// How many artists are indexed.
    #[must_use]
    pub fn artist_count(&self) -> usize {
        self.artists.len()
    }

    /// The tracks the table should draw, filtered and ordered.
    #[must_use]
    pub fn visible_tracks(&self) -> Vec<&Track> {
        filter_tracks(&self.tracks, &self.query(), self.sort)
    }

    /// The albums the grid should draw, filtered by name.
    #[must_use]
    pub fn visible_albums(&self) -> Vec<&AlbumCard> {
        let query = self.query();
        let mut matched: Vec<&AlbumCard> = self
            .albums
            .iter()
            .filter(|card| crate::search::matches_album(&card.album, &query))
            .collect();

        matched.sort_by(|left, right| left.album.name.cmp(&right.album.name));
        matched
    }

    /// The artists the list should draw, filtered by name.
    #[must_use]
    pub fn visible_artists(&self) -> Vec<&Artist> {
        filter_artists(&self.artists, &self.query())
    }

    /// The identifiers of the visible tracks, in the order they are drawn.
    ///
    /// Tracks without a row identifier cannot be selected, so they are left
    /// out; in practice every track read from the database has one.
    #[must_use]
    pub fn visible_track_ids(&self) -> Vec<TrackId> {
        self.visible_tracks()
            .into_iter()
            .filter_map(|track| track.id)
            .collect()
    }

    /// The selected tracks, in the order they are drawn.
    #[must_use]
    pub fn selected_tracks(&self) -> Vec<&Track> {
        self.visible_tracks()
            .into_iter()
            .filter(|track| track.id.is_some_and(|id| self.selection.contains(id)))
            .collect()
    }

    /// Whether the library holds no tracks at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// Whether a query is hiding some of the library.
    #[must_use]
    pub fn is_filtered(&self) -> bool {
        !self.query().is_empty()
    }

    /// The tracks of one album, in album order.
    #[must_use]
    pub fn tracks_of_album(&self, album: &str) -> Vec<&Track> {
        let mut tracks: Vec<&Track> = self
            .tracks
            .iter()
            .filter(|track| track.album.as_deref() == Some(album))
            .collect();

        tracks.sort_by(|left, right| {
            crate::search::compare_tracks(
                left,
                right,
                Sort::new(
                    SortKey::TrackNumber,
                    crate::search::SortDirection::Ascending,
                ),
            )
        });
        tracks
    }

    /// The tracks credited to one artist, in library order.
    #[must_use]
    pub fn tracks_of_artist(&self, artist: &str) -> Vec<&Track> {
        self.tracks
            .iter()
            .filter(|track| track.display_artist() == Some(artist))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn track(id: i64, title: &str, artist: Option<&str>, album: Option<&str>) -> Track {
        let mut track = Track::from_path(Path::new(&format!("/music/{title}.mp3")));
        track.id = Some(TrackId(id));
        track.title = title.to_owned();
        track.artist = artist.map(str::to_owned);
        track.album = album.map(str::to_owned);
        track
    }

    fn album(name: &str, track_count: usize) -> Album {
        Album {
            name: name.to_owned(),
            album_artist: None,
            track_count,
            duration: None,
        }
    }

    fn credited_album(name: &str, album_artist: &str, track_count: usize) -> Album {
        Album {
            album_artist: Some(album_artist.to_owned()),
            ..album(name, track_count)
        }
    }

    #[test]
    fn a_plain_click_replaces_the_selection() {
        let visible = vec![TrackId(1), TrackId(2), TrackId(3)];
        let mut selection = Selection::new();

        selection.apply(&visible, TrackId(1), ClickMode::Replace);
        selection.apply(&visible, TrackId(3), ClickMode::Replace);

        assert_eq!(selection.len(), 1);
        assert!(selection.contains(TrackId(3)));
    }

    #[test]
    fn an_additive_click_toggles_one_row() {
        let visible = vec![TrackId(1), TrackId(2)];
        let mut selection = Selection::new();

        selection.apply(&visible, TrackId(1), ClickMode::Replace);
        selection.apply(&visible, TrackId(2), ClickMode::Toggle);
        assert_eq!(selection.len(), 2);

        selection.apply(&visible, TrackId(2), ClickMode::Toggle);
        assert_eq!(selection.len(), 1);
        assert!(selection.contains(TrackId(1)));
    }

    #[test]
    fn a_range_click_selects_everything_between_the_anchor_and_the_row() {
        let visible = vec![TrackId(1), TrackId(2), TrackId(3), TrackId(4)];
        let mut selection = Selection::new();

        selection.apply(&visible, TrackId(4), ClickMode::Replace);
        selection.apply(&visible, TrackId(2), ClickMode::Range);

        assert_eq!(selection.len(), 3);
        assert!(selection.contains(TrackId(2)));
        assert!(selection.contains(TrackId(3)));
        assert!(selection.contains(TrackId(4)));
        assert!(!selection.contains(TrackId(1)));
    }

    #[test]
    fn a_range_click_without_an_anchor_behaves_like_a_plain_click() {
        let visible = vec![TrackId(1), TrackId(2)];
        let mut selection = Selection::new();

        selection.apply(&visible, TrackId(2), ClickMode::Range);

        assert_eq!(selection.len(), 1);
        assert!(selection.contains(TrackId(2)));
    }

    #[test]
    fn a_range_click_ignores_an_anchor_that_is_no_longer_visible() {
        let visible = vec![TrackId(2), TrackId(3)];
        let mut selection = Selection::new();

        selection.apply(&visible, TrackId(1), ClickMode::Replace);
        selection.apply(&visible, TrackId(3), ClickMode::Range);

        assert_eq!(selection.len(), 1);
        assert!(selection.contains(TrackId(3)));
    }

    #[test]
    fn the_modifiers_pick_the_click_mode() {
        assert_eq!(ClickMode::from_modifiers(false, false), ClickMode::Replace);
        assert_eq!(ClickMode::from_modifiers(true, false), ClickMode::Toggle);
        assert_eq!(ClickMode::from_modifiers(false, true), ClickMode::Range);
        assert_eq!(
            ClickMode::from_modifiers(true, true),
            ClickMode::Range,
            "shift outranks control"
        );
    }

    #[test]
    fn replacing_the_contents_drops_rows_that_are_gone() {
        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![
                track(1, "Xtal", None, None),
                track(2, "Roygbiv", None, None),
            ],
            vec![],
            vec![],
        );

        let visible = browse.visible_track_ids();
        browse
            .selection_mut()
            .apply(&visible, TrackId(2), ClickMode::Replace);

        browse.replace(vec![track(1, "Xtal", None, None)], vec![], vec![]);

        assert!(
            browse.selection().is_empty(),
            "a selection that no longer exists should not survive a refresh"
        );
    }

    #[test]
    fn album_cards_take_their_artwork_from_the_tracks() {
        let mut first = track(1, "Roygbiv", None, Some("Music Has the Right"));
        first.artwork_path = Some(PathBuf::from("/cache/cover.jpg"));
        let second = track(2, "Rue The Whirl", None, Some("Music Has the Right"));

        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![first, second],
            vec![album("Music Has the Right", 2), album("No Art", 1)],
            vec![],
        );

        let cards = browse.albums();
        assert_eq!(cards[0].artwork, Some(PathBuf::from("/cache/cover.jpg")));
        assert_eq!(
            cards[1].artwork, None,
            "an album with no art has no card art"
        );
    }

    #[test]
    fn the_query_filters_every_page() {
        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![
                track(
                    1,
                    "Xtal",
                    Some("Aphex Twin"),
                    Some("Selected Ambient Works"),
                ),
                track(
                    2,
                    "Roygbiv",
                    Some("Boards of Canada"),
                    Some("Music Has the Right"),
                ),
            ],
            vec![
                credited_album("Selected Ambient Works", "Aphex Twin", 1),
                credited_album("Music Has the Right", "Boards of Canada", 1),
            ],
            vec![
                Artist {
                    name: "Aphex Twin".to_owned(),
                    album_count: 1,
                    track_count: 1,
                },
                Artist {
                    name: "Boards of Canada".to_owned(),
                    album_count: 1,
                    track_count: 1,
                },
            ],
        );

        assert_eq!(browse.visible_tracks().len(), 2);
        assert_eq!(browse.visible_albums().len(), 2);
        assert_eq!(browse.visible_artists().len(), 2);

        browse.set_query_text("aphex");

        assert_eq!(browse.visible_tracks().len(), 1);
        assert_eq!(browse.visible_albums().len(), 1);
        assert_eq!(browse.visible_artists().len(), 1);
        assert!(browse.is_filtered());
    }

    #[test]
    fn the_table_sorts_by_the_active_column() {
        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![
                track(1, "Xtal", Some("Aphex Twin"), None),
                track(2, "Roygbiv", Some("Boards of Canada"), None),
            ],
            vec![],
            vec![],
        );

        let titles = |browse: &LibraryBrowse| -> Vec<String> {
            browse
                .visible_tracks()
                .into_iter()
                .map(|track| track.title.clone())
                .collect()
        };

        assert_eq!(titles(&browse), vec!["Roygbiv", "Xtal"]);

        browse.toggle_sort(SortKey::Title);
        assert_eq!(titles(&browse), vec!["Xtal", "Roygbiv"]);

        browse.toggle_sort(SortKey::Artist);
        assert_eq!(
            titles(&browse),
            vec!["Xtal", "Roygbiv"],
            "a new column starts ascending"
        );
    }

    #[test]
    fn the_selection_is_read_back_in_the_order_it_is_drawn() {
        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![
                track(1, "Xtal", None, None),
                track(2, "Roygbiv", None, None),
                track(3, "Aquarius", None, None),
            ],
            vec![],
            vec![],
        );

        let visible = browse.visible_track_ids();
        let drawn: Vec<String> = browse
            .visible_tracks()
            .into_iter()
            .map(|track| track.title.clone())
            .collect();

        browse
            .selection_mut()
            .apply(&visible, visible[0], ClickMode::Replace);
        browse
            .selection_mut()
            .apply(&visible, visible[2], ClickMode::Toggle);

        let selected: Vec<String> = browse
            .selected_tracks()
            .into_iter()
            .map(|track| track.title.clone())
            .collect();

        // Read back in the order the rows are drawn in, not the order they were
        // clicked in.
        assert_eq!(selected, vec![drawn[0].clone(), drawn[2].clone()]);
    }

    #[test]
    fn an_albums_tracks_come_back_in_disc_order() {
        let mut first = track(1, "Second", None, Some("Album"));
        first.track_number = Some(2);
        let mut second = track(2, "First", None, Some("Album"));
        second.track_number = Some(1);
        let other = track(3, "Elsewhere", None, Some("Another"));

        let mut browse = LibraryBrowse::new();
        browse.replace(vec![first, second, other], vec![], vec![]);

        let titles: Vec<&str> = browse
            .tracks_of_album("Album")
            .into_iter()
            .map(|track| track.title.as_str())
            .collect();

        assert_eq!(titles, vec!["First", "Second"]);
    }

    #[test]
    fn an_artists_tracks_are_found_by_the_displayed_artist() {
        let mut compilation = track(1, "Untitled", None, None);
        compilation.album_artist = Some("Various Artists".to_owned());
        let other = track(2, "Xtal", Some("Aphex Twin"), None);

        let mut browse = LibraryBrowse::new();
        browse.replace(vec![compilation, other], vec![], vec![]);

        assert_eq!(browse.tracks_of_artist("Various Artists").len(), 1);
        assert_eq!(browse.tracks_of_artist("Aphex Twin").len(), 1);
    }

    #[test]
    fn counts_describe_the_whole_library_not_the_visible_rows() {
        let mut browse = LibraryBrowse::new();
        browse.replace(
            vec![
                track(1, "Xtal", None, None),
                track(2, "Roygbiv", None, None),
            ],
            vec![album("Album", 2)],
            vec![Artist {
                name: "Artist".to_owned(),
                album_count: 1,
                track_count: 2,
            }],
        );
        browse.set_query_text("xtal");

        assert_eq!(browse.track_count(), 2);
        assert_eq!(browse.album_count(), 1);
        assert_eq!(browse.artist_count(), 1);
        assert_eq!(browse.visible_tracks().len(), 1);
    }
}
