use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use eframe::egui;

use crate::domain::LibrarySection;
use crate::media;
use crate::storage;
use playback::{Playback, UpNext};

use storage::LibraryData;

#[derive(Default)]
pub struct LibraryApp {
    pub selected_section: LibrarySection,
    pub search: String,
    pub music_files: Vec<PathBuf>,
    pub movie_files: Vec<PathBuf>,
    pub tv_files: Vec<PathBuf>,
    pub podcast_files: Vec<PathBuf>,
    pub scan_roots: Vec<PathBuf>,
    pub scan_progress: Arc<Mutex<media::ScanProgress>>,
    pub scanning: bool,
    pub queue: UpNext,
    pub show_queue: bool,
    pub playback: Playback,
    pub status: Option<String>,
    pub storage_error: Option<String>,
    pub dirty: bool,
}

impl LibraryApp {
    pub fn load() -> Self {
        let mut app = Self::default();
        match storage::load() {
            Ok(data) => {
                let (music, movies, tv, podcasts, queue_paths, scan_roots) = data.to_sections();
                app.music_files = music;
                app.movie_files = movies;
                app.tv_files = tv;
                app.podcast_files = podcasts;
                for path in queue_paths {
                    app.queue.enqueue(path);
                }
                app.scan_roots = scan_roots;
            }
            Err(error) => {
                app.storage_error = Some(error);
            }
        }
        app
    }

    fn files_for_section(&self, section: LibrarySection) -> &Vec<PathBuf> {
        match section {
            LibrarySection::Music => &self.music_files,
            LibrarySection::Movies => &self.movie_files,
            LibrarySection::TvShows => &self.tv_files,
            LibrarySection::Podcasts => &self.podcast_files,
        }
    }

    fn files_for_section_mut(&mut self, section: LibrarySection) -> &mut Vec<PathBuf> {
        match section {
            LibrarySection::Music => &mut self.music_files,
            LibrarySection::Movies => &mut self.movie_files,
            LibrarySection::TvShows => &mut self.tv_files,
            LibrarySection::Podcasts => &mut self.podcast_files,
        }
    }

    fn is_audio_section(section: LibrarySection) -> bool {
        matches!(section, LibrarySection::Music | LibrarySection::Podcasts)
    }

    fn start_scan(&mut self, ctx: eframe::egui::Context) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        {
            let mut progress = self.scan_progress.lock().unwrap();
            progress.scanning = true;
            progress.files_found = 0;
            progress.errors.clear();
            progress.result_files = None;
        }

        let roots = self.scan_roots.clone();
        let progress = self.scan_progress.clone();
        thread::spawn(move || {
            media::scan_folders_with_progress(&roots, progress.clone());
            ctx.request_repaint();
        });
    }

    fn poll_scan_result(&mut self) {
        if !self.scanning {
            return;
        }
        let scan_done = {
            let progress = self.scan_progress.lock().unwrap();
            !progress.scanning
        };
        if !scan_done {
            return;
        }

        self.scanning = false;
        let (result_files, errors) = {
            let mut progress = self.scan_progress.lock().unwrap();
            let files = progress.result_files.take();
            let errors = std::mem::take(&mut progress.errors);
            (files, errors)
        };

        if let Some(files) = result_files {
            let mut added_count = 0usize;
            for section in LibrarySection::ALL {
                let is_audio = Self::is_audio_section(section);
                let audio_exts = [
                    "mp3", "flac", "wav", "ogg", "oga", "m4a", "mp4", "aac", "opus",
                ];
                let video_exts = ["mp4", "mkv", "avi", "mov", "webm", "m4v"];
                let to_add: Vec<PathBuf> = {
                    let target = self.files_for_section(section);
                    let existing: HashSet<_> = target.iter().collect();
                    files
                        .iter()
                        .filter(|path| {
                            let ext = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            let matches = if is_audio {
                                audio_exts.contains(&ext.as_str())
                            } else {
                                video_exts.contains(&ext.as_str())
                            };
                            matches && !existing.contains(*path)
                        })
                        .cloned()
                        .collect()
                };
                added_count += to_add.len();
                let target = self.files_for_section_mut(section);
                target.extend(to_add);
            }
            self.dirty = true;
            if added_count > 0 {
                self.status = Some(format!("Scan complete: {} new file(s) found.", added_count));
            } else {
                self.status = Some("Scan complete: no new files found.".to_string());
            }
        }

        if !errors.is_empty() {
            self.status = Some(format!("Scan errors: {}", errors.join("; ")));
        }
    }

    fn save_if_needed(&mut self) {
        if !self.dirty {
            return;
        }
        let data = LibraryData::from_sections(
            &self.music_files,
            &self.movie_files,
            &self.tv_files,
            &self.podcast_files,
            self.queue.pending(),
            &self.scan_roots,
        );
        match storage::save(&data) {
            Ok(()) => {
                self.dirty = false;
                self.storage_error = None;
            }
            Err(error) => {
                self.storage_error = Some(error);
            }
        }
    }

    fn add_files_to_section(&mut self, section: LibrarySection) {
        let filter_exts: &[&str] = if Self::is_audio_section(section) {
            &[
                "mp3", "flac", "wav", "ogg", "oga", "m4a", "mp4", "aac", "opus",
            ]
        } else {
            &["mp4", "mkv", "avi", "mov", "webm", "m4v"]
        };
        let files = rfd::FileDialog::new()
            .add_filter("Media", filter_exts)
            .pick_files();
        if let Some(files) = files {
            let target = self.files_for_section_mut(section);
            let existing: HashSet<_> = target.iter().collect();
            let added: Vec<_> = files
                .into_iter()
                .filter(|path| !existing.contains(path))
                .collect();
            let count = added.len();
            target.extend(added);
            self.dirty = true;
            self.status = Some(format!("Added {count} file(s)."));
        }
    }

    fn add_folder_to_section(&mut self, section: LibrarySection) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            // Track this folder as a scan root
            if !self.scan_roots.contains(&folder) {
                self.scan_roots.push(folder.clone());
            }
            match media::scan_folder(&folder) {
                Ok(scanned) => {
                    let target = self.files_for_section_mut(section);
                    let existing: HashSet<_> = target.iter().collect();
                    let added: Vec<_> = scanned
                        .into_iter()
                        .filter(|path| !existing.contains(path))
                        .collect();
                    let count = added.len();
                    target.extend(added);
                    self.dirty = true;
                    self.status = Some(format!("Added {count} file(s) from folder."));
                }
                Err(error) => {
                    self.status = Some(error);
                }
            }
        }
    }

    fn play_track_for_section(&mut self, section: LibrarySection, index: usize) {
        let files = self.files_for_section(section);
        if let Some(path) = files.get(index).cloned() {
            if let Err(error) = self.playback.play_file(path.clone()) {
                self.status = Some(error.to_string());
            } else {
                self.queue.play_now(path);
                self.dirty = true;
                self.status = None;
            }
        }
    }

    fn skip_track(&mut self, forward: bool) {
        let path = if forward {
            self.queue.next()
        } else {
            self.queue.previous()
        };
        if let Some(path) = path {
            if let Err(error) = self.playback.play_file(path) {
                self.status = Some(error.to_string());
                self.queue.set_current(None);
            } else {
                self.status = None;
            }
        }
    }

    fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("LIBRARY").small().weak());
        ui.add_space(8.0);

        for section in LibrarySection::ALL {
            let selected = self.selected_section == section;
            if ui.selectable_label(selected, section.label()).clicked() {
                self.selected_section = section;
            }
        }

        ui.add_space(24.0);
        ui.label(
            egui::RichText::new("PLAYLISTS & COLLECTIONS")
                .small()
                .weak(),
        );
        ui.add_space(8.0);
        ui.label(egui::RichText::new("No playlists yet").weak());

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.separator();
            ui.label(egui::RichText::new("No device connected").small().weak());
            ui.label(egui::RichText::new("DEVICE").small().weak());
        });
    }

    fn show_empty_state(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(28, 32))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.heading(self.selected_section.empty_message());
                    ui.add_space(8.0);
                    ui.label("This section is not connected to a media library yet.");
                    ui.add_space(30.0);
                });
            });
    }

    fn show_media_section(&mut self, ui: &mut egui::Ui, section: LibrarySection) {
        let files = self.files_for_section(section).clone();
        let is_audio = Self::is_audio_section(section);

        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} files", files.len()));
            if ui.button("Add files…").clicked() {
                self.add_files_to_section(section);
            }
            if ui.button("Add folder…").clicked() {
                self.add_folder_to_section(section);
            }
            if section == LibrarySection::Music && ui.button("Backup library…").clicked() {
                let data = LibraryData::from_sections(
                    &self.music_files,
                    &self.movie_files,
                    &self.tv_files,
                    &self.podcast_files,
                    self.queue.pending(),
                    &self.scan_roots,
                );
                match storage::backup(&data) {
                    Ok(path) => {
                        self.status = Some(format!("Backup saved to {}", path.display()));
                    }
                    Err(error) => {
                        self.status = Some(error);
                    }
                }
            }
        });
        ui.add_space(12.0);

        if files.is_empty() {
            self.show_empty_state(ui);
            if is_audio {
                ui.label("Playback supports local MP3, FLAC, WAV, Ogg/Vorbis, and MP4/M4A audio in this build.");
            } else {
                ui.label("Video playback is not yet implemented.");
            }
            return;
        }

        let visible = crate::domain::filter_tracks(&files, &self.search);

        if visible.is_empty() {
            ui.label("No files match this search.");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (_index, path) in visible {
                ui.push_id(path.as_os_str(), |ui| {
                    ui.horizontal(|ui| {
                        let title = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown file");
                        ui.label(title);
                        if is_audio
                            && ui.button("Play now").clicked()
                            && let Some(real_index) = files.iter().position(|p| p == &path)
                        {
                            self.play_track_for_section(section, real_index);
                        }
                        if is_audio && ui.button("Add to Up Next").clicked() {
                            self.queue.enqueue(path.clone());
                            self.dirty = true;
                        }
                    });
                    ui.separator();
                });
            }
        });
    }

    fn show_player_controls(&mut self, ui: &mut egui::Ui) {
        let current = self.playback.current_path().cloned();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(current.is_some(), |ui| {
                if ui.button("⏮").clicked() {
                    self.skip_track(false);
                }
                if ui
                    .button(if self.playback.paused() { "▶" } else { "⏸" })
                    .clicked()
                {
                    self.playback.toggle();
                }
                if ui.button("⏹").clicked() {
                    self.playback.stop();
                    self.queue.set_current(None);
                }
                if ui.button("⏭").clicked() {
                    self.skip_track(true);
                }
            });

            ui.separator();
            let title = current
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("Nothing playing");
            ui.label(title);
            if ui
                .button(format!("Up Next ({})", self.queue.pending().len()))
                .clicked()
            {
                self.show_queue = !self.show_queue;
            }
            ui.add_space(10.0);

            let duration = self.playback.duration();
            let duration_secs = duration.map_or(0.0, |value| value.as_secs_f64() as f32);
            let mut position = self.playback.position().as_secs_f64() as f32;
            if let Some(duration) = duration {
                let response = ui.add_sized(
                    [ui.available_width().max(80.0), 18.0],
                    egui::Slider::new(&mut position, 0.0..=duration_secs).show_value(false),
                );
                if (response.drag_stopped() || response.clicked())
                    && let Err(error) = self.playback.seek(Duration::from_secs_f32(position))
                {
                    self.status = Some(error.to_string());
                }
                ui.label(crate::domain::format_time(Duration::from_secs_f32(
                    position,
                )));
                ui.label("/");
                ui.label(crate::domain::format_time(duration));
            } else if current.is_some() {
                ui.label(crate::domain::format_time(self.playback.position()));
            }
        });

        if let Some(error) = self.playback.error() {
            ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
        }

        if self.show_queue {
            let mut open = self.show_queue;
            egui::Window::new("Up Next")
                .open(&mut open)
                .resizable(true)
                .show(ui.ctx(), |ui| self.show_queue_contents(ui));
            self.show_queue = open;
        }
    }

    fn show_queue_contents(&mut self, ui: &mut egui::Ui) {
        if self.queue.pending().is_empty() {
            ui.label("Up Next is empty. Add tracks from your music list.");
        } else {
            let items: Vec<_> = self.queue.pending().iter().cloned().collect();
            for (index, path) in items.into_iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown audio file"),
                    );
                    if ui.small_button("Play").clicked() {
                        self.play_queued_track(index);
                    }
                    if ui.small_button("↑").clicked() && index > 0 {
                        self.queue.move_item(index, index - 1);
                        self.dirty = true;
                    }
                    if ui.small_button("↓").clicked() {
                        self.queue.move_item(index, index + 1);
                        self.dirty = true;
                    }
                    if ui.small_button("Remove").clicked() {
                        self.queue.remove(index);
                        self.dirty = true;
                    }
                });
            }
        }
        if ui.button("Clear Up Next").clicked() {
            self.queue.clear();
            self.dirty = true;
        }
    }

    fn play_queued_track(&mut self, index: usize) {
        let Some(path) = self.queue.pending().get(index).cloned() else {
            return;
        };
        if let Err(error) = self.playback.play_file(path.clone()) {
            self.status = Some(error.to_string());
        } else {
            self.queue.play_at(index);
            self.dirty = true;
            self.status = None;
        }
    }
}

impl eframe::App for LibraryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());
        if self.playback.current_path().is_some() || self.scanning {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }

        // Handle drag-and-drop of files and folders
        let dropped = ui.ctx().input(|input| input.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let paths: Vec<PathBuf> = dropped
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect();
            // Track dropped folders as scan roots
            for path in &paths {
                if path.is_dir() && !self.scan_roots.contains(path) {
                    self.scan_roots.push(path.clone());
                }
            }
            let scanned = media::collect_dropped(&paths);
            let files = self.files_for_section_mut(self.selected_section);
            let existing: HashSet<_> = files.iter().collect();
            let added: Vec<_> = scanned
                .into_iter()
                .filter(|path| !existing.contains(path))
                .collect();
            let count = added.len();
            files.extend(added);
            if count > 0 {
                self.dirty = true;
                self.status = Some(format!("Added {count} file(s) from drop."));
            } else {
                self.status = Some("No supported media files found in dropped items.".to_string());
            }
        }

        if self.playback.finished() {
            self.skip_track(true);
            if self.playback.finished() {
                self.playback.stop();
                self.queue.set_current(None);
            }
        }

        self.poll_scan_result();

        egui::Panel::top("top_bar")
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(20, 14)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("iCraftMusic");
                    ui.add_space(20.0);
                    ui.add_sized(
                        [ui.available_width().min(420.0), 32.0],
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search your library"),
                    );
                    ui.add_space(8.0);
                    if self.scanning {
                        ui.label(format!(
                            "Scanning... {} files found",
                            self.scan_progress.lock().unwrap().files_found
                        ));
                    } else if ui.button("Refresh library").clicked() {
                        if !self.scan_roots.is_empty() {
                            self.start_scan(ui.ctx().clone());
                        } else {
                            self.status =
                                Some("No folders to refresh. Add a folder first.".to_string());
                        }
                    }
                });
            });

        egui::Panel::bottom("player")
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(20, 12)))
            .show(ui, |ui| self.show_player_controls(ui));

        egui::Panel::left("navigation")
            .resizable(false)
            .default_size(190.0)
            .size_range(150.0..=230.0)
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(12, 16)))
            .show(ui, |ui| self.show_navigation(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::same(28)))
            .show(ui, |ui| {
                ui.heading(self.selected_section.label());
                ui.add_space(16.0);

                self.show_media_section(ui, self.selected_section);

                if let Some(status) = &self.status {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_RED));
                }
                if let Some(error) = &self.storage_error {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                }
            });

        self.save_if_needed();
    }
}
