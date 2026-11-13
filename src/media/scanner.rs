use std::fs;
use std::path::{Path, PathBuf};

/// Audio/video extensions that iCraftMusic can import.
/// Video extensions are included for future use; only audio is currently playable.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "ogg", "oga", "m4a", "mp4", "aac", "opus", "wma", "mkv", "avi", "mov",
    "webm", "m4v",
];

/// Returns true if the path has a supported media extension (case-insensitive).
pub fn is_supported_media(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub scanning: bool,
    pub files_found: usize,
    pub errors: Vec<String>,
    pub result_files: Option<Vec<PathBuf>>,
}

/// Recursively scan a folder for supported media files.
/// Unreadable entries are skipped. Returns an error only if the root cannot be read.
pub fn scan_folder(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Err(format!("Path does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root.display()));
    }

    let mut results = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            // Skip hidden files/directories
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'))
            {
                continue;
            }

            if path.is_dir() {
                // Don't follow symlinks to avoid escaping the root
                if path.is_symlink() {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() && is_supported_media(&path) {
                results.push(path);
            }
        }
    }

    Ok(results)
}

/// Scan multiple roots with progress reporting.
/// Runs synchronously but updates the shared progress counter.
/// Returns all found files and collects errors.
pub fn scan_folders_with_progress(
    roots: &[PathBuf],
    progress: Arc<Mutex<ScanProgress>>,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut all_files = Vec::new();
    let mut all_errors = Vec::new();

    for root in roots {
        if !root.exists() {
            all_errors.push(format!("Path does not exist: {}", root.display()));
            continue;
        }
        if !root.is_dir() {
            all_errors.push(format!("Not a directory: {}", root.display()));
            continue;
        }

        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| name.starts_with('.'))
                        {
                            continue;
                        }
                        if path.is_dir() {
                            if path.is_symlink() {
                                continue;
                            }
                            stack.push(path);
                        } else if path.is_file() && is_supported_media(&path) {
                            all_files.push(path);
                            if let Ok(mut p) = progress.lock() {
                                p.files_found = all_files.len();
                            }
                        }
                    }
                }
                Err(error) => {
                    all_errors.push(format!("Could not read {}: {error}", dir.display()));
                }
            }
        }
    }

    if let Ok(mut p) = progress.lock() {
        p.scanning = false;
        p.errors = all_errors.clone();
        p.result_files = Some(all_files.clone());
    }

    (all_files, all_errors)
}

/// Given a list of dropped paths (files or folders), expand folders and filter
/// for supported media, returning a flat list.
pub fn collect_dropped(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut results = Vec::new();
    for path in paths {
        if path.is_dir() {
            if let Ok(scanned) = scan_folder(path) {
                results.extend(scanned);
            }
        } else if path.is_file() && is_supported_media(path) {
            results.push(path.clone());
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn is_supported_media_recognizes_audio_extensions() {
        assert!(is_supported_media(Path::new("song.mp3")));
        assert!(is_supported_media(Path::new("track.FLAC")));
        assert!(is_supported_media(Path::new("audio.WAV")));
        assert!(is_supported_media(Path::new("voice.OGG")));
        assert!(is_supported_media(Path::new("clip.m4a")));
    }

    #[test]
    fn is_supported_media_rejects_non_media() {
        assert!(!is_supported_media(Path::new("readme.txt")));
        assert!(!is_supported_media(Path::new("image.png")));
        assert!(!is_supported_media(Path::new("no_extension")));
        assert!(!is_supported_media(Path::new("")));
    }

    #[test]
    fn scan_folder_finds_nested_media() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create nested structure
        fs::create_dir_all(root.join("album1")).unwrap();
        fs::create_dir_all(root.join("album2/sub")).unwrap();
        fs::write(root.join("top.mp3"), b"").unwrap();
        fs::write(root.join("album1/track1.flac"), b"").unwrap();
        fs::write(root.join("album2/sub/track2.wav"), b"").unwrap();
        fs::write(root.join("album1/readme.txt"), b"").unwrap();

        let results = scan_folder(root).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|p| p.ends_with("top.mp3")));
        assert!(results.iter().any(|p| p.ends_with("track1.flac")));
        let has_track2 = results.iter().any(|p| p.ends_with("track2.wav"));
        assert!(has_track2);
    }

    #[test]
    fn scan_folder_skips_hidden_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        fs::write(root.join("visible.mp3"), b"").unwrap();
        fs::write(root.join(".hidden.mp3"), b"").unwrap();
        fs::create_dir_all(root.join(".hidden_dir")).unwrap();
        fs::write(root.join(".hidden_dir/secret.mp3"), b"").unwrap();

        let results = scan_folder(root).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("visible.mp3"));
    }

    #[test]
    fn scan_folder_returns_error_for_missing_root() {
        let result = scan_folder(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }

    #[test]
    fn scan_folder_returns_error_for_file_path() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not_a_dir.mp3");
        std::fs::write(&file, b"").unwrap();
        let result = scan_folder(&file);
        assert!(result.is_err());
    }

    #[test]
    fn collect_dropped_expands_folders_and_filters_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/nested.mp3"), b"").unwrap();
        fs::write(root.join("standalone.flac"), b"").unwrap();
        fs::write(root.join("not_media.txt"), b"").unwrap();

        let dropped = vec![
            root.join("sub"),
            root.join("standalone.flac"),
            root.join("not_media.txt"),
        ];
        let results = collect_dropped(&dropped);
        assert_eq!(results.len(), 2);
    }
}
