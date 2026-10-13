//! Searching and ordering the music library.
//!
//! The library view needs to answer two questions while the user types and
//! clicks: which tracks match what was typed, and in what order should they be
//! shown. Both are pure functions over the tracks the database already
//! returned, so they live here rather than in the view module: they are easy to
//! test without a window, and a query never touches the database, which is what
//! keeps typing responsive on a large library.
//!
//! Matching is deliberately simple — a case-insensitive substring search across
//! the fields a user can see. It is fast enough for tens of thousands of tracks
//! and predictable, which matters more than clever ranking in a desktop
//! library.

use std::cmp::Ordering;
use std::time::Duration;

use crate::library::{Album, Artist, Track};

/// The column a list of tracks is ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    /// Track title.
    #[default]
    Title,
    /// Performing artist.
    Artist,
    /// Album the track belongs to.
    Album,
    /// Free-form genre.
    Genre,
    /// Playing time.
    Duration,
    /// Position within the album.
    TrackNumber,
    /// Full path to the file.
    Path,
    /// How many times the track has been played.
    PlayCount,
}

impl SortKey {
    /// Every key, in the order the track table shows its columns.
    pub const ALL: [SortKey; 6] = [
        Self::Title,
        Self::Artist,
        Self::Album,
        Self::Genre,
        Self::Duration,
        Self::PlayCount,
    ];

    /// Column heading for this key.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::Genre => "Genre",
            Self::Duration => "Duration",
            Self::TrackNumber => "Track",
            Self::Path => "Path",
            Self::PlayCount => "Plays",
        }
    }
}

/// Which way a sorted list runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Smallest first.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

impl SortDirection {
    /// The opposite direction.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Arrow shown next to the active column heading.
    #[must_use]
    pub fn arrow(self) -> &'static str {
        match self {
            Self::Ascending => "▲",
            Self::Descending => "▼",
        }
    }
}

/// A column and a direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sort {
    /// Column being ordered by.
    pub key: SortKey,
    /// Direction that column runs in.
    pub direction: SortDirection,
}

impl Sort {
    /// A sort of `key`, ascending.
    #[must_use]
    pub fn new(key: SortKey, direction: SortDirection) -> Self {
        Self { key, direction }
    }

    /// What clicking `key`'s column heading should do.
    ///
    /// Clicking the active column flips the direction; clicking another column
    /// starts it ascending, which is what a user expects from a table header.
    #[must_use]
    pub fn clicked(self, key: SortKey) -> Self {
        if self.key == key {
            Self {
                key,
                direction: self.direction.toggled(),
            }
        } else {
            Self {
                key,
                direction: SortDirection::Ascending,
            }
        }
    }
}

/// What the user typed into the search field, normalised for matching.
///
/// The text is trimmed and lowercased once, when the query is built, so a
/// keystroke does not re-normalise every track on every frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    text: String,
}

impl Query {
    /// Build a query from raw input.
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self {
            text: text.trim().to_lowercase(),
        }
    }

    /// The normalised text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether the query matches everything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl From<&str> for Query {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

/// Whether `text` contains the query, ignoring case.
fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

/// Whether a track matches the query.
///
/// Every field the interface can show is searched, so typing an album name, a
/// genre or a fragment of the file path all work.
#[must_use]
pub fn matches_track(track: &Track, query: &Query) -> bool {
    if query.is_empty() {
        return true;
    }

    let needle = query.as_str();

    [
        Some(track.title.as_str()),
        track.artist.as_deref(),
        track.album.as_deref(),
        track.album_artist.as_deref(),
        track.genre.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|field| contains(field, needle))
        || contains(&track.path.to_string_lossy(), needle)
}

/// Whether an album matches the query.
#[must_use]
pub fn matches_album(album: &Album, query: &Query) -> bool {
    if query.is_empty() {
        return true;
    }

    contains(&album.name, query.as_str())
        || album
            .album_artist
            .as_deref()
            .is_some_and(|artist| contains(artist, query.as_str()))
}

/// Whether an artist matches the query.
#[must_use]
pub fn matches_artist(artist: &Artist, query: &Query) -> bool {
    if query.is_empty() {
        return true;
    }

    contains(&artist.name, query.as_str())
}

/// Words a name is sorted past rather than under.
///
/// A library that files "The Beatles" under T is one nobody can find anything
/// in, so a leading article is skipped when two names are compared. Only the
/// English articles are handled, and only at the front of the string, which is
/// what a desktop player's user expects.
const LEADING_ARTICLES: [&str; 3] = ["the ", "a ", "an "];

/// The form of a name used for ordering: trimmed, lowercased, and with a
/// leading article removed.
///
/// Ordering only — matching still runs against the name as it is written, so a
/// search for "the beatles" keeps working.
#[must_use]
pub fn sort_name(value: &str) -> String {
    let lowered = value.trim().to_lowercase();

    for article in LEADING_ARTICLES {
        if let Some(rest) = lowered.strip_prefix(article) {
            // "A  Tribe" leaves a second space behind, and "The The" would sort
            // as nothing at all; keep at least the article when stripping it
            // would leave the name empty.
            let rest = rest.trim_start();
            if !rest.is_empty() {
                return rest.to_owned();
            }
        }
    }

    lowered
}

/// Order two optional names, with the missing value last.
///
/// This is [`compare_text`] with the library's own idea of how a name should
/// read in a list: case is ignored, and so is a leading article.
fn compare_names(left: Option<&str>, right: Option<&str>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => sort_name(left).cmp(&sort_name(right)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Order two optional strings, with the missing value last.
fn compare_text(left: Option<&str>, right: Option<&str>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.to_lowercase().cmp(&right.to_lowercase()),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Order two optional numbers, with the missing value last.
fn compare_number<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Order two optional durations, with the missing value last.
fn compare_duration(left: Option<Duration>, right: Option<Duration>) -> Ordering {
    compare_number(left, right)
}

/// The ordering of two tracks under `sort`.
///
/// Ties are broken by title and then by path so the list is stable across
/// frames, which stops rows from swapping places when two tracks share an
/// artist or an album.
#[must_use]
pub fn compare_tracks(left: &Track, right: &Track, sort: Sort) -> Ordering {
    let primary = match sort.key {
        SortKey::Title => compare_names(Some(&left.title), Some(&right.title)),
        SortKey::Artist => compare_names(left.display_artist(), right.display_artist()),
        SortKey::Album => compare_names(left.album.as_deref(), right.album.as_deref()),
        SortKey::Genre => compare_names(left.genre.as_deref(), right.genre.as_deref()),
        SortKey::Duration => compare_duration(left.duration, right.duration),
        SortKey::TrackNumber => compare_number(left.track_number, right.track_number),
        SortKey::Path => compare_text(
            Some(&left.path.to_string_lossy()),
            Some(&right.path.to_string_lossy()),
        ),
        SortKey::PlayCount => left.play_count.cmp(&right.play_count),
    };

    let ordered = primary
        .then_with(|| compare_names(Some(&left.title), Some(&right.title)))
        .then_with(|| {
            compare_text(
                Some(&left.path.to_string_lossy()),
                Some(&right.path.to_string_lossy()),
            )
        });

    match sort.direction {
        SortDirection::Ascending => ordered,
        SortDirection::Descending => ordered.reverse(),
    }
}

/// The tracks that match `query`, in `sort` order.
#[must_use]
pub fn filter_tracks<'a>(tracks: &'a [Track], query: &Query, sort: Sort) -> Vec<&'a Track> {
    let mut matched: Vec<&Track> = tracks
        .iter()
        .filter(|track| matches_track(track, query))
        .collect();

    matched.sort_by(|left, right| compare_tracks(left, right, sort));
    matched
}

/// The albums that match `query`, by name.
#[must_use]
pub fn filter_albums<'a>(albums: &'a [Album], query: &Query) -> Vec<&'a Album> {
    let mut matched: Vec<&Album> = albums
        .iter()
        .filter(|album| matches_album(album, query))
        .collect();

    matched.sort_by(|left, right| compare_names(Some(&left.name), Some(&right.name)));
    matched
}

/// The artists that match `query`, by name.
#[must_use]
pub fn filter_artists<'a>(artists: &'a [Artist], query: &Query) -> Vec<&'a Artist> {
    let mut matched: Vec<&Artist> = artists
        .iter()
        .filter(|artist| matches_artist(artist, query))
        .collect();

    matched.sort_by(|left, right| compare_names(Some(&left.name), Some(&right.name)));
    matched
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn track(title: &str, artist: Option<&str>, album: Option<&str>) -> Track {
        let mut track = Track::from_path(PathBuf::from(format!("/music/{title}.mp3")));
        track.title = title.to_owned();
        track.artist = artist.map(str::to_owned);
        track.album = album.map(str::to_owned);
        track
    }

    #[test]
    fn the_most_played_tracks_can_be_put_first_or_last() {
        let mut rarely = track("Olson", None, None);
        rarely.play_count = 2;
        let mut often = track("Xtal", None, None);
        often.play_count = 40;

        let descending = Sort::new(SortKey::PlayCount, SortDirection::Descending);
        assert_eq!(
            compare_tracks(&rarely, &often, descending),
            Ordering::Greater,
            "the track played fewer times comes second when sorting by plays, high to low"
        );

        let ascending = Sort::new(SortKey::PlayCount, SortDirection::Ascending);
        assert_eq!(compare_tracks(&rarely, &often, ascending), Ordering::Less);
    }

    #[test]
    fn every_column_heading_names_a_field_that_exists() {
        // A key added without a label would render an empty heading, and the
        // table would look like it had lost a column.
        for key in SortKey::ALL {
            assert!(!key.label().is_empty(), "{key:?} has no heading");
        }
    }

    #[test]
    fn an_empty_query_matches_every_track() {
        let query = Query::new("   ");
        assert!(query.is_empty());
        assert!(matches_track(&track("Roygbiv", None, None), &query));
    }

    #[test]
    fn matching_ignores_case_and_surrounding_space() {
        let query = Query::new("  ROYGBIV ");
        assert_eq!(query.as_str(), "roygbiv");
        assert!(matches_track(&track("Roygbiv", None, None), &query));
    }

    #[test]
    fn any_visible_field_can_match() {
        let track = track(
            "Roygbiv",
            Some("Boards of Canada"),
            Some("Music Has the Right"),
        );

        assert!(matches_track(&track, &Query::new("roygbiv")));
        assert!(matches_track(&track, &Query::new("canada")));
        assert!(matches_track(&track, &Query::new("right")));
        assert!(matches_track(&track, &Query::new("/music/")));
        assert!(!matches_track(&track, &Query::new("aphex")));
    }

    #[test]
    fn the_album_artist_is_searched_too() {
        let mut track = track("Untitled", None, Some("Compilation"));
        track.album_artist = Some("Various Artists".to_owned());

        assert!(matches_track(&track, &Query::new("various")));
    }

    #[test]
    fn genre_matches() {
        let mut track = track("Xtal", None, None);
        track.genre = Some("Ambient Techno".to_owned());

        assert!(matches_track(&track, &Query::new("techno")));
    }

    #[test]
    fn clicking_a_new_column_starts_it_ascending() {
        let sort = Sort::new(SortKey::Title, SortDirection::Descending);
        let clicked = sort.clicked(SortKey::Artist);

        assert_eq!(
            clicked,
            Sort::new(SortKey::Artist, SortDirection::Ascending)
        );
    }

    #[test]
    fn clicking_the_active_column_flips_the_direction() {
        let sort = Sort::new(SortKey::Title, SortDirection::Ascending);

        assert_eq!(
            sort.clicked(SortKey::Title).direction,
            SortDirection::Descending
        );
        assert_eq!(
            sort.clicked(SortKey::Title)
                .clicked(SortKey::Title)
                .direction,
            SortDirection::Ascending
        );
    }

    #[test]
    fn tracks_sort_by_title_in_both_directions() {
        let tracks = vec![
            track("Xtal", None, None),
            track("Roygbiv", None, None),
            track("An Ending", None, None),
        ];

        let ascending: Vec<&str> = filter_tracks(&tracks, &Query::default(), Sort::default())
            .into_iter()
            .map(|track| track.title.as_str())
            .collect();
        assert_eq!(ascending, vec!["An Ending", "Roygbiv", "Xtal"]);

        let descending = Sort::new(SortKey::Title, SortDirection::Descending);
        let titles: Vec<&str> = filter_tracks(&tracks, &Query::default(), descending)
            .into_iter()
            .map(|track| track.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Xtal", "Roygbiv", "An Ending"]);
    }

    #[test]
    fn tracks_without_the_sorted_field_come_last() {
        let mut with_artist = track("Zed", Some("Aphex Twin"), None);
        with_artist.artist = Some("Aphex Twin".to_owned());
        let without_artist = track("Alpha", None, None);

        let tracks = vec![without_artist, with_artist];
        let sort = Sort::new(SortKey::Artist, SortDirection::Ascending);

        let ordered: Vec<&str> = filter_tracks(&tracks, &Query::default(), sort)
            .into_iter()
            .map(|track| track.title.as_str())
            .collect();

        assert_eq!(
            ordered,
            vec!["Zed", "Alpha"],
            "a track with no artist sorts after one with an artist"
        );
    }

    #[test]
    fn the_same_track_set_always_sorts_the_same_way() {
        let tracks = vec![
            track("Zed", Some("Aphex Twin"), Some("Album")),
            track("Alpha", Some("Aphex Twin"), Some("Album")),
        ];
        let sort = Sort::new(SortKey::Artist, SortDirection::Ascending);

        let first: Vec<String> = filter_tracks(&tracks, &Query::default(), sort)
            .into_iter()
            .map(|track| track.title.clone())
            .collect();
        let second: Vec<String> = filter_tracks(&tracks, &Query::default(), sort)
            .into_iter()
            .map(|track| track.title.clone())
            .collect();

        assert_eq!(first, second);
        assert_eq!(first, vec!["Alpha", "Zed"], "ties break on the title");
    }

    #[test]
    fn a_leading_article_is_skipped_when_names_are_compared() {
        assert_eq!(sort_name("The Beatles"), "beatles");
        assert_eq!(sort_name("  A  Tribe Called Quest  "), "tribe called quest");
        assert_eq!(sort_name("An Ending"), "ending");
        assert_eq!(sort_name("The The"), "the", "something always remains");
        assert_eq!(sort_name("Aphex Twin"), "aphex twin", "only the first word");
    }

    #[test]
    fn the_beatles_file_under_b_rather_than_under_t() {
        let tracks = vec![
            track("Rain", Some("The Beatles"), None),
            track("Song 2", Some("Blur"), None),
        ];
        let sort = Sort::new(SortKey::Artist, SortDirection::Ascending);

        let artists: Vec<&str> = filter_tracks(&tracks, &Query::default(), sort)
            .into_iter()
            .filter_map(|track| track.display_artist())
            .collect();

        assert_eq!(artists, vec!["The Beatles", "Blur"]);
    }

    #[test]
    fn an_album_is_ordered_by_its_name_without_its_article() {
        let tracks = vec![
            track("Xtal", None, Some("The Campfire Headphase")),
            track("Roygbiv", None, Some("Blue Album")),
        ];
        let sort = Sort::new(SortKey::Album, SortDirection::Ascending);

        let albums: Vec<&str> = filter_tracks(&tracks, &Query::default(), sort)
            .into_iter()
            .filter_map(|track| track.album.as_deref())
            .collect();

        assert_eq!(albums, vec!["Blue Album", "The Campfire Headphase"]);
    }

    #[test]
    fn filtering_keeps_the_sort_order() {
        let tracks = vec![
            track("Roygbiv", Some("Boards of Canada"), None),
            track("Xtal", Some("Aphex Twin"), None),
            track("Rue The Whirl", Some("Boards of Canada"), None),
        ];

        let matched: Vec<&str> = filter_tracks(&tracks, &Query::new("canada"), Sort::default())
            .into_iter()
            .map(|track| track.title.as_str())
            .collect();

        assert_eq!(matched, vec!["Roygbiv", "Rue The Whirl"]);
    }

    #[test]
    fn albums_and_artists_are_filtered_by_name() {
        let albums = vec![
            Album {
                name: "Music Has the Right to Children".to_owned(),
                album_artist: Some("Boards of Canada".to_owned()),
                track_count: 17,
                duration: None,
            },
            Album {
                name: "Selected Ambient Works".to_owned(),
                album_artist: Some("Aphex Twin".to_owned()),
                track_count: 13,
                duration: None,
            },
        ];
        let artists = vec![
            Artist {
                name: "Boards of Canada".to_owned(),
                album_count: 1,
                track_count: 17,
            },
            Artist {
                name: "Aphex Twin".to_owned(),
                album_count: 1,
                track_count: 13,
            },
        ];

        assert_eq!(filter_albums(&albums, &Query::new("ambient")).len(), 1);
        assert_eq!(filter_albums(&albums, &Query::new("canada")).len(), 1);
        assert_eq!(filter_artists(&artists, &Query::new("aphex")).len(), 1);
        assert!(filter_albums(&albums, &Query::new("nothing")).is_empty());
    }
}
