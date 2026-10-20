//! Application state and the `eframe` integration.
//!
//! The shell renders navigation, content panes and the playback bar, and owns
//! the audio side through a single [`Playback`] facade: the output device, the
//! queue and the transport live there. This module keeps the parts that belong
//! to the window — which section is open, the theme, the settings that get
//! written to disk — and the thin wrappers the views call.
//!
//! The music library is the second half of that job. The database and the
//! folders the user watches are owned here, a scan runs on a worker thread
//! while the window keeps drawing, and the views read what they need through
//! [`MusicCraftApp::browse`] rather than talking to the database themselves.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::CreationContext;

use crate::audio::{AudioDevice, Playback, QueueItem};
use crate::config::{AppConfig, ThemeMode};
use crate::error::AppError;
use crate::library::browse::LibraryBrowse;
use crate::library::{
    CopyProgress, CopySession, ImportSummary, Library, PlaylistId, ScanProgress, ScanSession,
    Track, TrackEditor, TrackId,
};
use crate::playlists::{PlaylistDraft, PlaylistSummary, SmartDraft};
use crate::ui;
use crate::ui::artwork::ArtworkCache;
use crate::ui::dialog::{Confirmation, PendingAdd};
use crate::ui::navigation::NavSection;
use crate::update::{ReleaseCheck, Update};

// The methods live with the surface they serve; each file adds its own
// `impl MusicCraftApp` block to the type defined here.
mod artwork;
mod confirm;
mod editor;
mod library;
mod playback;
mod playlists;
mod release;
mod settings;
mod windows;

/// What was learned about the user's settings before the window opened.
///
/// The window has to be able to open even when the settings file is missing,
/// unreadable or malformed, so a failure is carried alongside the defaults
/// rather than aborting startup.
#[derive(Debug)]
pub struct Startup {
    /// Settings loaded from disk, or the defaults when loading failed.
    pub config: AppConfig,
    /// Where the settings file lives, or `None` when the platform has no
    /// per-user configuration directory.
    pub config_path: Option<PathBuf>,
    /// Why the stored settings were not used, if they were not.
    pub error: Option<AppError>,
}

impl Startup {
    /// Read the settings for the current user.
    pub fn load() -> Self {
        match AppConfig::default_path() {
            Ok(path) => {
                // Whether a settings file was there at all is what separates a
                // first boot from a returning user. A first boot is given the
                // default media home (`~/iCraftMusic`) to watch; anyone whose
                // file loads is left exactly as they left it, including someone
                // who deliberately watches no folder.
                let first_boot = !path.exists();
                let (mut config, error) = AppConfig::load_or_default(&path);
                if first_boot {
                    config.ensure_media_home();
                }
                Self {
                    config,
                    config_path: Some(path),
                    error,
                }
            }
            Err(error) => Self {
                config: AppConfig::default(),
                config_path: None,
                error: Some(error),
            },
        }
    }
}

/// The state the playback bar mirrors.
///
/// It is derived from [`Playback`] on demand rather than stored, so the bar can
/// never disagree with the engine. [`TransportState::Unavailable`] means no
/// output device was opened — the queue and the settings still work, only sound
/// is impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportState {
    /// No audio backend is attached.
    #[default]
    Unavailable,
    /// A track is loaded and playback is stopped.
    Stopped,
    /// Audio is playing.
    Playing,
    /// Playback is temporarily suspended.
    Paused,
}

impl TransportState {
    /// Short label shown next to the playback controls.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "Not connected",
            Self::Stopped => "Stopped",
            Self::Playing => "Playing",
            Self::Paused => "Paused",
        }
    }
}

/// The track shown in the playback bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    /// Track title, taken from the file name until Stage 3 reads tags.
    pub title: String,
    /// Where the track sits in the queue, e.g. `Track 2 of 3`.
    pub position: Option<String>,
    /// The track's length, e.g. `3:45`, when the decoder reported one.
    pub duration: Option<String>,
    /// The cover the library holds for this track, when it holds one.
    pub artwork: Option<PathBuf>,
}

/// What a finished cover download came back with, kept for the album's
/// screen until the next attempt replaces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtworkNotice {
    /// The archive had a front image and the album now draws it.
    Found {
        /// The record the picture was for.
        album: String,
    },
    /// There is no new cover — the archive knows no such release, the network
    /// refused, or the picture was not a picture. The reason is shown as said.
    Missing {
        /// The record that was asked about.
        album: String,
        /// Why there is nothing to show, in the words the fetcher chose.
        reason: String,
    },
}

/// Top-level application state.
pub struct MusicCraftApp {
    config: AppConfig,
    config_path: Option<PathBuf>,
    /// Set whenever a setting changes, cleared once it is written to disk.
    config_dirty: bool,
    config_error: Option<String>,
    section: NavSection,
    /// Whether the queue popover, opened from the playback bar, is showing.
    queue_open: bool,
    /// Whether the settings window, opened from the gear menu, is showing.
    settings_open: bool,
    /// Whether the equalizer window, opened from the gear menu, is showing.
    equalizer_open: bool,
    /// The output device, the queue and the transport, as one object.
    playback: Playback,
    /// The database and the watched folders, or `None` when the database could
    /// not be opened — playback still works in that case.
    library: Option<Library>,
    /// The database file the library was opened from, for the diagnostics.
    database_path: Option<PathBuf>,
    /// Why the library is missing or the last refresh failed, ready to show.
    library_error: Option<String>,
    /// What the Library view is showing: the rows, the query and the selection.
    browse: LibraryBrowse,
    /// The scan running on a worker thread, if one is.
    scan: Option<ScanSession>,
    /// How far the running scan has got.
    scan_progress: ScanProgress,
    /// What the last finished import did, for the status line.
    last_import: Option<String>,
    /// Cover art textures for the Library view, owned here because the views
    /// are redrawn from scratch every frame and cannot hold them.
    artwork: ArtworkCache,
    /// Text the user is typing into the library's play field.
    path_input: String,
    /// Text the user is typing into the library's "import a folder" field.
    folder_input: String,
    /// The metadata editor, when one is open.
    editor: Option<TrackEditor>,
    /// A destructive library action waiting for the user to agree to it.
    pending: Option<Confirmation>,
    /// Chosen music waiting to hear whether it is watched in place or copied
    /// into the media home first.
    pending_add: Option<PendingAdd>,
    /// Further add questions waiting behind the open one. A drop of folders
    /// and loose files asks two honest questions in turn rather than one
    /// umbrella that quietly reclassifies one of them.
    add_queue: std::collections::VecDeque<PendingAdd>,
    /// A copy into the media home running on a worker thread, if one is.
    copy: Option<CopySession>,
    /// How far the running copy has got.
    copy_progress: CopyProgress,
    /// A cover download running on a worker thread, if one is.
    artwork_job: Option<crate::library::artworkdl::ArtworkSession>,
    /// The album the running cover download was asked for, so the album's
    /// screen can say "fetching" about the right record.
    artwork_pending: Option<String>,
    /// What the last cover download said, kept until a new one replaces it.
    artwork_notice: Option<ArtworkNotice>,
    /// Whether the gear on the status bar is showing its menu.
    gear_menu_open: bool,
    /// The output devices the system offered when the list was last read.
    ///
    /// Kept between frames because enumerating devices asks the audio backend,
    /// which is not something to do on every frame: the settings screen reads
    /// this list, and the button beside it reads it again.
    output_devices: Option<Vec<AudioDevice>>,
    /// Why the last settings action failed, ready to show.
    settings_error: Option<String>,
    /// What the last library-database check said, ready to show.
    integrity: Option<String>,
    /// The theme currently installed into `egui`, so it is only re-applied when
    /// the preference actually changes.
    applied_theme: Option<ThemeMode>,
    /// Every playlist the library holds, as a name and its totals.
    ///
    /// Read from the database by [`MusicCraftApp::refresh`] like the track rows
    /// are, so the sidebar and the Playlists section never query it themselves.
    playlists: Vec<PlaylistSummary>,
    /// The playlist the Playlists section is showing, when one is open.
    selected_playlist: Option<PlaylistId>,
    /// The selected playlist's tracks, in play order.
    playlist_tracks: Vec<Track>,
    /// The name editor, when a playlist is being created or renamed.
    playlist_draft: Option<PlaylistDraft>,
    /// The rule editor, when a smart playlist is being made or its rules
    /// edited.
    smart_draft: Option<SmartDraft>,
    /// Why the last playlist action failed, ready to show.
    playlist_error: Option<String>,
    /// Tracks waiting to go into a playlist that is about to be named.
    ///
    /// The "New playlist…" entry in the Library's menu has the tracks in hand
    /// before the playlist exists, so they are held here until the name editor
    /// is answered and the playlist can be made to put them in.
    playlist_pending: Vec<TrackId>,
    /// The release check running on its own thread, until it answers.
    release_check: Option<ReleaseCheck>,
    /// What the last finished release check found, once it has answered.
    release_update: Option<Update>,
    /// This process's own AppImage file, when it is running from one — the
    /// file an update would replace. `None` for every development run, and
    /// the updater says so rather than pretending.
    app_image: Option<std::path::PathBuf>,
    /// The installer's worker, while the download runs.
    update_install: Option<crate::update::InstallSession>,
    /// What the installer last said, kept after it ends: the centre box
    /// keeps telling the user the update is in until they restart.
    install_state: Option<crate::update::Install>,
    /// The version the running install is carrying, for the bar's wording.
    install_version: Option<String>,
}

impl MusicCraftApp {
    /// Build the application from the result of [`Startup::load`].
    ///
    /// The user's library database is opened here, so what the window shows
    /// first is what was indexed last time rather than an empty list.
    pub fn new(_creation_context: &CreationContext<'_>, startup: Startup) -> Self {
        let volume = startup.config.volume;
        let device = startup.config.output_device.clone();
        let mut app = Self::with_playback(
            startup,
            Playback::start_with_device(volume, device.as_deref()),
        );
        app.open_library_from_disk();

        app
    }

    /// Build the application around an existing [`Playback`] and no library.
    ///
    /// Tests pass a player whose device could not be opened, so the transport
    /// can be exercised on machines without a sound card.
    ///
    /// No library is invented here. Encoding a scratch database in this
    /// constructor meant the application opened two libraries at startup — an
    /// in-memory one whose migrations were logged as though the user's own
    /// library were being upgraded, and then the real one — so a start-up log
    /// said the schema had been migrated to 3 while the file had been at 3 all
    /// along. Callers that want a library use
    /// [`MusicCraftApp::with_library`]; the real application calls
    /// [`MusicCraftApp::open_library_from_disk`].
    pub fn with_playback(startup: Startup, mut playback: Playback) -> Self {
        // Shuffle and repeat are preferences like the theme and the volume: the
        // transport is put into the state the user left it in, so the first
        // album played starts shuffled when that is how they listen.
        playback.set_repeat(startup.config.repeat);
        if startup.config.shuffle {
            playback.toggle_shuffle();
        }
        // The audio effects are preferences too: the equalizer the user set and
        // the crossfade length are in force from the first track.
        playback.set_equalizer(startup.config.equalizer.clone());
        playback.set_crossfade(startup.config.crossfade_seconds);

        Self::with_library_later(startup, playback)
    }

    /// Assemble an application whose library is chosen by the caller later.
    ///
    /// Kept separate from [`MusicCraftApp::with_playback`] so the "no library"
    /// case is explicit at the call site rather than a side effect of passing a
    /// player.
    fn with_library_later(startup: Startup, playback: Playback) -> Self {
        Self::from_parts(startup, playback, None, None)
    }

    /// Build the application around a library the caller controls.
    pub fn with_library(startup: Startup, playback: Playback, library: Library) -> Self {
        Self::from_parts(startup, playback, Some(library), None)
    }

    /// Assemble the application, with or without a library.
    ///
    /// The window opens even when no library could be opened: browsing is then
    /// replaced by the reason, and playback, settings and queueing still work.
    /// That failure is reported rather than turned into a panic, because a
    /// read-only or missing data directory is a normal thing to run into.
    fn from_parts(
        startup: Startup,
        playback: Playback,
        library: Option<Library>,
        library_error: Option<String>,
    ) -> Self {
        let error = startup.error.as_ref().map(ToString::to_string);

        let mut app = Self {
            config: startup.config,
            config_path: startup.config_path,
            config_dirty: false,
            config_error: error,
            section: NavSection::default(),
            queue_open: false,
            settings_open: false,
            equalizer_open: false,
            playback,
            library,
            database_path: None,
            library_error,
            browse: LibraryBrowse::new(),
            scan: None,
            scan_progress: ScanProgress::default(),
            last_import: None,
            artwork: ArtworkCache::new(),
            path_input: String::new(),
            folder_input: String::new(),
            editor: None,
            pending: None,
            pending_add: None,
            add_queue: std::collections::VecDeque::new(),
            copy: None,
            copy_progress: CopyProgress::default(),
            artwork_job: None,
            artwork_pending: None,
            artwork_notice: None,
            gear_menu_open: false,
            output_devices: None,
            settings_error: None,
            integrity: None,
            applied_theme: None,
            playlists: Vec::new(),
            selected_playlist: None,
            playlist_tracks: Vec::new(),
            playlist_draft: None,
            smart_draft: None,
            playlist_error: None,
            playlist_pending: Vec::new(),
            release_check: None,
            release_update: None,
            app_image: crate::update::app_image(),
            update_install: None,
            install_state: None,
            install_version: None,
        };

        if let Some(library) = app.library.as_ref() {
            let folders = library.folders().to_vec();
            app.config.library_folders = folders;
        }

        if let Err(error) = app.refresh() {
            app.library_error = Some(error.to_string());
        }

        app
    }
}

/// The desktop's folder chooser, or `None` when the user dismissed it.
///
/// The only call to the file-dialog crate in the application, kept on its own
/// so everything around it — deciding what a chosen folder does, and saying so
/// when it cannot be used — stays testable without a native dialog. It blocks
/// until the dialog is answered, which is what a modal chooser does; nothing is
/// drawn behind it meanwhile.
fn choose_folder(start: Option<&PathBuf>) -> Option<PathBuf> {
    let dialog = rfd::FileDialog::new().set_title("Choose a folder to add to the library");

    // Open where the user's music already is, when there is anywhere to start.
    let dialog = match start {
        Some(folder) if folder.is_dir() => dialog.set_directory(folder),
        _ => dialog,
    };

    dialog.pick_folder()
}

/// The desktop's file chooser, or `None` when the user dismissed it.
///
/// The sibling of [`choose_folder`], and the reason the add dialog exists: the
/// files it returns are not watched anywhere yet, so the user still has to say
/// whether they are indexed where they lie or copied into the media home
/// first.
fn choose_files() -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new()
        .set_title("Choose audio files to add to the library")
        .pick_files()
}

/// A queue entry for a library track, labelled with its tag title.
fn queue_item(track: &Track) -> QueueItem {
    QueueItem::new(track.path.clone(), track.title.clone())
}

/// What to tell the user when a name they typed is already in use.
///
/// The database refuses a duplicate name with its own error, which names a
/// constraint rather than the thing the user is looking at. The playlist rules
/// are the interesting part here — the name is taken, and that is all the user
/// needs to be told — so the message is put into their terms rather than passed
/// through.
fn playlist_conflict(error: &AppError, name: &str) -> String {
    match error {
        AppError::Database {
            source: rusqlite::Error::SqliteFailure(_, Some(message)),
        } if message.contains("playlists.name") => {
            format!("a playlist called `{name}` already exists")
        }
        other => other.to_string(),
    }
}

/// What the now-playing bar shows while the library is being built.
///
/// A scan and a copy wear the same face here because they are the same job
/// seen from the bar: music coming in. `total` is `0` while a job's walk is
/// still counting — the bar slides without numbers until the total is real —
/// and `left_out` counts only what has genuinely been set aside: formats this
/// build cannot read, and files that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexingStatus {
    /// Files read or copied so far.
    pub done: usize,
    /// Files the job expects, or `0` while its walk is still counting.
    pub total: usize,
    /// Files the job has already left out.
    pub left_out: usize,
}

/// What the updater has to show, composed from the release check's answer and
/// the installer's state so that no view has to read — and order — the two
/// itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// A release check is on its way.
    Checking,
    /// A newer release is out. `can_install` is the difference between an
    /// update the app can carry out and one it can only point at: it needs
    /// both a downloadable image and an AppImage to replace.
    Available {
        /// The release's tag, e.g. `v0.2.0`.
        version: String,
        /// Whether the gear menu can promise an update or only a pointer.
        can_install: bool,
    },
    /// An image is being downloaded: `done` of `total` bytes, `total` `0`
    /// until the server has named the size.
    Downloading {
        /// The release being taken in.
        version: String,
        /// Bytes written so far.
        done: u64,
        /// Bytes to write, or `0` while the size is unknown.
        total: u64,
    },
    /// The new image is installed; a restart is what runs it.
    Installed {
        /// The release now on disk.
        version: String,
    },
    /// A download or swap failed. The running image is untouched, and the
    /// menu offers the attempt again.
    Failed {
        /// What stopped it, from the installer's own words.
        reason: String,
    },
}

/// The one-line report shown after an import finishes.
///
/// It names what changed rather than dumping the whole [`ImportSummary`]: the
/// counts a user cares about are the new tracks, the ones that could not be
/// read, and the ones that have gone from disk.
#[must_use]
pub fn import_message(summary: &ImportSummary, cancelled: bool) -> String {
    let scan = &summary.scan;

    if cancelled {
        return format!(
            "Scan stopped after {} {}",
            scan.scanned,
            if scan.scanned == 1 { "track" } else { "tracks" }
        );
    }

    let mut message = format!(
        "Indexed {} {} ({} new, {} updated)",
        scan.scanned,
        if scan.scanned == 1 { "track" } else { "tracks" },
        summary.inserted,
        summary.updated
    );

    if scan.failed > 0 {
        message.push_str(&format!(", {} unreadable", scan.failed));
    }

    // A file the player cannot decode is not a failure of the scan, but it is
    // the reason part of the user's music is not in the list, so the formats
    // are named rather than left as a silent gap.
    if scan.unreadable_count() > 0 {
        let formats = scan.unreadable_formats().join(", ");
        message.push_str(&format!(
            ", {} in formats this player cannot read ({formats})",
            scan.unreadable_count()
        ));
    }

    if summary.missing > 0 {
        message.push_str(&format!(", {} missing from disk", summary.missing));
    }

    message
}

/// The one-line report shown after a copy into the media home finishes.
///
/// Like the scan's report it counts what moved and what could not be read, and
/// says plainly when the user stopped it — a stopped copy leaves files on both
/// sides, so the message must not read as if the job were done.
#[must_use]
pub fn copy_message(report: &crate::library::CopyReport, home: Option<&Path>) -> String {
    let into = match home {
        Some(home) => format!(" into {}", home.display()),
        None => String::new(),
    };

    if report.cancelled {
        return format!(
            "Copy stopped after {} {}{into}",
            report.copied,
            if report.copied == 1 { "file" } else { "files" }
        );
    }

    let mut message = format!(
        "Copied {} {}{into}",
        report.copied,
        if report.copied == 1 { "file" } else { "files" }
    );

    if report.failed > 0 {
        message.push_str(&format!(", {} failed", report.failed));
    }

    message
}

/// Format a track length the way a player shows it: `3:45`, or `1:02:03` once
/// a track runs past an hour.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

impl eframe::App for MusicCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.config.theme) {
            ui::theme::apply(ui.ctx(), self.config.theme);
            self.applied_theme = Some(self.config.theme);
        }

        // A track ends between frames and nothing calls back into the interface,
        // so the frame loop is what advances the queue.
        self.poll_playback();

        // A scan runs on its own thread and reports back through a channel, so
        // the frame loop is also what collects its progress and stores it.
        self.poll_library();

        // Files dragged from the desktop arrive as input of the frame they
        // land in, so the frame loop is what notices a drop and asks where the
        // dropped music should live.
        self.poll_dropped_files(ui.ctx());

        // The release check runs on its own thread for the same reason: the
        // frame loop collects its answer when it arrives — and the installer's
        // progress and outcome are collected the same way.
        self.poll_release_check();
        self.poll_update_install();

        // A cover download asks its question of the archive on a thread of
        // its own, for the same reason everything above does: the frame loop
        // collects the picture, or the honest failure, when it arrives.
        self.poll_album_artwork();

        // The progress bar and the elapsed time only move while audio plays, so
        // ask for another frame for exactly as long as that is true. A running
        // scan or copy asks for frames for the same reason: without them the
        // progress bar and its barber-pole would freeze until the user moved
        // the mouse — and the same for a release check's reply and a download's
        // bytes, which move on their threads and want a frame the moment they
        // arrive.
        if self.transport() == TransportState::Playing
            || self.is_scanning()
            || self.is_copying()
            || self.update_job_running()
            || self.album_artwork_busy()
        {
            ui.ctx().request_repaint();
        }

        ui::render(self, ui);
        self.persist();
    }
}

#[cfg(test)]
mod tests;
