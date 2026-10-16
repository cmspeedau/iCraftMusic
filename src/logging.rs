//! Process-wide logging setup built on [`tracing`].
//!
//! Every log line goes to standard error — where a terminal or the system
//! journal picks it up — and, when the platform provides a data directory, to a
//! file under it as well. The file is what makes a failure that happened in a
//! session the user has already closed still diagnosable: the window has no
//! console, and the Settings screen names the file so it can be found.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tracing_subscriber::EnvFilter;

/// Environment variable consulted for log filtering.
const ENV_FILTER: &str = "RUST_LOG";

/// Default filter used when `RUST_LOG` is unset.
const DEFAULT_FILTER: &str = "musiccraft=info,warn";

/// Directory the log file lives in, under the data directory.
pub const LOG_DIRECTORY: &str = "logs";

/// Name of the log file inside [`LOG_DIRECTORY`].
pub const LOG_FILE: &str = "icraftmusic.log";

/// How large the log file is allowed to grow before it is started afresh.
///
/// A player is run for months; without a ceiling its log would grow until
/// somebody noticed. The previous file is kept beside it as `.previous`, so
/// nothing is thrown away without a trace.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// The log file for a given data directory.
#[must_use]
pub fn log_path(data_directory: &Path) -> PathBuf {
    data_directory.join(LOG_DIRECTORY).join(LOG_FILE)
}

/// The log file for the current user, when the platform provides one.
#[must_use]
pub fn default_log_path() -> Option<PathBuf> {
    crate::config::AppConfig::data_directory()
        .ok()
        .map(|directory| log_path(&directory))
}

/// Install the global tracing subscriber.
///
/// Verbosity is controlled with `RUST_LOG` (for example
/// `RUST_LOG=musiccraft=debug`). Calling this more than once is harmless: the
/// second call is ignored because a global subscriber already exists.
pub fn init() {
    init_with(default_log_path().as_deref());
}

/// Install the global tracing subscriber, logging to `path` as well as stderr.
///
/// A log file that cannot be opened is not a reason to fail: the failure is
/// reported on stderr and logging carries on there. Losing the file costs the
/// diagnostic trail, not the application.
pub fn init_with(path: Option<&Path>) {
    let filter =
        EnvFilter::try_from_env(ENV_FILTER).unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let sink = LogSink::open(path);

    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(sink)
        .try_init();

    if installed.is_err() {
        // Another component already installed a subscriber; nothing to do.
        tracing::debug!("logging subscriber was already installed");
    }
}

/// Where a log line is written: the file, when there is one, and stderr.
///
/// Cloning shares the same file, so the subscriber's writer and every clone of
/// it append to one log rather than truncating each other's.
#[derive(Debug, Clone, Default)]
struct LogSink {
    file: Option<Arc<Mutex<File>>>,
}

impl LogSink {
    /// Open `path` for appending, rotating it first when it has grown too big.
    ///
    /// Never fails: a path that cannot be opened logs to stderr alone, with a
    /// note saying so.
    fn open(path: Option<&Path>) -> Self {
        let Some(path) = path else {
            return Self::default();
        };

        let mut file = match open_log_file(path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!(
                    "iCraftMusic could not open its log file at {}: {error}",
                    path.display()
                );
                return Self::default();
            }
        };

        // The file is opened before the subscriber is installed, so this line
        // cannot be logged — the user sees it only if they are running from a
        // terminal, which is exactly when it is useful.
        writeln!(file, "--- iCraftMusic log opened ---").ok();

        Self {
            file: Some(Arc::new(Mutex::new(file))),
        }
    }

    /// Write one formatted log line to every destination.
    ///
    /// A failure to write is deliberately ignored: there is nothing useful to
    /// do about a full disk from inside the logging path, and panicking while
    /// reporting an error would be worse than the error.
    fn write_line(&self, line: &[u8]) {
        io::stderr().write_all(line).ok();

        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            file.write_all(line).ok();
        }
    }
}

/// Open the log file, rotating it aside when it is already at its limit.
///
/// # Errors
///
/// Returns the I/O failure when the directory or the file cannot be created.
fn open_log_file(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::metadata(path).is_ok_and(|metadata| metadata.len() >= MAX_LOG_BYTES) {
        let previous = path.with_extension("log.previous");
        // Renaming over the old file is the rotation: there is only ever one
        // previous log, and it is always the run before this one.
        fs::rename(path, previous)?;
    }

    OpenOptions::new().create(true).append(true).open(path)
}

/// A writer handed to the subscriber for the length of one log event.
///
/// `tracing` writes an event in several pieces, so the pieces are collected
/// here and handed to the sink as one line when the writer is flushed or
/// dropped — which keeps two threads' log lines from interleaving mid-line.
#[derive(Debug)]
struct EventWriter<'a> {
    sink: &'a LogSink,
    buffer: Vec<u8>,
}

impl Write for EventWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.sink.write_line(&self.buffer);
            self.buffer.clear();
        }
        Ok(())
    }
}

impl Drop for EventWriter<'_> {
    fn drop(&mut self) {
        self.flush().ok();
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogSink {
    type Writer = EventWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        EventWriter {
            sink: self,
            buffer: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory of this test's own.
    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-logging-{}-{name}", std::process::id()));
        fs::remove_dir_all(&directory).ok();
        fs::create_dir_all(&directory).expect("the scratch directory");

        directory
    }

    #[test]
    fn the_log_lives_under_the_data_directory() {
        let path = log_path(Path::new("/home/example/.local/share/icraftmusic"));

        assert_eq!(
            path,
            Path::new("/home/example/.local/share/icraftmusic/logs/icraftmusic.log")
        );
    }

    #[test]
    fn a_log_line_reaches_the_file() {
        let directory = scratch("writes");
        let path = directory.join("icraftmusic.log");
        let sink = LogSink::open(Some(&path));

        let mut writer = tracing_subscriber::fmt::MakeWriter::make_writer(&sink);
        writer.write_all(b"indexed 12 tracks\n").expect("write");
        writer.flush().expect("flush");
        drop(writer);

        let contents = fs::read_to_string(&path).expect("the log file is there");
        assert!(
            contents.contains("indexed 12 tracks"),
            "the line never reached the file: {contents:?}"
        );

        fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn an_event_written_in_pieces_arrives_as_one_line() {
        let directory = scratch("pieces");
        let path = directory.join("icraftmusic.log");
        let sink = LogSink::open(Some(&path));

        // This is what `tracing` does: the level, the message and the newline
        // are three writes.
        let mut writer = tracing_subscriber::fmt::MakeWriter::make_writer(&sink);
        writer.write_all(b"INFO ").expect("level");
        writer.write_all(b"scanned 3 tracks").expect("message");
        writer.write_all(b"\n").expect("newline");
        drop(writer);

        let contents = fs::read_to_string(&path).expect("the log file is there");
        assert!(
            contents.contains("INFO scanned 3 tracks\n"),
            "the pieces were not joined: {contents:?}"
        );

        fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn a_log_that_has_grown_too_large_is_started_afresh() {
        let directory = scratch("rotation");
        let path = directory.join("icraftmusic.log");
        fs::write(&path, vec![b'x'; MAX_LOG_BYTES as usize]).expect("a full log");

        let sink = LogSink::open(Some(&path));
        let mut writer = tracing_subscriber::fmt::MakeWriter::make_writer(&sink);
        writer.write_all(b"this run\n").expect("write");
        drop(writer);

        let contents = fs::read_to_string(&path).expect("the new log is there");
        assert!(
            contents.contains("this run"),
            "the new run was not logged: {contents:?}"
        );
        assert!(
            contents.len() < MAX_LOG_BYTES as usize,
            "the file should have been started afresh, not appended to"
        );

        let previous = path.with_extension("log.previous");
        assert!(
            fs::metadata(&previous).expect("the previous log").len() == MAX_LOG_BYTES,
            "the run before this one is kept"
        );

        fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn a_log_file_that_cannot_be_opened_leaves_logging_working() {
        // A directory where the file should be: nothing can be appended to it.
        let directory = scratch("unopenable");
        let path = directory.join("icraftmusic.log");
        fs::create_dir_all(&path).expect("a directory in the file's place");

        let sink = LogSink::open(Some(&path));

        assert!(
            sink.file.is_none(),
            "the sink must fall back to stderr rather than panicking"
        );

        // And writing through it is still harmless.
        let mut writer = tracing_subscriber::fmt::MakeWriter::make_writer(&sink);
        writer.write_all(b"still logging\n").expect("write");
        drop(writer);

        fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn no_log_path_at_all_is_not_a_failure() {
        let sink = LogSink::open(None);
        assert!(sink.file.is_none(), "there is nowhere to put a file");

        // Logging still works: it just goes to stderr alone, which is what a
        // platform without a data directory gets.
        let mut writer = tracing_subscriber::fmt::MakeWriter::make_writer(&sink);
        writer.write_all(b"stderr only\n").expect("write");
        assert!(writer.flush().is_ok());
    }
}
