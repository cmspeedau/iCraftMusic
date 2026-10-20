//! The music library: folders watched, scans, imports, copies and the media home.

use super::{
    IndexingStatus, MusicCraftApp, choose_files, choose_folder, copy_message, import_message,
};
use crate::config::AppConfig;
use crate::error::Result;
use crate::library::browse::LibraryBrowse;
use crate::library::{
    CopyProgress, CopyRequest, CopySession, Library, ScanProgress, ScanSession, Track, TrackId,
};
use crate::playlists::Playlists;
use crate::ui::artwork::ArtworkCache;
use crate::ui::dialog::PendingAdd;
use std::path::{Path, PathBuf};

impl MusicCraftApp {
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

        // The media home a first boot was given is created on the user's
        // behalf — the way iTunes creates *iTunes Media* — so there is
        // somewhere to put music from the very first run. A folder the user
        // chose is never created here: a chosen folder that is missing is a
        // mistake to report when it is scanned, not a directory to invent
        // behind their back.
        if let Some(home) = self.config.media_home().map(PathBuf::from)
            && AppConfig::default_media_home().as_ref() == Some(&home)
            && !home.exists()
            && let Err(error) = std::fs::create_dir_all(&home)
        {
            tracing::warn!(
                "could not create the media home {}: {error}",
                home.display()
            );
        }

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
                self.database_path = Some(database_path.to_path_buf());
                self.library_error = None;

                if let Err(error) = self.refresh() {
                    self.library_error = Some(error.to_string());
                }
            }
            Err(error) => {
                self.library = None;
                self.database_path = Some(database_path.to_path_buf());
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

    /// Ask the desktop for a music folder, and start watching it.
    ///
    /// The chooser is the platform's own — the same dialog a file manager
    /// opens, so the places the user already knows are there. Whatever comes
    /// back takes the same route as a folder typed into the field: it is put in
    /// the field and imported, which is what makes a typo and a wrong choice
    /// fail in exactly the same way.
    ///
    /// Dismissing the dialog is not an error and does nothing at all.
    pub fn choose_library_folder(&mut self) {
        let Some(folder) = choose_folder(self.library_folders().first()) else {
            return;
        };

        self.folder_input = folder.display().to_string();
        self.import_folder_input();
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

    /// Stop watching `folder`, returning whether it was being watched.
    ///
    /// The tracks it contributed stay in the library and their files stay on
    /// disk: this changes what is scanned, not what is indexed. The settings
    /// file is written at the end of the frame, so the folder stays gone after
    /// a restart.
    pub fn remove_library_folder(&mut self, folder: &Path) -> bool {
        let removed = self.config.remove_library_folder(folder);
        if !removed {
            return false;
        }

        self.config_dirty = true;
        if let Some(library) = self.library.as_mut() {
            library.set_folders(self.config.library_folders.clone());
        }

        tracing::info!("stopped watching {}", folder.display());

        true
    }

    /// Ask the desktop for audio files, then ask where they should live.
    ///
    /// The chooser blocks until it is answered, so this belongs to a click and
    /// to nothing a frame does on its own. A dismissed dialog asks nothing
    /// afterwards.
    pub fn add_files(&mut self) {
        if let Some(paths) = choose_files() {
            self.ask_to_add(PendingAdd::Files { paths });
        }
    }

    /// Ask the desktop for a folder, then ask where its music should live.
    pub fn add_folder(&mut self) {
        let start = self.config.media_home().map(PathBuf::from);
        if let Some(folder) = choose_folder(start.as_ref()) {
            self.ask_to_add(PendingAdd::Folders {
                paths: vec![folder],
            });
        }
    }

    /// Show the add dialog for `add`; an empty choice asks no questions.
    pub fn ask_to_add(&mut self, add: PendingAdd) {
        if !add.paths().is_empty() {
            self.pending_add = Some(add);
        }
    }

    /// The add dialog's question, while it waits to be answered.
    #[must_use]
    pub fn pending_add(&self) -> Option<PendingAdd> {
        self.pending_add.clone()
    }

    /// The library's media home — the folder the add dialog's "Copy" copies
    /// into — if the library has one.
    #[must_use]
    pub fn media_home(&self) -> Option<&Path> {
        self.config.media_home()
    }

    /// Close the add dialog without adding anything.
    pub fn cancel_add(&mut self) {
        self.pending_add = None;
        self.pump_add_queue();
    }

    /// Show the next queued add question, if the dialog is not already asking.
    fn pump_add_queue(&mut self) {
        if self.pending_add.is_none() {
            self.pending_add = self.add_queue.pop_front();
        }
    }

    /// Consume the files the desktop dropped onto the window this frame.
    ///
    /// A drop is an addition like any other and gets the add dialog's question
    /// asked of it — folders in one question, loose files in the next — so the
    /// window accepts music from the pointer exactly as the chooser does from
    /// the gear menu. Dropping is never adding: the dialog still decides.
    pub fn poll_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });

        if dropped.is_empty() {
            return;
        }

        let folders: Vec<PathBuf> = dropped
            .iter()
            .filter(|path| path.is_dir())
            .cloned()
            .collect();
        let files: Vec<PathBuf> = dropped
            .iter()
            .filter(|path| !path.is_dir())
            .cloned()
            .collect();

        if !folders.is_empty() {
            self.add_queue
                .push_back(PendingAdd::Folders { paths: folders });
        }

        if !files.is_empty() {
            self.add_queue.push_back(PendingAdd::Files { paths: files });
        }

        self.pump_add_queue();
    }

    /// Answer the add dialog with "Add": watch what was chosen, where it is.
    ///
    /// Folders and single files are watched alike — the scanner reads an entry
    /// that is a file as the one file to index — so one answer covers both
    /// kinds of choice. The scan reports what arrived; the dialog itself
    /// promises nothing beyond watching.
    pub fn add_paths_in_place(&mut self) {
        let Some(add) = self.pending_add.take() else {
            return;
        };
        self.pump_add_queue();

        for path in add.paths() {
            self.config.add_library_folder(path.clone());
        }
        self.config_dirty = true;
        if let Some(library) = self.library.as_mut() {
            library.set_folders(self.config.library_folders.clone());
        }

        self.start_scan();
    }

    /// Answer the add dialog with "Copy": copy into the media home on a worker
    /// thread, and index the copies once it lands.
    ///
    /// A scan may run alongside a copy: a file caught mid-write fails to read,
    /// is reported as unreadable, and the scan that follows the finished copy
    /// indexes it properly. Nothing is moved or deleted either way — the
    /// originals are exactly where the user left them.
    pub fn copy_paths_into_home(&mut self) {
        let Some(add) = self.pending_add.take() else {
            return;
        };
        self.pump_add_queue();

        let home = match self.config.media_home().map(PathBuf::from) {
            Some(home) => home,
            None => match AppConfig::default_media_home() {
                Some(home) => {
                    self.config.set_media_home(home.clone());
                    self.config_dirty = true;
                    if let Some(library) = self.library.as_mut() {
                        library.set_folders(self.config.library_folders.clone());
                    }
                    home
                }
                None => {
                    self.library_error =
                        Some("this platform has no home folder to copy into".to_owned());
                    return;
                }
            },
        };

        if self.copy.is_some() {
            self.library_error = Some("a copy is already running".to_owned());
            return;
        }

        self.library_error = None;
        self.copy_progress = CopyProgress::default();
        self.copy = Some(CopySession::spawn(CopyRequest {
            paths: add.paths().to_vec(),
            home,
        }));
    }

    /// Whether a copy is running on a worker thread.
    #[must_use]
    pub fn is_copying(&self) -> bool {
        self.copy.is_some()
    }

    /// How far the running copy has got.
    #[must_use]
    pub fn copy_progress(&self) -> &CopyProgress {
        &self.copy_progress
    }

    /// Ask the running copy to stop after the file it is on.
    pub fn cancel_copy(&mut self) {
        if let Some(copy) = self.copy.as_ref() {
            copy.cancel();
        }
    }

    /// The now-playing bar's Indexing widget, ready to be drawn.
    ///
    /// One getter rather than the view reading scans and copies separately:
    /// to the bar they are the same job — the library being built — and the
    /// widget must not flicker a second verb when a copy hands over to the
    /// scan that indexes what it moved.
    #[must_use]
    pub fn indexing_status(&self) -> Option<IndexingStatus> {
        if self.is_scanning() {
            let progress = &self.scan_progress;
            return Some(IndexingStatus {
                done: progress.processed,
                total: progress.total,
                left_out: progress.failed + progress.unreadable,
            });
        }

        if self.is_copying() {
            let progress = &self.copy_progress;
            return Some(IndexingStatus {
                done: progress.copied,
                total: progress.total,
                left_out: progress.failed,
            });
        }

        None
    }

    /// Stop whichever indexing job the bar's Cancel button is pointing at.
    pub fn cancel_indexing(&mut self) {
        if self.is_scanning() {
            self.cancel_scan();
        } else {
            self.cancel_copy();
        }
    }

    /// Ask the desktop for a new location for the media home, and move the
    /// library's watch to it.
    ///
    /// The library follows the folder: the files that were in the old home are
    /// not moved, because moving an entire library's bytes is a job that needs
    /// its own progress and verification, and guessing wrong about a copy of
    /// that size is not something a folder chooser should do behind a dialog.
    pub fn change_media_home(&mut self) {
        let start = self.config.media_home().map(PathBuf::from);
        if let Some(folder) = choose_folder(start.as_ref()) {
            self.set_media_home_now(folder);
        }
    }

    /// Return the media home to the location a first boot would have chosen.
    ///
    /// This is the Settings pane's Reset button: wherever the home had been
    /// moved, `~/iCraftMusic` — the one place the application always knows —
    /// becomes the home again.
    pub fn reset_media_home(&mut self) {
        if self.config.reset_media_home() {
            let home = self.config.media_home().map(PathBuf::from);
            self.sync_media_home();
            if let Some(home) = home {
                self.ensure_home_exists(&home);
            }
            self.start_scan();
        }
    }

    /// Point the library and its settings at the home the config now names.
    fn set_media_home_now(&mut self, folder: PathBuf) {
        if self.config.media_home() == Some(folder.as_path()) {
            // Asking for the home it already has is a refresh, the way choosing
            // a watched folder again rescans it.
            self.start_scan();
            return;
        }

        self.config.set_media_home(folder.clone());
        self.config_dirty = true;
        self.sync_media_home();
        self.ensure_home_exists(&folder);
        self.start_scan();
    }

    /// Hand the config's watched folders to the library, so a scan after a
    /// home change sees the change rather than the old list.
    fn sync_media_home(&mut self) {
        if let Some(library) = self.library.as_mut() {
            library.set_folders(self.config.library_folders.clone());
        }
    }

    /// Create `folder` if it is not there, without ceremony: the home is the
    /// one folder the application owns, so a home that was chosen or reset to
    /// but does not exist yet is a folder to make, not a mistake to report.
    fn ensure_home_exists(&mut self, folder: &Path) {
        if !folder.exists()
            && let Err(error) = std::fs::create_dir_all(folder)
        {
            self.library_error = Some(format!(
                "{} could not be created: {error}",
                folder.display()
            ));
        }
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

    /// Let the running scan and the running copy make progress, finishing
    /// either when its worker reports done.
    ///
    /// This runs once per frame from [`eframe::App::ui`]: the work itself is on
    /// worker threads, and a worker is only joined once it has reported that it
    /// is finished, so the frame loop never waits on either.
    pub fn poll_library(&mut self) {
        self.poll_scan();
        self.poll_copy();
    }

    /// Let a running scan make progress, and store what it found when it ends.
    fn poll_scan(&mut self) {
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

    /// Let a running copy make progress, and say what it did when it ends.
    ///
    /// A copy that landed anything is followed by a scan: the whole point of
    /// copying into the home is that the home's watch indexes the copies.
    fn poll_copy(&mut self) {
        if self.copy.is_none() {
            return;
        }

        if let Some(copy) = self.copy.as_ref() {
            if let Some(progress) = copy.try_progress() {
                self.copy_progress = progress;
            }

            if !copy.is_finished() {
                return;
            }
        }

        let Some(copy) = self.copy.take() else {
            return;
        };
        let report = copy.join();
        self.last_import = Some(copy_message(&report, self.media_home()));

        if report.copied > 0 {
            self.start_scan();
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

        // The playlists come back with the tracks, so the sidebar and the
        // section are drawn from what the database last said rather than
        // querying it from a frame.
        self.playlists = match self.library.as_mut() {
            Some(library) => Playlists::new(library.repository_mut()).list()?,
            None => Vec::new(),
        };

        // A playlist that has been deleted — here or in another window — leaves
        // the section showing something that no longer exists.
        if let Some(open) = self.selected_playlist
            && !self.playlists.iter().any(|playlist| playlist.id == open)
        {
            self.selected_playlist = None;
            self.playlist_tracks.clear();
        }

        self.reload_playlist_tracks();

        tracing::debug!("library refreshed: {} tracks", self.browse.track_count());

        Ok(())
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

    /// Forget every row whose file has vanished.
    ///
    /// Returns how many rows went. A file that comes back is indexed again by
    /// the next scan, so this is housekeeping rather than a deletion.
    pub fn forget_missing(&mut self) -> usize {
        let removed = match self.library.as_ref() {
            Some(library) => match library.repository().remove_missing() {
                Ok(removed) => removed,
                Err(error) => {
                    self.library_error =
                        Some(format!("the missing files could not be removed: {error}"));
                    return 0;
                }
            },
            None => return 0,
        };

        if removed > 0
            && let Err(error) = self.refresh()
        {
            self.library_error = Some(error.to_string());
        }

        removed
    }

    /// Every track the library holds, for the "add the selection" flow.
    #[must_use]
    pub fn library_tracks(&self) -> &[Track] {
        self.browse.tracks()
    }

    /// One track of the library by its row identifier.
    #[must_use]
    pub fn track_of(&self, id: TrackId) -> Option<Track> {
        self.browse
            .tracks()
            .iter()
            .find(|track| track.id == Some(id))
            .cloned()
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
}
