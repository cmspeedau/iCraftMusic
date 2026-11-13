use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LibraryData {
    #[serde(default, alias = "tracks")]
    pub music: Vec<String>,
    #[serde(default)]
    pub movies: Vec<String>,
    #[serde(default)]
    pub tv_shows: Vec<String>,
    #[serde(default)]
    pub podcasts: Vec<String>,
    #[serde(default)]
    pub queue: Vec<String>,
    #[serde(default)]
    pub scan_roots: Vec<String>,
}

pub type Sections = (
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    std::collections::VecDeque<PathBuf>,
    Vec<PathBuf>,
);

impl LibraryData {
    pub fn from_sections(
        music: &[PathBuf],
        movies: &[PathBuf],
        tv_shows: &[PathBuf],
        podcasts: &[PathBuf],
        queue: &std::collections::VecDeque<PathBuf>,
        scan_roots: &[PathBuf],
    ) -> Self {
        Self {
            music: music
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            movies: movies
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            tv_shows: tv_shows
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            podcasts: podcasts
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            queue: queue
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            scan_roots: scan_roots
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
        }
    }

    pub fn to_sections(&self) -> Sections {
        (
            self.music.iter().map(PathBuf::from).collect(),
            self.movies.iter().map(PathBuf::from).collect(),
            self.tv_shows.iter().map(PathBuf::from).collect(),
            self.podcasts.iter().map(PathBuf::from).collect(),
            self.queue.iter().map(PathBuf::from).collect(),
            self.scan_roots.iter().map(PathBuf::from).collect(),
        )
    }
}

pub fn storage_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("iCraftMusic"))
}

pub fn storage_path() -> Option<PathBuf> {
    storage_dir().map(|dir| dir.join("library.json"))
}

pub fn load() -> Result<LibraryData, String> {
    let path = storage_path().ok_or_else(|| "Could not determine storage directory".to_string())?;
    if !path.exists() {
        return Ok(LibraryData::default());
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read library file: {error}"))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse library file: {error}"))
}

pub fn save(data: &LibraryData) -> Result<(), String> {
    let dir = storage_dir().ok_or_else(|| "Could not determine storage directory".to_string())?;
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create storage directory: {error}"))?;
    let path = dir.join("library.json");
    let contents = serde_json::to_string_pretty(data)
        .map_err(|error| format!("Could not serialize library: {error}"))?;
    fs::write(&path, contents).map_err(|error| format!("Could not write library file: {error}"))
}

pub fn backup_path() -> Option<PathBuf> {
    storage_dir().map(|dir| dir.join("library.backup.json"))
}

pub fn backup(data: &LibraryData) -> Result<PathBuf, String> {
    let path = backup_path().ok_or_else(|| "Could not determine backup directory".to_string())?;
    let dir = path
        .parent()
        .ok_or_else(|| "Invalid backup path".to_string())?;
    fs::create_dir_all(dir)
        .map_err(|error| format!("Could not create backup directory: {error}"))?;
    let contents = serde_json::to_string_pretty(data)
        .map_err(|error| format!("Could not serialize backup: {error}"))?;
    fs::write(&path, contents).map_err(|error| format!("Could not write backup file: {error}"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::LibraryData;
    use std::path::PathBuf;

    #[test]
    fn round_trips_sections() {
        let music = vec![PathBuf::from("/music/song.mp3")];
        let movies = vec![PathBuf::from("/movies/film.mkv")];
        let tv = vec![PathBuf::from("/tv/show.mkv")];
        let podcasts = vec![PathBuf::from("/podcasts/ep.mp3")];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(PathBuf::from("/music/song.mp3"));
        let data = LibraryData::from_sections(&music, &movies, &tv, &podcasts, &queue, &[]);
        let json = serde_json::to_string(&data).unwrap();
        let parsed: LibraryData = serde_json::from_str(&json).unwrap();
        let (m, mv, t, p, q, _sr) = parsed.to_sections();
        assert_eq!(m, music);
        assert_eq!(mv, movies);
        assert_eq!(t, tv);
        assert_eq!(p, podcasts);
        assert_eq!(q, queue);
    }

    #[test]
    fn empty_data_deserializes() {
        let data: LibraryData = serde_json::from_str("{}").unwrap();
        assert!(data.music.is_empty());
        assert!(data.movies.is_empty());
        assert!(data.tv_shows.is_empty());
        assert!(data.podcasts.is_empty());
        assert!(data.queue.is_empty());
    }

    #[test]
    fn legacy_tracks_alias_deserializes() {
        let data: LibraryData = serde_json::from_str(r#"{"tracks": ["/old/song.mp3"]}"#).unwrap();
        assert_eq!(data.music, vec!["/old/song.mp3"]);
    }

    #[test]
    fn missing_fields_default_to_empty() {
        let data: LibraryData = serde_json::from_str(r#"{"music": ["/a.mp3"]}"#).unwrap();
        assert_eq!(data.music, vec!["/a.mp3"]);
        assert!(data.movies.is_empty());
        assert!(data.queue.is_empty());
    }

    #[test]
    fn corrupt_json_fails_deserialization() {
        let result: Result<LibraryData, _> = serde_json::from_str("{not valid json");
        assert!(result.is_err());
    }
}
