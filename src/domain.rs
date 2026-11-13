use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LibrarySection {
    #[default]
    Music,
    Movies,
    TvShows,
    Podcasts,
}

impl LibrarySection {
    pub const ALL: [Self; 4] = [Self::Music, Self::Movies, Self::TvShows, Self::Podcasts];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Music => "Music",
            Self::Movies => "Movies",
            Self::TvShows => "TV Shows",
            Self::Podcasts => "Podcasts",
        }
    }

    pub const fn empty_message(self) -> &'static str {
        match self {
            Self::Music => "Your music library is empty.",
            Self::Movies => "Your movie library is empty.",
            Self::TvShows => "Your TV shows library is empty.",
            Self::Podcasts => "Your podcast library is empty.",
        }
    }
}

pub fn format_time(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub fn filter_tracks(tracks: &[PathBuf], query: &str) -> Vec<(usize, PathBuf)> {
    let query = query.trim().to_lowercase();
    tracks
        .iter()
        .enumerate()
        .filter(|(_, path)| {
            query.is_empty()
                || path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.to_lowercase().contains(&query))
        })
        .map(|(index, path)| (index, path.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn all_primary_library_sections_have_distinct_labels() {
        let labels = LibrarySection::ALL.map(LibrarySection::label);
        let mut unique = labels.to_vec();
        unique.sort_unstable();
        unique.dedup();

        assert_eq!(unique.len(), LibrarySection::ALL.len());
        assert_eq!(labels, ["Music", "Movies", "TV Shows", "Podcasts"]);
    }

    #[test]
    fn every_section_has_a_specific_empty_state() {
        for section in LibrarySection::ALL {
            assert!(section.empty_message().contains("library is empty"));
        }
    }

    #[test]
    fn formats_player_time_as_minutes_and_seconds() {
        assert_eq!(format_time(Duration::from_secs(0)), "0:00");
        assert_eq!(format_time(Duration::from_secs(125)), "2:05");
    }

    #[test]
    fn filter_tracks_returns_all_when_query_is_empty() {
        let tracks = vec![
            PathBuf::from("/music/a.mp3"),
            PathBuf::from("/music/b.flac"),
        ];
        let result = filter_tracks(&tracks, "");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_tracks_matches_case_insensitively() {
        let tracks = vec![
            PathBuf::from("/music/Song One.mp3"),
            PathBuf::from("/music/Another Track.flac"),
        ];
        let result = filter_tracks(&tracks, "song");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, PathBuf::from("/music/Song One.mp3"));
    }

    #[test]
    fn filter_tracks_returns_empty_when_no_match() {
        let tracks = vec![PathBuf::from("/music/song.mp3")];
        let result = filter_tracks(&tracks, "zzz");
        assert!(result.is_empty());
    }

    #[test]
    fn filter_tracks_trims_query() {
        let tracks = vec![PathBuf::from("/music/song.mp3")];
        let result = filter_tracks(&tracks, "  song  ");
        assert_eq!(result.len(), 1);
    }
}
