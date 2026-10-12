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

use crate::audio::{Playback, PlaybackEvent, QueueItem, RepeatMode};
use crate::config::{AppConfig, ThemeMode};
use crate::error::{AppError, Result};
use crate::library::browse::LibraryBrowse;
use crate::library::{ImportSummary, Library, ScanProgress, ScanSession, Track, TrackId};
use crate::ui;
use crate::ui::artwork::ArtworkCache;
use crate::ui::navigation::NavSection;

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
                let (config, error) = AppConfig::load_or_default(&path);
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
    /// The output device, the queue and the transport, as one object.
    playback: Playback,
    /// The database and the watched folders, or `None` when the database could
    /// not be opened — playback still works in that case.
    library: Option<Library>,
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
    /// The theme currently installed into `egui`, so it is only re-applied when
    /// the preference actually changes.
    applied_theme: Option<ThemeMode>,
}

impl MusicCraftApp {
    /// Build the application from the result of [`Startup::load`].
    ///
    /// The user's library database is opened here, so what the window shows
    /// first is what was indexed last time rather than an empty list.
    pub fn new(_creation_context: &CreationContext<'_>, startup: Startup) -> Self {
        let volume = startup.config.volume;
        let mut app = Self::with_playback(startup, Playback::start(volume));
        app.open_library_from_disk();

        app
    }

    /// Build the application around an existing [`Playback`].
    ///
    /// Tests pass a player whose device could not be opened, so the transport
    /// can be exercised on machines without a sound card, and get a library
    /// that lives in memory rather than in the user's data directory.
    pub fn with_playback(startup: Startup, playback: Playback) -> Self {
        let artwork_directory =
            std::env::temp_dir().join(crate::library::artwork::ARTWORK_DIRECTORY);

        match Library::in_memory(artwork_directory) {
            Ok(library) => Self::with_library(startup, playback, library),
            Err(error) => Self::from_parts(startup, playback, None, Some(error.to_string())),
        }
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
            playback,
            library,
            library_error,
            browse: LibraryBrowse::new(),
            scan: None,
            scan_progress: ScanProgress::default(),
            last_import: None,
            artwork: ArtworkCache::new(),
            path_input: String::new(),
            folder_input: String::new(),
            applied_theme: None,
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

    /// The section currently shown in the content area.
    pub fn section(&self) -> NavSection {
        self.section
    }

    /// Switch to another section.
    pub fn set_section(&mut self, section: NavSection) {
        self.section = section;
    }

    /// Whether the queue popover is showing.
    pub fn queue_open(&self) -> bool {
        self.queue_open
    }

    /// Open the queue popover, or close it if it is already up.
    pub fn toggle_queue(&mut self) {
        self.queue_open = !self.queue_open;
    }

    /// Close the queue popover.
    pub fn close_queue(&mut self) {
        self.queue_open = false;
    }

    /// The state of the transport, as the playback bar should draw it.
    #[must_use]
    pub fn transport(&self) -> TransportState {
        if !self.playback.is_available() {
            TransportState::Unavailable
        } else if self.playback.is_playing() {
            TransportState::Playing
        } else if self.playback.is_paused() {
            TransportState::Paused
        } else {
            TransportState::Stopped
        }
    }

    /// The track the playback bar should show, if the queue has one selected.
    ///
    /// Tags are not read yet, so the title comes from the file name and the
    /// only other information available is what the queue and the decoder hold.
    #[must_use]
    pub fn now_playing(&self) -> Option<NowPlaying> {
        let track = self.playback.current()?;
        let position = self
            .playback
            .queue_position()
            .map(|(index, total)| format!("Track {index} of {total}"));
        let duration = self.playback.duration().map(format_duration);

        Some(NowPlaying {
            title: track.title().to_owned(),
            position,
            duration,
        })
    }

    /// How far into the current track playback has reached.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.playback.position()
    }

    /// The length of the current track, when the decoder reported one.
    #[must_use]
    pub fn track_duration(&self) -> Option<Duration> {
        self.playback.duration()
    }

    /// How far through the current track playback is, in `0.0..=1.0`.
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.playback.progress()
    }

    /// The audio side, for views that only need to read it.
    #[must_use]
    pub fn playback(&self) -> &Playback {
        &self.playback
    }

    /// The audio side, for views that drive the transport.
    pub fn playback_mut(&mut self) -> &mut Playback {
        &mut self.playback
    }

    /// Start or pause, whichever the current state calls for.
    pub fn toggle_playback(&mut self) {
        self.playback.toggle();
    }

    /// Skip to the next track.
    pub fn next_track(&mut self) {
        self.playback.next();
    }

    /// Go back to the previous track.
    pub fn previous_track(&mut self) {
        self.playback.previous();
    }

    /// Stop playback and unload the track.
    pub fn stop_playback(&mut self) {
        self.playback.stop();
    }

    /// Seek to a fraction of the current track.
    pub fn seek_to_fraction(&mut self, fraction: f32) {
        self.playback.seek_fraction(fraction);
    }

    /// Change how the queue behaves at the end of a track.
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.playback.set_repeat(mode);
    }

    /// Turn shuffle on or off, returning the new state.
    pub fn toggle_shuffle(&mut self) -> bool {
        self.playback.toggle_shuffle()
    }

    /// Move a queued track to another position, returning whether it moved.
    ///
    /// Nothing about the transport changes: this is the Queue page reordering
    /// the list, not the user skipping.
    pub fn move_in_queue(&mut self, from: usize, to: usize) -> bool {
        self.playback.move_in_queue(from, to)
    }

    /// Take a queued track out, returning whether it came out.
    ///
    /// The track being played is not removable; the Queue view greys the button
    /// out rather than offering an action the player would refuse.
    pub fn remove_from_queue(&mut self, index: usize) -> bool {
        self.playback.remove_from_queue(index).is_some()
    }

    /// Empty the queue, stopping playback so nothing plays from a spent list.
    pub fn clear_queue(&mut self) {
        self.playback.clear_queue();
    }

    /// Jump to a queued track and start it.
    pub fn play_queue_index(&mut self, index: usize) {
        self.playback.play_index(index);
    }

    /// Let the player notice that a track ended, and advance if it did.
    ///
    /// The engine cannot call back into the interface, so this runs once per
    /// frame from [`eframe::App::ui`].
    pub fn poll_playback(&mut self) -> Option<PlaybackEvent> {
        self.playback.poll()
    }

    /// The failure the player wants the user to see, ready to display.
    #[must_use]
    pub fn playback_error(&self) -> Option<String> {
        self.playback.error().map(ToString::to_string)
    }

    /// Hide the playback failure message.
    pub fn dismiss_playback_error(&mut self) {
        self.playback.dismiss_error();
    }

    /// The text in the library's "play a file or folder" field.
    #[must_use]
    pub fn path_input(&self) -> &str {
        &self.path_input
    }

    /// The play field's text, for the view to edit.
    pub fn path_input_mut(&mut self) -> &mut String {
        &mut self.path_input
    }

    /// Play whatever the user typed into the play field, if anything.
    ///
    /// The field is cleared once something was added, so the next entry starts
    /// from an empty line.
    pub fn play_path_input(&mut self) -> usize {
        let text = self.path_input.trim().to_owned();
        if text.is_empty() {
            return 0;
        }

        let added = self.open_path(Path::new(&text));
        if added > 0 {
            self.path_input.clear();
        }

        added
    }

    /// Loads one file or folder into the queue, and plays it when idle.
    pub fn open_path(&mut self, path: &Path) -> usize {
        let added = self.playback.enqueue_path(path);
        self.start_if_idle(added);

        added
    }

    /// Loads several files or folders, and plays the first when idle.
    ///
    /// The command line uses this so `musiccraft album/` starts playing
    /// immediately, and folders are scanned one level deep.
    pub fn open_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;
        for path in paths {
            added += self.playback.enqueue_path(&path);
        }
        self.start_if_idle(added);

        added
    }

    /// Starts the selected track when something was just added and nothing is
    /// playing, so selecting a folder begins at its first track.
    fn start_if_idle(&mut self, added: usize) {
        if added > 0 && !self.playback.is_playing() {
            self.playback.play_current();
        }
    }

    /// Open the user's library database and remember the folders they watch.
    ///
    /// Failures are kept in [`MusicCraftApp::library_error`] and shown by the
    /// Library view rather than aborting startup.
    pub fn open_library_from_disk(&mut self) {
        let data_directory = match AppConfig::data_directory() {
            Ok(directory) => directory,
            Err(error) => {
                self.library_error = Some(error.to_string());
                return;
            }
        };

        let database = data_directory.join(crate::database::DATABASE_FILE);
        let artwork_directory = crate::library::artwork::cache_directory(&data_directory);
        let folders = self.config.library_folders.clone();

        self.open_library(&database, artwork_directory, folders);
    }

    /// Open the library stored at `database_path` and browse `folders`.
    ///
    /// Split from [`MusicCraftApp::open_library_from_disk`] so a test can point
    /// a library at a database of its own instead of the user's.
    pub fn open_library(
        &mut self,
        database_path: &Path,
        artwork_directory: PathBuf,
        folders: Vec<PathBuf>,
    ) {
        match Library::open(database_path, artwork_directory) {
            Ok(mut library) => {
                library.set_folders(folders);
                self.library = Some(library);
                self.library_error = None;

                if let Err(error) = self.refresh() {
                    self.library_error = Some(error.to_string());
                }
            }
            Err(error) => {
                self.library = None;
                self.library_error =
                    Some(format!("the music library could not be opened: {error}"));
            }
        }
    }

    /// The library, when one could be opened.
    #[must_use]
    pub fn library(&self) -> Option<&Library> {
        self.library.as_ref()
    }

    /// The library, mutably, for storing tracks outside a scan.
    pub fn library_mut(&mut self) -> Option<&mut Library> {
        self.library.as_mut()
    }

    /// Why the library is unavailable, or why the last library action failed.
    #[must_use]
    pub fn library_error(&self) -> Option<&str> {
        self.library_error.as_deref()
    }

    /// Hide the library failure banner.
    pub fn dismiss_library_error(&mut self) {
        self.library_error = None;
    }

    /// The folders the library is built from.
    #[must_use]
    pub fn library_folders(&self) -> &[PathBuf] {
        &self.config.library_folders
    }

    /// What the Library view is showing.
    #[must_use]
    pub fn browse(&self) -> &LibraryBrowse {
        &self.browse
    }

    /// The browsing state, mutably, for the view's click handling.
    pub fn browse_mut(&mut self) -> &mut LibraryBrowse {
        &mut self.browse
    }

    /// The cover-art textures the Library view draws with.
    pub fn artwork_mut(&mut self) -> &mut ArtworkCache {
        &mut self.artwork
    }

    /// The text in the Library's import field.
    #[must_use]
    pub fn folder_input(&self) -> &str {
        &self.folder_input
    }

    /// The import field's text, for the view to edit.
    pub fn folder_input_mut(&mut self) -> &mut String {
        &mut self.folder_input
    }

    /// Import the folder typed into the import field, returning whether it was
    /// new to the library.
    ///
    /// The field is cleared once the folder was accepted, so the next entry
    /// starts from an empty line.
    pub fn import_folder_input(&mut self) -> bool {
        let text = self.folder_input.trim().to_owned();
        if text.is_empty() {
            return false;
        }

        let added = self.add_library_folder(Path::new(&text));
        if added {
            self.folder_input.clear();
        }

        added
    }

    /// Start watching `folder` and scan it, returning whether it was new.
    ///
    /// The folder is written to the settings file, which is what makes a
    /// library survive a restart; the scan itself runs on a worker thread, so
    /// this returns while the import is still going on.
    pub fn add_library_folder(&mut self, folder: &Path) -> bool {
        if folder.as_os_str().is_empty() || !folder.is_dir() {
            self.library_error = Some(format!(
                "`{}` is not a folder that can be read.",
                folder.display()
            ));
            return false;
        }

        if self.library.is_none() {
            self.library_error = Some("there is no library to import into".to_owned());
            return false;
        }

        let added = self.config.add_library_folder(folder);
        if added {
            self.config_dirty = true;
            if let Some(library) = self.library.as_mut() {
                library.set_folders(self.config.library_folders.clone());
            }
        }

        // Rescan even when the folder was already watched: a user who asks
        // again is asking for a refresh, not for nothing to happen.
        self.start_scan();

        added
    }

    /// Scan the watched folders, unless a scan is already running.
    ///
    /// # Errors
    ///
    /// The reason a scan could not start is kept in [`MusicCraftApp::library_error`].
    pub fn start_scan(&mut self) {
        if self.scan.is_some() {
            return;
        }

        let Some(library) = self.library.as_ref() else {
            self.library_error = Some("there is no library to scan into".to_owned());
            return;
        };

        if library.folders().is_empty() {
            self.library_error = Some("add a folder before scanning".to_owned());
            return;
        }

        let request = library.scan_request();
        tracing::info!("scanning {} folders", request.folders.len());

        self.library_error = None;
        self.last_import = None;
        self.scan_progress = ScanProgress::default();
        self.scan = Some(ScanSession::spawn(request));
    }

    /// Whether a scan is running on a worker thread.
    #[must_use]
    pub fn is_scanning(&self) -> bool {
        self.scan.is_some()
    }

    /// How far the running scan has got.
    #[must_use]
    pub fn scan_progress(&self) -> &ScanProgress {
        &self.scan_progress
    }

    /// What the last finished import did, ready to show.
    #[must_use]
    pub fn last_import(&self) -> Option<&str> {
        self.last_import.as_deref()
    }

    /// Ask the running scan to stop after the file it is on.
    pub fn cancel_scan(&mut self) {
        if let Some(scan) = self.scan.as_ref() {
            scan.cancel();
        }
    }

    /// Let a running scan make progress, and store what it found when it ends.
    ///
    /// This runs once per frame from [`eframe::App::ui`]: the scan itself is on
    /// a worker thread, and the worker is only joined once it has reported that
    /// it is finished, so the frame loop never waits on a scan.
    pub fn poll_library(&mut self) {
        if self.scan.is_none() {
            return;
        }

        if let Some(scan) = self.scan.as_ref() {
            if let Some(progress) = scan.try_progress() {
                self.scan_progress = progress;
            }

            if !scan.is_finished() {
                return;
            }
        }

        let Some(scan) = self.scan.take() else {
            return;
        };
        let report = scan.join();
        let cancelled = report.cancelled;

        let stored = match self.library.as_mut() {
            Some(library) => library.store(report).map_err(|error| error.to_string()),
            None => Err("there is no library to store the scan in".to_owned()),
        };

        match stored {
            Ok(summary) => {
                self.last_import = Some(import_message(&summary, cancelled));
                if let Err(error) = self.refresh() {
                    self.library_error = Some(error.to_string());
                }
            }
            Err(error) => {
                self.library_error = Some(format!("the scan could not be saved: {error}"));
            }
        }
    }

    /// Read the library back out of the database into the browsing state.
    ///
    /// # Errors
    ///
    /// Returns the database failure, so the caller can decide how to show it.
    pub fn refresh(&mut self) -> Result<()> {
        let Some(library) = self.library.as_ref() else {
            return Ok(());
        };

        let tracks = library.tracks()?;
        let albums = library.albums()?;
        let artists = library.artists()?;

        self.browse.replace(tracks, albums, artists);
        tracing::debug!("library refreshed: {} tracks", self.browse.track_count());

        Ok(())
    }

    /// Replace the queue with `tracks` and start playing the one at `start`.
    ///
    /// The queue is rebuilt from what the user is looking at, so playing the
    /// fourth row of a sorted, filtered table plays that row and what follows
    /// it, in the order shown on screen.
    pub fn play_tracks(&mut self, tracks: &[Track], start: usize) -> usize {
        let items: Vec<QueueItem> = tracks.iter().map(queue_item).collect();
        let count = items.len();
        self.playback.play_items(items, start);

        count
    }

    /// Play the selected tracks, starting at the first one.
    ///
    /// Returns how many tracks were queued, so the view can tell "nothing was
    /// selected" from "playing".
    pub fn play_selection(&mut self) -> usize {
        let selected: Vec<Track> = self.browse.selected_tracks().into_iter().cloned().collect();

        self.play_tracks(&selected, 0)
    }

    /// Add the selected tracks to the end of the queue, without playing them.
    pub fn queue_selection(&mut self) -> usize {
        let selected: Vec<Track> = self.browse.selected_tracks().into_iter().cloned().collect();

        self.queue_tracks(&selected)
    }

    /// Add tracks to the end of the queue, without playing them.
    ///
    /// Returns how many entries were added, which is zero for an empty slice.
    pub fn queue_tracks(&mut self, tracks: &[Track]) -> usize {
        let items: Vec<QueueItem> = tracks.iter().map(queue_item).collect();

        self.playback.queue_items(items)
    }

    /// Add every track of an album to the end of the queue.
    pub fn queue_album(&mut self, album: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_album(album)
            .into_iter()
            .cloned()
            .collect();

        self.queue_tracks(&tracks)
    }

    /// Add every track credited to an artist to the end of the queue.
    pub fn queue_artist(&mut self, artist: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_artist(artist)
            .into_iter()
            .cloned()
            .collect();

        self.queue_tracks(&tracks)
    }

    /// Play every track of an album, in album order.
    pub fn play_album(&mut self, album: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_album(album)
            .into_iter()
            .cloned()
            .collect();

        self.play_tracks(&tracks, 0)
    }

    /// Play every track credited to an artist.
    pub fn play_artist(&mut self, artist: &str) -> usize {
        let tracks: Vec<Track> = self
            .browse
            .tracks_of_artist(artist)
            .into_iter()
            .cloned()
            .collect();

        self.play_tracks(&tracks, 0)
    }

    /// Forget the selected tracks, leaving their files on disk untouched.
    ///
    /// Returns how many rows were removed, which is zero when nothing was
    /// selected. This is the only destructive library action in the interface,
    /// and it is deliberately the one that cannot lose music: the database is a
    /// cache of what the files say, so a later scan puts the tracks back.
    pub fn remove_selection_from_library(&mut self) -> usize {
        let ids: Vec<TrackId> = self.browse.selection().ids().collect();
        if ids.is_empty() {
            return 0;
        }

        let mut removed = 0;
        let mut failure = None;

        if let Some(library) = self.library.as_ref() {
            for id in ids {
                match library.repository().remove_track(id) {
                    Ok(true) => removed += 1,
                    Ok(false) => {}
                    Err(error) => failure = Some(error.to_string()),
                }
            }
        }

        self.browse.selection_mut().clear();

        if let Some(error) = failure {
            self.library_error = Some(format!("a track could not be removed: {error}"));
        }

        // A refresh failure is reported like any other library failure rather
        // than thrown away: the rows are gone from the database either way, and
        // the view should say so if it could not be updated.
        if removed > 0
            && let Err(error) = self.refresh()
        {
            self.library_error = Some(error.to_string());
        }

        removed
    }

    /// The tracks of an album, ready to be queued.
    #[must_use]
    pub fn album_tracks(&self, album: &str) -> Vec<Track> {
        self.browse
            .tracks_of_album(album)
            .into_iter()
            .cloned()
            .collect()
    }

    /// The configured theme.
    pub fn theme(&self) -> ThemeMode {
        self.config.theme
    }

    /// Change the theme and schedule a save.
    pub fn set_theme(&mut self, theme: ThemeMode) {
        if self.config.theme != theme {
            self.config.theme = theme;
            self.config_dirty = true;
        }
    }

    /// The configured default volume, in the range `0.0..=1.0`.
    pub fn volume(&self) -> f32 {
        self.config.volume
    }

    /// Change the default volume and schedule a save.
    ///
    /// The value is clamped because the setting is written to disk, and it is
    /// handed to the player straight away so the change is audible.
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        if self.config.volume != volume {
            self.config.volume = volume;
            self.config_dirty = true;
        }
        self.playback.set_volume(volume);
    }

    /// Where settings are stored, when the platform provides a location.
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// The most recent settings failure, shown in the top bar.
    pub fn config_error(&self) -> Option<&str> {
        self.config_error.as_deref()
    }

    /// Hide the settings failure banner.
    pub fn dismiss_config_error(&mut self) {
        self.config_error = None;
    }

    /// Write the settings back to disk when something changed.
    ///
    /// Failures are reported through the top bar rather than panicking: losing
    /// a preference is annoying, not fatal, and the in-memory value is still
    /// correct for this session.
    fn persist(&mut self) {
        if !self.config_dirty {
            return;
        }

        let Some(path) = self.config_path.clone() else {
            // Nothing to write to; stop retrying every frame.
            self.config_dirty = false;
            return;
        };

        match self.config.save(&path) {
            Ok(()) => {
                tracing::debug!("settings written to {}", path.display());
            }
            Err(error) => {
                tracing::error!("failed to save settings to {}: {error}", path.display());
                self.config_error = Some(error.to_string());
            }
        }

        // Either way, do not retry on every frame: the user sees the banner and
        // the next change will try again.
        self.config_dirty = false;
    }
}

/// A queue entry for a library track, labelled with its tag title.
fn queue_item(track: &Track) -> QueueItem {
    QueueItem::new(track.path.clone(), track.title.clone())
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

    if summary.missing > 0 {
        message.push_str(&format!(", {} missing from disk", summary.missing));
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

        // The progress bar and the elapsed time only move while audio plays, so
        // ask for another frame for exactly as long as that is true. A running
        // scan asks for frames for the same reason: without them the progress
        // count would freeze until the user moved the mouse.
        if self.transport() == TransportState::Playing || self.is_scanning() {
            ui.ctx().request_repaint();
        }

        ui::render(self, ui);
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::audio::AudioEngine;

    #[test]
    fn transport_starts_unavailable() {
        assert_eq!(TransportState::default(), TransportState::Unavailable);
    }

    #[test]
    fn every_transport_state_has_a_label() {
        for state in [
            TransportState::Unavailable,
            TransportState::Stopped,
            TransportState::Playing,
            TransportState::Paused,
        ] {
            assert!(!state.label().is_empty(), "{state:?} has no label");
        }
    }

    #[test]
    fn startup_without_config_dir_still_yields_defaults() {
        let startup = Startup {
            config: AppConfig::default(),
            config_path: None,
            error: Some(AppError::NoConfigDir),
        };

        assert!(startup.config_path.is_none());
        assert!(startup.error.is_some());
        assert!((startup.config.volume - crate::config::DEFAULT_VOLUME).abs() < f32::EPSILON);
    }

    #[test]
    fn set_section_changes_the_active_section() {
        let mut app = app();
        assert_eq!(app.section(), NavSection::Library);

        app.set_section(NavSection::Settings);
        assert_eq!(app.section(), NavSection::Settings);
    }

    #[test]
    fn changing_the_theme_marks_the_config_dirty() {
        let mut app = app();
        assert!(!app.config_dirty);

        app.set_theme(ThemeMode::Light);
        assert_eq!(app.theme(), ThemeMode::Light);
        assert!(app.config_dirty, "a real change should schedule a save");
    }

    #[test]
    fn setting_the_theme_it_already_has_is_a_no_op() {
        let mut app = app();

        app.set_theme(app.theme());
        assert!(
            !app.config_dirty,
            "an unchanged value should not dirty the config"
        );
    }

    #[test]
    fn set_volume_clamps_marks_dirty_and_reaches_the_player() {
        let mut app = app();

        app.set_volume(2.0);
        assert_eq!(app.volume(), 1.0);
        assert!((app.playback().volume() - 1.0).abs() < f32::EPSILON);

        app.set_volume(-1.0);
        assert_eq!(app.volume(), 0.0);
        assert!((app.playback().volume() - 0.0).abs() < f32::EPSILON);

        assert!(app.config_dirty);
    }

    #[test]
    fn persist_without_a_path_clears_the_dirty_flag() {
        let mut app = app();
        app.set_theme(ThemeMode::Light);

        app.persist();

        assert!(!app.config_dirty, "nothing to write to, so stop retrying");
        assert!(app.config_error().is_none());
    }

    #[test]
    fn lengths_are_formatted_as_a_player_shows_them() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0:00");
        assert_eq!(format_duration(Duration::from_secs(9)), "0:09");
        assert_eq!(format_duration(Duration::from_secs(225)), "3:45");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1:00:00");
        assert_eq!(format_duration(Duration::from_secs(3723)), "1:02:03");
    }

    #[test]
    fn the_transport_is_unavailable_without_an_output_device() {
        let app = app();

        assert_eq!(app.transport(), TransportState::Unavailable);
        assert!(!app.playback().is_available());
    }

    #[test]
    fn opening_a_file_selects_it_and_reports_it_as_now_playing() {
        let mut app = app();

        let added = app.open_path(Path::new("/music/song.mp3"));

        assert_eq!(added, 1);
        assert_eq!(
            app.now_playing(),
            Some(NowPlaying {
                title: "song".to_owned(),
                position: Some("Track 1 of 1".to_owned()),
                duration: None,
            })
        );
    }

    #[test]
    fn a_missing_file_surfaces_an_error_instead_of_panicking() {
        let mut app = app();

        app.open_path(Path::new("/music/does-not-exist.mp3"));

        // The file was accepted into the queue, so playing it is what fails.
        // Without an output device the device error is what the user hears
        // about first, so the assertion is that a message reaches the bar at
        // all — the file is named again on a machine with sound.
        let message = app
            .playback_error()
            .expect("a failing track should be reported for the interface");
        assert!(!message.is_empty());

        // A failure leaves the app coherent: the queue survives it and the
        // transport is not left claiming to play.
        assert_eq!(app.playback().queue().len(), 1);
        assert!(!app.playback().is_playing());

        app.dismiss_playback_error();
        assert!(app.playback_error().is_none(), "the message is dismissable");
    }

    #[test]
    fn a_folder_with_no_audio_adds_nothing_and_stays_quiet() {
        let mut app = app();
        let folder = std::env::temp_dir().join("musiccraft-empty-folder");
        std::fs::create_dir_all(&folder).expect("a temp folder should be creatable");
        std::fs::write(folder.join("cover.png"), b"not audio").expect("the file should be written");

        let added = app.open_path(&folder);

        assert_eq!(added, 0);
        assert_eq!(app.now_playing(), None);
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn transport_controls_without_a_device_do_not_panic() {
        let mut app = app();

        app.toggle_playback();
        assert!(
            app.playback_error().is_some(),
            "starting an empty queue should be reported"
        );

        app.dismiss_playback_error();
        app.next_track();
        app.previous_track();
        app.seek_to_fraction(0.5);
        app.stop_playback();
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn playing_the_input_field_clears_it_once_something_was_added() {
        let mut app = app();
        *app.path_input_mut() = "  /music/song.mp3  ".to_owned();

        let added = app.play_path_input();

        assert_eq!(added, 1);
        assert_eq!(app.path_input(), "", "the field resets after a success");
    }

    #[test]
    fn playing_an_empty_input_field_does_nothing() {
        let mut app = app();
        *app.path_input_mut() = "   ".to_owned();

        assert_eq!(app.play_path_input(), 0);
        assert!(app.playback_error().is_none());
    }

    #[test]
    fn a_player_reports_its_own_state_through_the_transport() {
        let mut playback = Playback::with_engine(AudioEngine::without_device(), 0.5);
        playback.enqueue([PathBuf::from("/music/song.mp3")]);
        let app = MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            playback,
        );

        assert!((app.volume() - crate::config::DEFAULT_VOLUME).abs() < f32::EPSILON);
        assert_eq!(app.transport(), TransportState::Unavailable);
    }

    /// An application with a player that has no output device.
    fn app() -> MusicCraftApp {
        MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
        )
    }
}
