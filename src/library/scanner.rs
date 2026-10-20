//! Walking the user's folders and turning files into [`Track`]s.
//!
//! A scan is deliberately a plain function that reports progress through a
//! callback, plus a small wrapper that runs it on a worker thread. The
//! interface uses the threaded form so a large folder never blocks the frame
//! loop, while tests use the synchronous form because it needs no timing.
//!
//! Failures are collected, not propagated: one unreadable file must not abort a
//! scan of ten thousand good ones. A file whose tags cannot be parsed is still
//! indexed, using its file name as the title.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use walkdir::WalkDir;

use crate::audio;
use crate::error::Result;
use crate::library::artwork;
use crate::library::metadata;
use crate::library::track::Track;

/// How many files a scan has looked at so far.
///
/// A scan reports progress every [`PROGRESS_INTERVAL`] files and once at the
/// end, so the interface can show a live count without the worker drowning it
/// in messages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanProgress {
    /// Files found so far that the engine can decode.
    pub discovered: usize,
    /// Files whose tags have been read (or failed).
    pub processed: usize,
    /// Files branded as broken.
    pub failed: usize,
    /// Files the walk expects to read, or `0` until the walk has finished
    /// counting. It is what lets the progress bar proportion itself; without
    /// it the bar slides and promises no number.
    pub total: usize,
    /// Music found in formats this build cannot decode. The walk knows them
    /// before the first file is read, so what is left out of the library can
    /// be stated from the first frame rather than guessed at the last.
    pub unreadable: usize,
    /// The file currently being read, when the scan is mid-file.
    pub current: Option<PathBuf>,
}

/// A file that could not be read during a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanError {
    /// The file that failed.
    pub path: PathBuf,
    /// A human-readable reason, taken from the underlying error.
    pub message: String,
}

/// Everything a completed scan produced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// Number of decodable files found.
    pub discovered: usize,
    /// Number of files that produced a [`Track`].
    pub scanned: usize,
    /// Number of files that could not be parsed and were indexed by file name.
    pub degraded: usize,
    /// Number of files that could not be read at all (they are not indexed).
    pub failed: usize,
    /// Files that are music in a format this build cannot decode.
    ///
    /// A scan never indexes these — it cannot read them — but it does report
    /// them, because a user who has just imported a folder deserves to be told
    /// that part of their collection is in a format the player does not
    /// understand rather than being left to wonder where it went.
    pub unreadable: Vec<PathBuf>,
    /// The tracks, in the order they were found.
    pub tracks: Vec<Track>,
    /// One entry per file that could not be read.
    pub errors: Vec<ScanError>,
    /// Whether the scan stopped early because it was cancelled.
    pub cancelled: bool,
}

impl ScanReport {
    /// Whether any track was found at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// How many files were passed over because of their format.
    #[must_use]
    pub fn unreadable_count(&self) -> usize {
        self.unreadable.len()
    }

    /// The distinct formats that were passed over, e.g. `["ape", "wma"]`.
    ///
    /// Names rather than paths: what the user needs to know is *which* of their
    /// formats is not supported, and the extensions say that in one line where a
    /// list of file names would not.
    #[must_use]
    pub fn unreadable_formats(&self) -> Vec<String> {
        unreadable_formats(&self.unreadable)
    }
}

/// The distinct, sorted file extensions of a list of passed-over files.
#[must_use]
pub fn unreadable_formats(paths: &[PathBuf]) -> Vec<String> {
    let mut formats: Vec<String> = paths
        .iter()
        .filter_map(|path| path.extension())
        .filter_map(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|extension| !extension.is_empty())
        .collect();

    formats.sort();
    formats.dedup();
    formats
}

/// How often a long scan reports progress, in files.
pub const PROGRESS_INTERVAL: usize = 64;

/// What to scan and where to put the cover art it finds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    /// Absolute directories to walk.
    pub folders: Vec<PathBuf>,
    /// Directory the extracted cover art is written to.
    pub artwork_directory: PathBuf,
    /// Whether to extract and cache embedded cover art at all.
    pub cache_artwork: bool,
}

/// Walk `folders` and read every decodable file.
///
/// `observer` is called with a fresh [`ScanProgress`] roughly every
/// [`PROGRESS_INTERVAL`] files and once when the scan finishes.
#[must_use]
pub fn scan(request: &ScanRequest, mut observer: impl FnMut(&ScanProgress)) -> ScanReport {
    let cancel = Arc::new(AtomicBool::new(false));
    scan_with_cancel(request, &cancel, &mut observer)
}

/// Walk `folders`, stopping early when `cancel` is set.
///
/// This is the implementation both the synchronous and threaded entry points
/// share; `cancel` is checked per file, so a cancelled scan stops promptly
/// without leaving a partially written track behind.
#[must_use]
pub fn scan_with_cancel(
    request: &ScanRequest,
    cancel: &AtomicBool,
    observer: &mut dyn FnMut(&ScanProgress),
) -> ScanReport {
    let mut report = ScanReport::default();
    let mut progress = ScanProgress::default();

    let found = find_files(&request.folders);
    report.unreadable = found.unreadable;

    // The walk has ended before the reading begins, so the totals the bar is
    // drawn from are real before a single file is opened — and the first
    // observer call carries them, not an empty struct.
    progress.total = found.audio.len();
    progress.unreadable = report.unreadable.len();
    observer(&progress);

    for path in found.audio {
        if cancel.load(Ordering::Relaxed) {
            report.cancelled = true;
            break;
        }

        report.discovered += 1;
        progress.discovered += 1;
        progress.current = Some(path.clone());

        match read_track(&path, request) {
            Ok((track, degraded)) => {
                report.tracks.push(track);
                report.scanned += 1;
                if degraded {
                    report.degraded += 1;
                }
            }
            Err(error) => {
                report.failed += 1;
                progress.failed += 1;
                report.errors.push(ScanError {
                    path,
                    message: error.to_string(),
                });
            }
        }

        progress.processed += 1;

        if report.discovered.is_multiple_of(PROGRESS_INTERVAL) {
            observer(&progress);
        }
    }

    progress.current = None;
    observer(&progress);

    tracing::info!(
        "scan finished: {} tracks from {} folders ({} degraded, {} failed, {} in other formats, \
         cancelled: {})",
        report.scanned,
        request.folders.len(),
        report.degraded,
        report.failed,
        report.unreadable.len(),
        report.cancelled
    );

    report
}

/// Read one file, returning the track and whether the tags were unusable.
///
/// A file with broken tags is not a failure: the track is indexed from its
/// name so the user can still see and play it, and the reason is only logged.
fn read_track(path: &Path, request: &ScanRequest) -> Result<(Track, bool)> {
    let mut track = Track::from_path(path);

    if let Ok(metadata) = std::fs::metadata(path) {
        track.size_bytes = Some(metadata.len());
        track.modified_at = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX));
    }

    let tags = match metadata::read(path) {
        Ok(tags) => tags,
        Err(error) => {
            tracing::warn!("{} could not be read as audio: {error}", path.display());
            return Ok((track, true));
        }
    };

    if let Some(title) = tags.title {
        track.title = title;
    }
    track.artist = tags.artist;
    track.album = tags.album;
    track.album_artist = tags.album_artist;
    track.genre = tags.genre;
    track.track_number = tags.track_number;
    track.disc_number = tags.disc_number;
    track.duration = tags.duration;

    if request.cache_artwork
        && let Some(image) = tags.artwork.as_deref()
        && let Ok(cached) = artwork::store(&request.artwork_directory, path, image)
    {
        track.artwork_path = Some(cached);
    }

    Ok((track, false))
}

/// What a walk of the user's folders turned up.
///
/// The two lists are kept apart because the interface treats them differently:
/// the first is what the library is built from, and the second is what the user
/// is told about. Anything else in the folders — playlists, cover images, text —
/// appears in neither.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoundFiles {
    /// Files the engine can decode.
    pub audio: Vec<PathBuf>,
    /// Files in a music format this build cannot decode.
    pub unreadable: Vec<PathBuf>,
}

/// Classify every file under `folders`, sorted for a stable order.
///
/// Hidden directories and unreadable ones are skipped silently: a music folder
/// with a broken `.thumbnails` directory is normal, not an error worth
/// interrupting a scan for.
#[must_use]
pub fn find_files(folders: &[PathBuf]) -> FoundFiles {
    let mut found = FoundFiles::default();

    for folder in folders {
        let walker = WalkDir::new(folder)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| !is_hidden(entry.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.into_path();
            if audio::is_supported(&path) {
                found.audio.push(path);
            } else if audio::is_unreadable(&path) {
                found.unreadable.push(path);
            }
        }
    }

    found.audio.sort();
    found.audio.dedup();
    found.unreadable.sort();
    found.unreadable.dedup();

    found
}

/// Find every decodable file under `folders`, sorted for a stable order.
#[must_use]
pub fn collect_audio_files(folders: &[PathBuf]) -> Vec<PathBuf> {
    find_files(folders).audio
}

/// Whether an entry is hidden, judged by its file name.
///
/// Only the last component is inspected, so a user whose music lives under a
/// dotted parent directory (say `~/.local/share/music`) can still scan it.
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.') && name != "." && name != "..")
}

/// A scan running on a worker thread.
///
/// Progress arrives on a channel so the interface can keep drawing; the final
/// [`ScanReport`] is available from [`ScanSession::join`] once the thread
/// finishes.
pub struct ScanSession {
    handle: JoinHandle<ScanReport>,
    progress: Receiver<ScanProgress>,
    cancel: Arc<AtomicBool>,
}

impl ScanSession {
    /// Queue another worker thread to scan `request`.
    #[must_use]
    pub fn spawn(request: ScanRequest) -> Self {
        let (sender, progress) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);

        let handle = thread::spawn(move || {
            let mut sender = Some(sender);
            scan_with_cancel(&request, &worker_cancel, &mut |progress| {
                forward(&mut sender, progress);
            })
        });

        Self {
            handle,
            progress,
            cancel,
        }
    }

    /// Take the most recent progress snapshot, if the worker has sent one.
    #[must_use]
    pub fn try_progress(&self) -> Option<ScanProgress> {
        let mut latest = None;
        while let Ok(progress) = self.progress.try_recv() {
            latest = Some(progress);
        }
        latest
    }

    /// Ask the worker to stop after the file it is on.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Whether the worker has finished, without waiting for it.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// Wait for the scan to finish and return what it found.
    ///
    /// # Panics
    ///
    /// Panics if the worker thread panicked; that can only happen if a bug
    /// bypasses the error handling inside the scan.
    #[must_use]
    pub fn join(self) -> ScanReport {
        match self.handle.join() {
            Ok(report) => report,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}

impl std::fmt::Debug for ScanSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScanSession")
            .field("finished", &self.is_finished())
            .field("cancelled", &self.cancel.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

/// Forward a progress snapshot, ignoring a receiver that has gone away.
fn forward(sender: &mut Option<Sender<ScanProgress>>, progress: &ScanProgress) {
    if let Some(channel) = sender.as_ref()
        && channel.send(progress.clone()).is_err()
    {
        *sender = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A fresh scratch directory containing the given files.
    fn scratch(name: &str, files: &[&str]) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "musiccraft-scan-{}-{}-{:?}",
            std::process::id(),
            name,
            std::thread::current().id()
        ));
        fs::remove_dir_all(&directory).ok();

        for file in files {
            let path = directory.join(file);
            fs::create_dir_all(path.parent().expect("parent")).expect("scratch directory");
            fs::write(&path, b"").expect("scratch file");
        }

        directory
    }

    fn request(folders: &[&Path]) -> ScanRequest {
        ScanRequest {
            folders: folders.iter().map(|path| path.to_path_buf()).collect(),
            artwork_directory: std::env::temp_dir().join("musiccraft-artwork"),
            cache_artwork: false,
        }
    }

    #[test]
    fn only_decodable_files_are_collected() {
        let library = scratch("formats", &["a.mp3", "b.flac", "notes.txt", "cover.jpg"]);

        let found = collect_audio_files(std::slice::from_ref(&library));
        let names: Vec<String> = found
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();

        assert_eq!(names, vec!["a.mp3", "b.flac"]);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn subdirectories_are_walked() {
        let library = scratch("nested", &["one/a.mp3", "one/two/three/b.flac"]);

        assert_eq!(collect_audio_files(std::slice::from_ref(&library)).len(), 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn hidden_directories_are_skipped() {
        let library = scratch("hidden", &["a.mp3", ".gitignore/b.mp3"]);

        assert_eq!(collect_audio_files(std::slice::from_ref(&library)).len(), 1);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn overlapping_folders_do_not_duplicate_tracks() {
        let library = scratch("overlap", &["a.mp3", "one/b.mp3"]);

        let found = collect_audio_files(&[library.clone(), library.join("one")]);
        assert_eq!(found.len(), 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn a_lone_file_named_as_an_entry_is_found_by_itself() {
        // The add dialog's "Add" button indexes a chosen file where it lives,
        // so the file's own path arrives as a watch entry. A walk of a file
        // yields the file, which is what makes that work.
        let library = scratch("lone-file", &["song.mp3", "notes.txt"]);

        let found = collect_audio_files(std::slice::from_ref(&library.join("song.mp3")));

        assert_eq!(found, vec![library.join("song.mp3")]);
        assert!(
            collect_audio_files(std::slice::from_ref(&library.join("notes.txt"))).is_empty(),
            "a named non-audio file indexes nothing"
        );

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn music_in_a_format_the_engine_cannot_read_is_reported_by_its_extension() {
        let library = scratch(
            "formats-unreadable",
            &["a.mp3", "b.wma", "c.APE", "notes.txt", "cover.png"],
        );

        let report = scan(&request(&[&library]), |_| {});

        assert_eq!(report.scanned, 1, "only the mp3 can be read");
        assert_eq!(report.unreadable_count(), 2);
        assert_eq!(
            report.unreadable_formats(),
            vec!["ape".to_owned(), "wma".to_owned()],
            "the report names the formats, not the files"
        );
        assert!(
            !report
                .unreadable
                .iter()
                .any(|path| path.extension().is_some_and(|e| e == "txt")),
            "a text file is not an unsupported music format"
        );

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn the_run_before_a_walk_keeps_the_two_lists_apart() {
        let library = scratch("found", &["one/a.mp3", "two/b.wv", "c.flac"]);

        let found = find_files(std::slice::from_ref(&library));

        assert_eq!(found.audio.len(), 2);
        assert_eq!(found.unreadable.len(), 1);
        assert_eq!(collect_audio_files(std::slice::from_ref(&library)).len(), 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn a_folder_of_unreadable_music_still_reports_nothing_indexed() {
        let library = scratch("only-unreadable", &["a.wma", "b.ape"]);

        let report = scan(&request(&[&library]), |_| {});

        assert!(report.is_empty());
        assert_eq!(report.scanned, 0);
        assert_eq!(report.unreadable_count(), 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn a_scan_of_an_empty_folder_reports_nothing() {
        let empty = scratch("empty", &[]);

        let report = scan(&request(&[&empty]), |_| {});

        assert!(report.is_empty());
        assert_eq!(report.discovered, 0);
        assert!(!report.cancelled);
        assert!(report.errors.is_empty());

        fs::remove_dir_all(&empty).ok();
    }

    #[test]
    fn a_file_with_broken_tags_is_indexed_by_name() {
        let library = scratch("broken", &["Roygbiv.mp3"]);

        let report = scan(&request(&[&library]), |_| {});

        assert_eq!(report.scanned, 1);
        assert_eq!(report.degraded, 1);
        assert_eq!(report.tracks[0].title, "Roygbiv");
        assert_eq!(report.tracks[0].size_bytes, Some(0));
        assert!(report.tracks[0].modified_at.is_some());

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn progress_is_reported_at_least_once_per_scan() {
        let library = scratch("progress", &["a.mp3", "b.mp3"]);
        let mut updates = Vec::new();

        let report = scan(&request(&[&library]), |progress| {
            updates.push(progress.clone());
        });

        assert_eq!(report.discovered, 2);
        assert!(!updates.is_empty());
        assert_eq!(updates.last().unwrap().current, None);
        assert_eq!(updates.last().unwrap().processed, 2);

        fs::remove_dir_all(&library).ok();
    }

    /// The walk finishes counting before the reading begins, and the first
    /// report carries what it counted: the bar can proportion itself and name
    /// the formats it cannot read from the very first frame, rather than
    /// only once some files have already been read.
    #[test]
    fn the_walk_reports_its_totals_before_the_first_file_is_read() {
        let odd = format!("m.{}", crate::audio::UNREADABLE_EXTENSIONS[0]);
        let library = scratch("totals", &["a.mp3", "b.mp3", &odd]);
        let mut updates = Vec::new();

        let report = scan(&request(&[&library]), |progress| {
            updates.push(progress.clone());
        });

        assert_eq!(report.discovered, 2);
        assert_eq!(report.unreadable_count(), 1);

        let first = updates
            .first()
            .expect("the walk reports before the reading");
        assert_eq!(first.total, 2, "two files the engine can read");
        assert_eq!(first.unreadable, 1, "and one format it cannot");
        assert_eq!(first.processed, 0, "the totals arrive ahead of the reads");

        let last = updates.last().expect("the scan reports at the end too");
        assert_eq!(last.total, 2);
        assert_eq!(last.unreadable, 1);
        assert_eq!(last.processed, 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn folders_that_do_not_exist_are_ignored() {
        let missing = std::env::temp_dir().join("musiccraft-no-such-folder");

        let report = scan(&request(&[&missing]), |_| {});

        assert!(report.is_empty());
        assert_eq!(report.discovered, 0);
    }

    #[test]
    fn a_cancelled_scan_stops_early() {
        let library = scratch("cancel", &["a.mp3", "b.mp3", "c.mp3"]);
        let cancel = AtomicBool::new(true);

        let report = scan_with_cancel(&request(&[&library]), &cancel, &mut |_| {});

        assert!(report.cancelled);
        assert!(report.scanned < 3);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn a_background_scan_returns_its_report() {
        let library = scratch("background", &["a.mp3", "b.flac"]);

        let session = ScanSession::spawn(request(&[&library]));
        let report = session.join();

        assert_eq!(report.scanned, 2);
        assert_eq!(report.tracks.len(), 2);

        fs::remove_dir_all(&library).ok();
    }

    #[test]
    fn a_background_scan_can_be_cancelled() {
        let library = scratch("background-cancel", &["a.mp3", "b.flac"]);

        let session = ScanSession::spawn(request(&[&library]));
        session.cancel();
        let report = session.join();

        assert!(report.cancelled || report.scanned == 2);

        fs::remove_dir_all(&library).ok();
    }
}
