//! Copying files the user chose into the media home.
//!
//! The add dialog offers two ways to bring music in: watch it where it is, or
//! copy it into the media home and watch that. This module does the second.
//! It is deliberately narrow: only audio this build can decode is copied,
//! because a media folder of unreadable extras is not a library, and the
//! original files are never touched — the copy is one-way and a failed copy
//! leaves the source exactly as it found it.
//!
//! The shape mirrors [`crate::library::scanner`]: a request goes in, a report
//! comes out, an [`std::sync::atomic::AtomicBool`] cancels it between files,
//! and a [`CopySession`] runs the whole thing on a worker thread so the window
//! keeps drawing while a large folder is on its way across the disk.
//!
//! Collisions are never destructive. A destination that already holds a file
//! of that name gets a numbered sibling (`Song 2.mp3`) rather than an
//! overwrite: guessing that two files of the same name are the same song is
//! how music gets lost.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread::{self, JoinHandle};

use crate::audio;
use crate::library::scanner::ScanError;

/// How often the observer hears from a running copy, in files.
const PROGRESS_INTERVAL: usize = 20;

/// What to copy, and where it is going.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRequest {
    /// The files and folders the user chose.
    pub paths: Vec<PathBuf>,
    /// The media home they are copied into. It is created if absent.
    pub home: PathBuf,
}

/// How far a running copy has got.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyProgress {
    /// Files written into the home so far.
    pub copied: usize,
    /// Files the whole copy is expected to write, from the walk that precedes
    /// it. `0` until that walk has run.
    pub total: usize,
    /// Files that could not be written so far. Live like the rest: the bar's
    /// "left out" count moves while the copy runs, it is not a verdict saved
    /// for the report at the end.
    pub failed: usize,
    /// The file being copied right now.
    pub current: Option<PathBuf>,
}

/// What a finished copy did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyReport {
    /// Files written into the home, including numbered siblings.
    pub copied: usize,
    /// Sources that were unreadable or vanished mid-copy.
    pub failed: usize,
    /// The individual failures, named.
    pub errors: Vec<ScanError>,
    /// Whether the user stopped the copy before it finished.
    pub cancelled: bool,
}

/// A file to copy, and the folder-relative name it gets under the home.
type Pending = (PathBuf, PathBuf);

/// What [`copy_with_cancel`] will do with one chosen path.
///
/// A folder contributes its whole tree under its own name, so the chosen
/// folders keep their shape inside the home; a loose file lands at the home's
/// top level, named as it is named where it lives.
fn plan(path: &Path) -> Vec<Pending> {
    let name = match path.file_name() {
        Some(name) => PathBuf::from(name),
        None => return Vec::new(),
    };

    if path.is_dir() {
        let files = crate::library::scanner::find_files(&[path.to_path_buf()]);
        return files
            .audio
            .into_iter()
            .map(|found| {
                let relative = found.strip_prefix(path).unwrap_or(&found).to_path_buf();
                (found, name.join(relative))
            })
            .collect();
    }

    if audio::is_supported(path) {
        vec![(path.to_path_buf(), name)]
    } else {
        Vec::new()
    }
}

/// Copy `request`'s files into the home, reporting each batch to `observer`.
#[must_use]
pub fn copy(request: &CopyRequest, mut observer: impl FnMut(&CopyProgress)) -> CopyReport {
    let cancel = Arc::new(AtomicBool::new(false));
    copy_with_cancel(request, &cancel, &mut observer)
}

/// The implementation both entry points share; `cancel` is checked per file.
#[must_use]
pub fn copy_with_cancel(
    request: &CopyRequest,
    cancel: &AtomicBool,
    observer: &mut dyn FnMut(&CopyProgress),
) -> CopyReport {
    let mut report = CopyReport::default();
    let mut progress = CopyProgress::default();

    let mut queue: Vec<Pending> = request.paths.iter().flat_map(|path| plan(path)).collect();
    queue.sort();
    queue.dedup();
    progress.total = queue.len();
    observer(&progress);

    if let Err(error) = std::fs::create_dir_all(&request.home) {
        report.failed = queue.len();
        for (source, _) in queue {
            report.errors.push(ScanError {
                path: source,
                message: format!("{} could not be created: {error}", request.home.display()),
            });
        }
        observer(&progress);
        return report;
    }

    for (source, relative) in queue {
        if cancel.load(Ordering::Relaxed) {
            report.cancelled = true;
            break;
        }

        progress.current = Some(source.clone());
        observer(&progress);

        let destination = unique_destination(&request.home.join(&relative));
        match copy_file(&source, &destination) {
            Ok(()) => {
                report.copied += 1;
                progress.copied += 1;
            }
            Err(error) => {
                report.failed += 1;
                progress.failed += 1;
                report.errors.push(ScanError {
                    path: source,
                    message: error.to_string(),
                });
            }
        }

        if report.copied != 0 && report.copied.is_multiple_of(PROGRESS_INTERVAL) {
            observer(&progress);
        }
    }

    progress.current = None;
    observer(&progress);

    tracing::info!(
        "copy finished: {} files into {} ({} failed, cancelled: {})",
        report.copied,
        request.home.display(),
        report.failed,
        report.cancelled
    );

    report
}

/// A destination for `wanted` that does not exist yet: `wanted`, or `Song 2`
/// and up beside it. Never overwrites, so two files that share a name but are
/// not the same recording both survive the copy.
fn unique_destination(wanted: &Path) -> PathBuf {
    if !wanted.exists() {
        return wanted.to_path_buf();
    }

    let directory = wanted.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = wanted
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("track");
    let extension = wanted.extension().and_then(|s| s.to_str());

    for suffix in 2.. {
        let name = match extension {
            Some(extension) => format!("{stem} {suffix}.{extension}"),
            None => format!("{stem} {suffix}"),
        };
        let candidate = directory.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    wanted.to_path_buf() // unreachable: the loop always finds a free name
}

/// Copy one file's bytes and nothing else: no metadata mirroring, because the
/// scan that follows reads tags and stamps from the copy itself.
fn copy_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(source, destination)?;
    Ok(())
}

/// A copy running on a worker thread.
///
/// Identical in shape to [`crate::library::scanner::ScanSession`]: progress
/// arrives on a channel for the interface to draw, and [`CopySession::join`]
/// returns the report once [`CopySession::is_finished`] has said so.
pub struct CopySession {
    handle: JoinHandle<CopyReport>,
    progress: Receiver<CopyProgress>,
    cancel: Arc<AtomicBool>,
}

impl CopySession {
    /// Queue another worker thread to carry out `request`.
    #[must_use]
    pub fn spawn(request: CopyRequest) -> Self {
        let (sender, progress) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);

        let handle = thread::spawn(move || {
            let mut sender = Some(sender);
            copy_with_cancel(&request, &worker_cancel, &mut |progress| {
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
    pub fn try_progress(&self) -> Option<CopyProgress> {
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

    /// Wait for the copy to finish and return what it did.
    ///
    /// # Panics
    ///
    /// Panics if the worker thread panicked; that can only happen if a bug
    /// bypasses the error handling inside the copy.
    #[must_use]
    pub fn join(self) -> CopyReport {
        match self.handle.join() {
            Ok(report) => report,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}

impl std::fmt::Debug for CopySession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CopySession")
            .field("finished", &self.is_finished())
            .field("cancelled", &self.cancel.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

/// Forward a progress snapshot, ignoring a receiver that has gone away.
fn forward(sender: &mut Option<std::sync::mpsc::Sender<CopyProgress>>, progress: &CopyProgress) {
    if let Some(sender) = sender {
        let _ = sender.send(progress.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-importer-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a scratch directory");
        directory
    }

    fn write(path: &Path, bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(path, bytes).expect("write");
    }

    #[test]
    fn a_loose_file_lands_at_the_home_s_top_level() {
        let source = scratch("file-src");
        let home = scratch("file-home");
        write(&source.join("song.mp3"), b"abc");

        let report = copy(
            &CopyRequest {
                paths: vec![source.join("song.mp3")],
                home: home.clone(),
            },
            |_| {},
        );

        assert_eq!(report.copied, 1, "{report:?}");
        assert!(home.join("song.mp3").is_file(), "the copy is in the home");
        assert!(
            source.join("song.mp3").is_file(),
            "the original is untouched"
        );
    }

    #[test]
    fn a_folder_keeps_its_shape_under_its_own_name() {
        let source = scratch("tree-src");
        let home = scratch("tree-home");
        write(&source.join("Album").join("one.mp3"), b"1");
        write(&source.join("Album").join("two.flac"), b"22");
        write(&source.join("Album").join("cover.jpg"), b"not audio");

        let report = copy(
            &CopyRequest {
                paths: vec![source.clone()],
                home: home.clone(),
            },
            |_| {},
        );

        assert_eq!(report.copied, 2, "only decodable files travel: {report:?}");
        let folder = source.file_name().expect("a named scratch folder");
        assert!(home.join(folder).join("Album").join("one.mp3").is_file());
        assert!(home.join(folder).join("Album").join("two.flac").is_file());
        assert!(
            !home.join(folder).join("Album").join("cover.jpg").exists(),
            "the cover image is not copied"
        );
    }

    #[test]
    fn a_name_clash_goes_beside_the_file_it_clashes_with() {
        let source = scratch("clash-src");
        let home = scratch("clash-home");
        write(&source.join("song.mp3"), b"new song");
        write(&home.join("song.mp3"), b"a different song already here");

        let report = copy(
            &CopyRequest {
                paths: vec![source.join("song.mp3")],
                home: home.clone(),
            },
            |_| {},
        );

        assert_eq!(report.copied, 1);
        assert_eq!(
            std::fs::read(home.join("song.mp3")).expect("the old file"),
            b"a different song already here",
            "the file already in the home was not overwritten"
        );
        assert_eq!(
            std::fs::read(home.join("song 2.mp3")).expect("the sibling"),
            b"new song",
            "the copy went beside it"
        );
    }

    #[test]
    fn a_second_clash_numbered_upwards() {
        let source = scratch("clash2-src");
        let home = scratch("clash2-home");
        write(&source.join("song.mp3"), b"x");
        write(&home.join("song.mp3"), b"one");
        write(&home.join("song 2.mp3"), b"two");

        let report = copy(
            &CopyRequest {
                paths: vec![source.join("song.mp3")],
                home: home.clone(),
            },
            |_| {},
        );

        assert_eq!(report.copied, 1);
        assert!(home.join("song 3.mp3").is_file(), "{report:?}");
    }

    #[test]
    fn a_cancelled_copy_stops_and_says_so() {
        let source = scratch("cancel-src");
        let home = scratch("cancel-home");
        for index in 0..50 {
            write(&source.join(format!("song{index}.mp3")), b"audio bytes");
        }

        let cancel = Arc::new(AtomicBool::new(true));
        let report = copy_with_cancel(
            &CopyRequest {
                paths: vec![source],
                home,
            },
            &cancel,
            &mut |_| {},
        );

        assert!(report.cancelled, "the report admits it");
        assert_eq!(report.copied, 0, "not one file was copied");
    }

    #[test]
    fn a_source_that_vanished_after_it_was_chosen_is_named_as_a_failure() {
        let home = scratch("vanished-home");
        let mut updates = Vec::new();

        let report = copy(
            &CopyRequest {
                paths: vec![std::env::temp_dir().join("musiccraft-no-such-file.mp3")],
                home: home.clone(),
            },
            |progress| updates.push(progress.clone()),
        );

        assert_eq!(report.copied, 0);
        assert_eq!(
            report.failed, 1,
            "a file that was chosen and cannot be found is a failure, not a silence"
        );
        assert_eq!(report.errors.len(), 1, "and it is named");
        assert!(
            updates.last().is_some_and(|progress| progress.failed == 1),
            "the bar's live count failed what the report admits: {updates:?}"
        );
    }

    #[test]
    fn the_progress_told_the_truth_about_the_total() {
        let source = scratch("total-src");
        let home = scratch("total-home");
        write(&source.join("one.mp3"), b"1");
        write(&source.join("two.mp3"), b"2");

        let mut totals = Vec::new();
        let mut last = CopyProgress::default();
        let report = copy(
            &CopyRequest {
                paths: vec![source],
                home,
            },
            |progress: &CopyProgress| {
                totals.push(progress.total);
                last = progress.clone();
            },
        );

        assert!(
            totals.contains(&2),
            "the walk announced two files: {totals:?}"
        );
        assert_eq!(last.copied, report.copied);
    }

    #[test]
    fn a_session_carries_the_same_work_on_a_thread() {
        let source = scratch("session-src");
        let home = scratch("session-home");
        write(&source.join("song.mp3"), b"abc");

        let session = CopySession::spawn(CopyRequest {
            paths: vec![source.join("song.mp3")],
            home: home.clone(),
        });

        let report = session.join();

        assert_eq!(report.copied, 1, "{report:?}");
        assert!(home.join("song.mp3").is_file());
    }
}
