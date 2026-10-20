use crate::audio::RepeatMode;
use crate::library::edit::Outcome;

use super::*;

use crate::audio::AudioEngine;
use crate::library::{ClickMode, ScanReport};

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

    app.set_section(NavSection::Playlists);
    assert_eq!(app.section(), NavSection::Playlists);
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
            artwork: None,
        })
    );
}

#[test]
fn the_bar_takes_its_cover_from_the_library_row_of_the_playing_track() {
    let audio = crate::fixtures::scratch_file("cover.wav", &crate::fixtures::silent_wav(64));
    let cover = crate::fixtures::scratch_file("cover.png", &crate::fixtures::png());
    let library = crate::library::Library::in_memory(crate::fixtures::scratch("app-cover-art"))
        .expect("an in-memory library");
    let mut app = resolve(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        library,
    );

    let mut track = Track::from_path(&audio);
    track.id = Some(TrackId(1));
    track.title = "Roygbiv".to_owned();
    track.artwork_path = Some(cover.clone());
    app.library_mut()
        .expect("a library")
        .repository_mut()
        .upsert_track(&track)
        .expect("the track should be stored");
    app.refresh().expect("the library should refresh");

    // The queue entry carries the path the library indexed, which is what
    // the bar looks the cover up by.
    let path = std::fs::canonicalize(&audio).unwrap_or(audio);
    app.open_path(&path);

    assert_eq!(
        app.now_playing().and_then(|track| track.artwork),
        Some(cover),
        "the playing track's cover is the library's"
    );
}

#[test]
fn a_track_the_library_does_not_know_has_no_cover() {
    let mut app = imported();
    let known = app.browse().tracks()[0].path.clone();

    app.open_path(Path::new("/music/nowhere.wav"));

    assert_eq!(
        app.now_playing().and_then(|track| track.artwork),
        None,
        "a track outside the library has no row to take a cover from"
    );
    assert!(
        app.browse().tracks().iter().any(|row| row.path == known),
        "and the library is left alone"
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

#[test]
fn an_application_built_without_a_library_has_none_invented_for_it() {
    // A constructor that silently supplies an in-memory database makes the
    // application open two libraries at startup: the scratch one, whose
    // migrations are then logged as though the user's own file had been
    // upgraded, and the real one. No library being there is the honest
    // state of an application that has not opened one yet — and it is what
    // keeps `with_playback` from being a way to "have a library" by
    // accident.
    let app = app();

    assert!(
        app.library().is_none(),
        "a library was invented by the constructor"
    );
    assert!(
        app.library_error().is_none(),
        "having no library is not a failure: {:?}",
        app.library_error()
    );
}

#[test]
fn a_folder_chosen_into_the_field_is_added_and_scanned() {
    // A folder is added to a library, so this needs one; `app()` is an
    // application with no library at all.
    let mut app = imported();
    let folder = crate::fixtures::scratch("chosen-folder");

    *app.folder_input_mut() = folder.display().to_string();
    let added = app.import_folder_input();

    assert!(added, "the folder is new to the library");
    assert_eq!(
        app.library_folders().len(),
        1,
        "the two tracks `imported()` stores are rows, not watched folders, \
         so the folder chosen here is the first one"
    );
    assert_eq!(
        app.folder_input(),
        "",
        "the field resets, so the next folder starts from an empty line"
    );
    assert!(
        app.is_scanning(),
        "a folder is scanned as soon as it is chosen"
    );
}

#[test]
fn a_dropped_folder_asks_the_add_question_instead_of_adding_itself() {
    #[derive(Debug)]
    struct Dropped {
        path: PathBuf,
    }

    impl egui::DroppedFile for Dropped {
        fn path(&self) -> &Path {
            &self.path
        }

        fn bytes(&self) -> std::result::Result<Vec<u8>, String> {
            std::fs::read(&self.path).map_err(|error| error.to_string())
        }
    }

    let mut app = empty();
    let folder = crate::fixtures::scratch("dropped-folder");

    let ctx = egui::Context::default();
    let dropped = egui::RawInput {
        dropped_files: vec![std::sync::Arc::new(Dropped {
            path: folder.clone(),
        })],
        ..Default::default()
    };
    let mut run = ctx.run_ui(dropped, |_ui| app.poll_dropped_files(&ctx));
    run.textures_delta.clear();

    assert_eq!(
        app.pending_add(),
        Some(PendingAdd::Folders {
            paths: vec![folder]
        }),
        "a dropped folder is added through the same dialog as a chosen one"
    );
    assert!(app.library_folders().is_empty(), "dropping is not adding");
}

#[test]
fn a_mixed_drop_asks_its_two_questions_one_at_a_time() {
    #[derive(Debug)]
    struct Dropped {
        path: PathBuf,
    }

    impl egui::DroppedFile for Dropped {
        fn path(&self) -> &Path {
            &self.path
        }

        fn bytes(&self) -> std::result::Result<Vec<u8>, String> {
            std::fs::read(&self.path).map_err(|error| error.to_string())
        }
    }

    let mut app = empty();
    let folder = crate::fixtures::scratch("mixed-folder");
    let file = crate::fixtures::scratch_file("mixed-file.wav", b"RIFF");

    let ctx = egui::Context::default();
    let dropped = egui::RawInput {
        dropped_files: vec![
            std::sync::Arc::new(Dropped {
                path: folder.clone(),
            }),
            std::sync::Arc::new(Dropped { path: file.clone() }),
        ],
        ..Default::default()
    };
    let mut run = ctx.run_ui(dropped, |_ui| app.poll_dropped_files(&ctx));
    run.textures_delta.clear();

    assert_eq!(
        app.pending_add(),
        Some(PendingAdd::Folders {
            paths: vec![folder]
        }),
        "folders are asked about first"
    );

    app.cancel_add();

    assert_eq!(
        app.pending_add(),
        Some(PendingAdd::Files { paths: vec![file] }),
        "the second question arrives when the first is answered"
    );
}

#[test]
fn the_add_dialog_answered_with_add_watches_what_it_chose() {
    let mut app = empty();
    let file = crate::fixtures::scratch_file("loose-add.wav", &crate::fixtures::silent_wav(8000));

    app.ask_to_add(PendingAdd::Files {
        paths: vec![file.clone()],
    });
    assert!(app.pending_add().is_some(), "the dialog is asking");

    app.add_paths_in_place();

    assert!(app.pending_add().is_none(), "the answer closes the dialog");
    assert!(
        app.library_folders().contains(&file),
        "the file is watched exactly where it lives"
    );
    assert!(app.is_scanning(), "watching is followed by a scan");
}

#[test]
fn the_add_dialog_answered_with_cancel_adds_nothing() {
    let mut app = empty();

    app.ask_to_add(PendingAdd::Folders {
        paths: vec![PathBuf::from("/music/nowhere")],
    });
    app.cancel_add();

    assert!(app.pending_add().is_none());
    assert!(
        app.library_folders().is_empty(),
        "Cancel is not a hidden way to watch a folder"
    );
}

#[test]
fn an_empty_choice_asks_no_question() {
    let mut app = empty();

    app.ask_to_add(PendingAdd::Files { paths: Vec::new() });

    assert!(app.pending_add().is_none(), "nothing chosen, nothing asked");
}

#[test]
fn the_add_dialog_answered_with_copy_lands_the_file_in_the_home_and_indexes_it() {
    let mut app = empty();
    let home = crate::fixtures::scratch("copy-into-home");
    app.config.set_media_home(home.clone());
    app.library_mut()
        .expect("a library")
        .set_folders(vec![home.clone()]);

    let source = crate::fixtures::scratch_file("copy-me.wav", &crate::fixtures::silent_wav(8000));

    app.ask_to_add(PendingAdd::Files {
        paths: vec![source.clone()],
    });
    app.copy_paths_into_home();

    assert!(app.pending_add().is_none());
    assert!(app.is_copying(), "the copy runs on its worker thread");

    // The copy finishes on its thread and is followed by the scan that
    // indexes what landed; polling is the same one the frames do.
    for _ in 0..4000 {
        app.poll_library();
        if !app.is_copying()
            && !app.is_scanning()
            && app.last_import().is_some_and(|m| m.contains("Indexed"))
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert!(!app.is_copying() && !app.is_scanning(), "both jobs ended");
    assert!(
        home.join("copy-me.wav").is_file(),
        "the copy is in the home"
    );
    assert!(source.is_file(), "the original is where it always was");
    assert_eq!(
        app.browse().track_count(),
        1,
        "the copy is what the scan indexed"
    );
}

#[test]
fn the_indexing_widget_turns_a_running_scan_into_one_status() {
    let mut app = empty();

    assert_eq!(
        app.indexing_status(),
        None,
        "with nothing running the bar has nothing to say"
    );

    // The scan as spawned: the walk has not reported back yet, so the
    // widget knows only that work is happening — no total, no numbers.
    app.ask_to_add(PendingAdd::Folders {
        paths: vec![crate::fixtures::scratch("index-status")],
    });
    app.add_paths_in_place();
    assert!(app.is_scanning(), "the job is on its worker thread");
    assert_eq!(
        app.indexing_status(),
        Some(IndexingStatus {
            done: 0,
            total: 0,
            left_out: 0,
        }),
        "before the walk has counted, the bar claims no total"
    );

    // Mid-job, as the walk's report and the reads since have come in:
    // both kinds of left-out file — unreadable formats and failures —
    // count together as the one honest "left out".
    app.scan_progress = ScanProgress {
        discovered: 42,
        processed: 42,
        failed: 2,
        total: 118,
        unreadable: 3,
        current: None,
    };
    assert_eq!(
        app.indexing_status(),
        Some(IndexingStatus {
            done: 42,
            total: 118,
            left_out: 5,
        }),
        "the bar's line is the scan's own counts"
    );
}

#[test]
fn the_indexing_widget_reads_a_copy_as_the_same_job() {
    let mut app = empty();
    let home = crate::fixtures::scratch("index-status-copy");
    app.config.set_media_home(home.clone());
    app.library_mut()
        .expect("a library")
        .set_folders(vec![home.clone()]);
    let source =
        crate::fixtures::scratch_file("index-copy.wav", &crate::fixtures::silent_wav(8000));

    app.ask_to_add(PendingAdd::Files {
        paths: vec![source],
    });
    app.copy_paths_into_home();
    assert!(app.is_copying(), "the copy is on its worker thread");

    // The copy reports through the same widget with the same words: the
    // handover from copy to the scan that indexes is not the bar's affair.
    app.copy_progress = CopyProgress {
        copied: 7,
        total: 20,
        failed: 1,
        current: None,
    };
    assert_eq!(
        app.indexing_status(),
        Some(IndexingStatus {
            done: 7,
            total: 20,
            left_out: 1,
        }),
        "a copy wears the scan's face"
    );
}

#[test]
fn the_updater_says_what_it_knows_as_one_status() {
    let mut app = empty();
    // The tests say what kind of run they are, rather than inheriting
    // whether the machine that runs `cargo test` happens to be an
    // AppImage itself.
    app.set_app_image(None);

    assert_eq!(app.update_status(), None, "nothing asked, nothing shown");

    app.set_release_update(Update::UpToDate);
    assert_eq!(app.update_status(), None, "being up to date is not news");

    // A release carrying an image, but this build is not running from an
    // AppImage: the gear row may point at the release, and must not
    // promise a swap.
    app.set_release_update(Update::Available {
        version: "v9.9.9".to_owned(),
        url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
        asset: Some("https://e/iCraftMusic-9.9.9-x86_64.AppImage".to_owned()),
    });
    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Available {
            version: "v9.9.9".to_owned(),
            can_install: false,
        }),
        "a non-AppImage run was offered a swap it cannot perform"
    );

    // The same release while the process knows its own image file: now
    // the swap is on the table.
    app.set_app_image(Some(std::env::temp_dir().join("iCraftMusic-test.AppImage")));
    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Available {
            version: "v9.9.9".to_owned(),
            can_install: true,
        }),
        "an AppImage run was not offered the update it can install"
    );
}

#[test]
fn the_installer_outranks_the_release_news() {
    let mut app = empty();
    app.set_release_update(Update::Available {
        version: "v9.9.9".to_owned(),
        url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
        asset: Some("https://e/iCraftMusic-9.9.9-x86_64.AppImage".to_owned()),
    });

    // Once bytes are moving, that is the news — not the offer they came from.
    app.set_install(
        Some("v9.9.9"),
        Some(crate::update::Install::Running { done: 5, total: 10 }),
    );
    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Downloading {
            version: "v9.9.9".to_owned(),
            done: 5,
            total: 10,
        }),
        "a running download was hidden behind the release news"
    );

    app.set_install(
        Some("v9.9.9"),
        Some(crate::update::Install::Installed {
            version: "v9.9.9".to_owned(),
        }),
    );
    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Installed {
            version: "v9.9.9".to_owned(),
        }),
        "an installed update still reads as merely available"
    );

    // A failed attempt says so, once, rather than silently falling back to
    // the offer and letting the user wonder whether the last one worked.
    app.set_install(
        None,
        Some(crate::update::Install::Failed {
            reason: "the download stopped".to_owned(),
        }),
    );
    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Failed {
            reason: "the download stopped".to_owned(),
        }),
        "the failure hid behind the news"
    );
}

#[test]
fn an_update_is_started_only_where_a_swap_is_possible() {
    let mut app = empty();
    app.set_app_image(None);
    app.set_release_update(Update::Available {
        version: "v9.9.9".to_owned(),
        url: "https://github.com/cmspeedau/iCraftMusic/releases/tag/v9.9.9".to_owned(),
        asset: Some("https://e/iCraftMusic-9.9.9-x86_64.AppImage".to_owned()),
    });

    // Not running from an AppImage: pressing the row must not start a
    // worker — the menu offers this release as a page instead, and a
    // download with nowhere to land is bytes for nothing.
    app.start_update();
    assert!(
        !app.update_job_running(),
        "a worker started with no AppImage to swap into"
    );
    assert!(app.app_image_path().is_none());
}

#[test]
fn a_pending_check_is_the_updater_checking() {
    let mut app = empty();
    app.set_check_pending();

    assert_eq!(
        app.update_status(),
        Some(crate::app::UpdateStatus::Checking)
    );
    assert!(app.update_job_running(), "a pending check is a job");
}

#[test]
fn a_path_that_is_not_a_folder_is_refused_with_a_reason() {
    let mut app = app();

    *app.folder_input_mut() = "/music/no-such-folder".to_owned();
    let added = app.import_folder_input();

    assert!(!added);
    assert!(
        app.library_folders().is_empty(),
        "nothing is watched that cannot be read"
    );
    let message = app.library_error().expect("a reason the user can act on");
    assert!(message.contains("not a folder"), "{message}");
    assert_eq!(
        app.folder_input(),
        "/music/no-such-folder",
        "a path that added nothing stays put so it can be corrected"
    );
}

#[test]
fn the_import_line_names_the_formats_that_could_not_be_read() {
    let report = ScanReport {
        scanned: 4,
        unreadable: vec![PathBuf::from("/music/a.ape"), PathBuf::from("/music/b.wma")],
        ..ScanReport::default()
    };
    let summary = ImportSummary {
        scan: report,
        inserted: 4,
        ..ImportSummary::default()
    };

    let message = import_message(&summary, false);

    assert!(message.contains("Indexed 4 tracks"), "{message}");
    assert!(
        message.contains("2 in formats this player cannot read"),
        "{message}"
    );
    assert!(message.contains("ape, wma"), "{message}");
}

#[test]
fn an_edit_is_saved_into_the_library_without_touching_the_file() {
    let mut app = edited();
    let path = app.editor().expect("an editor").draft().path().clone();
    let before = std::fs::read(&path).expect("the fixture is readable");

    *app.editor_mut().expect("an editor").draft_mut().title_mut() = "Olson".to_owned();
    app.save_editor();

    let id = app.editor().expect("an editor").track_id();
    let stored = app.track_of(id).expect("the track");
    assert_eq!(stored.title, "Olson");
    assert!(
        stored.tags_dirty,
        "the library says the file has not been told"
    );
    assert_eq!(
        std::fs::read(&path).expect("still readable"),
        before,
        "saving to the library must not rewrite the file"
    );
    assert_eq!(
        app.editor().expect("an editor").outcome(),
        Some(&Outcome::Saved)
    );
}

#[test]
fn writing_to_the_file_puts_the_edit_into_its_tags() {
    let mut app = edited();
    let path = app.editor().expect("an editor").draft().path().clone();

    *app.editor_mut().expect("an editor").draft_mut().title_mut() = "Olson".to_owned();
    app.write_editor_tags();

    let tags = crate::library::metadata::read(&path).expect("the file should still read");
    assert_eq!(tags.title.as_deref(), Some("Olson"));
    assert_eq!(
        app.editor().expect("an editor").outcome(),
        Some(&Outcome::Written)
    );

    let id = app.editor().expect("an editor").track_id();
    assert!(!app.track_of(id).expect("the track").tags_dirty);
}

#[test]
fn a_missing_title_stops_the_edit_and_says_which_field_is_wrong() {
    let mut app = edited();
    *app.editor_mut().expect("an editor").draft_mut().title_mut() = "   ".to_owned();

    app.save_editor();

    let outcome = app.editor().expect("an editor").outcome().expect("outcome");
    assert!(outcome.is_failure());
    assert!(outcome.message().contains("Title"), "{}", outcome.message());

    let id = app.editor().expect("an editor").track_id();
    assert_eq!(
        app.track_of(id).expect("the track").title,
        "Roygbiv",
        "nothing was written"
    );
}

#[test]
fn saving_without_changing_anything_does_not_mark_the_row_as_edited() {
    let mut app = edited();

    app.save_editor();

    let id = app.editor().expect("an editor").track_id();
    assert!(!app.track_of(id).expect("the track").tags_dirty);
    assert_eq!(
        app.editor().expect("an editor").outcome(),
        Some(&Outcome::Saved)
    );
}

#[test]
fn a_file_that_is_not_on_disk_cannot_be_written_into() {
    let mut app = edited();
    let id = app.editor().expect("an editor").track_id();
    let path = app.editor().expect("an editor").draft().path().clone();
    let before = std::fs::read(&path).expect("the fixture is readable");

    // A row can outlive its file: the scan flags it and keeps it.
    app.library_mut()
        .expect("a library")
        .repository()
        .mark_missing(&path, true)
        .expect("the flag");
    app.refresh().expect("refresh");
    assert!(app.track_of(id).expect("the track").missing);

    *app.editor_mut().expect("an editor").draft_mut().title_mut() = "Olson".to_owned();
    app.write_editor_tags();

    let outcome = app.editor().expect("an editor").outcome().expect("outcome");
    assert!(outcome.is_failure());
    assert!(
        outcome.message().contains("not on disk"),
        "{}",
        outcome.message()
    );
    assert_eq!(
        std::fs::read(&path).expect("still readable"),
        before,
        "nothing was written into the file"
    );
}

#[test]
fn choosing_an_image_caches_it_and_still_writes_nothing_to_the_file() {
    let mut app = edited();
    let path = app.editor().expect("an editor").draft().path().clone();
    let image = crate::fixtures::scratch_file("chosen.png", &crate::fixtures::png());
    let before = std::fs::read(&path).expect("the fixture is readable");

    *app.editor_mut()
        .expect("an editor")
        .draft_mut()
        .artwork_input_mut() = image.display().to_string();
    app.use_editor_artwork();

    let cached = app
        .editor()
        .expect("an editor")
        .draft()
        .artwork()
        .expect("the cover the editor now shows")
        .clone();
    assert!(cached.is_file(), "the picture was cached at {cached:?}");
    assert_eq!(
        app.editor().expect("an editor").draft().replacement(),
        Some(&cached)
    );
    assert_eq!(
        app.editor().expect("an editor").draft().artwork_input(),
        "",
        "the field resets after a success"
    );

    app.save_editor();
    let id = app.editor().expect("an editor").track_id();
    assert_eq!(
        app.track_of(id).expect("the track").artwork_path,
        Some(cached)
    );
    assert_eq!(
        std::fs::read(&path).expect("still readable"),
        before,
        "choosing a cover is not yet writing to the file"
    );
}

#[test]
fn a_file_that_is_not_an_image_is_refused_as_a_cover() {
    let mut app = edited();
    let text = crate::fixtures::scratch_file("notes.png", b"this is not an image");

    *app.editor_mut()
        .expect("an editor")
        .draft_mut()
        .artwork_input_mut() = text.display().to_string();
    app.use_editor_artwork();

    let outcome = app.editor().expect("an editor").outcome().expect("outcome");
    assert!(outcome.is_failure());
    assert!(outcome.message().contains("image"), "{}", outcome.message());
    assert!(app.editor().expect("an editor").draft().artwork().is_none());
}

#[test]
fn choosing_a_cover_that_is_not_there_reports_the_failure() {
    let mut app = edited();

    *app.editor_mut()
        .expect("an editor")
        .draft_mut()
        .artwork_input_mut() = "/music/no-such-cover.png".to_owned();
    app.use_editor_artwork();

    assert!(
        app.editor()
            .expect("an editor")
            .outcome()
            .expect("outcome")
            .is_failure()
    );
}

#[test]
fn an_empty_cover_field_asks_for_one_instead_of_failing_silently() {
    let mut app = edited();

    app.use_editor_artwork();

    let outcome = app.editor().expect("an editor").outcome().expect("outcome");
    assert!(outcome.is_failure());
    assert!(outcome.message().contains("path"), "{}", outcome.message());
}

#[test]
fn the_editor_opens_on_a_single_selection_and_not_on_several() {
    let mut app = imported();
    let visible = app.browse().visible_track_ids();

    assert!(
        !app.edit_selection(),
        "nothing is selected, so there is nothing to edit"
    );

    select(&mut app, visible[0]);
    assert!(app.edit_selection());
    assert_eq!(app.editor().map(TrackEditor::track_id), Some(visible[0]));
    app.close_editor();

    app.browse_mut()
        .selection_mut()
        .apply(&visible, visible[1], ClickMode::Toggle);
    assert!(!app.edit_selection(), "two tracks have two sets of tags");
}

#[test]
fn forgetting_the_missing_rows_leaves_the_rest_of_the_library_alone() {
    let mut app = imported();
    let visible = app.browse().visible_track_ids();
    let gone = visible[0];

    app.library_mut()
        .expect("a library")
        .repository()
        .mark_missing("/music/1.wav", true)
        .expect("the flag");
    app.refresh().expect("refresh");
    assert_eq!(app.browse().missing_count(), 1);

    app.ask_to_forget_missing();
    assert!(app.pending_confirmation().is_some());
    app.confirm_pending();

    assert_eq!(app.browse().track_count(), 1);
    assert!(
        app.browse()
            .tracks()
            .iter()
            .all(|track| track.id != Some(gone)),
        "the row that was gone is the one that went"
    );
}

/// An application with a real audio file indexed and its editor open.
///
/// The file is a silent WAV, so the paths that write tags have something
/// real to write into; the library row is the one a scan would have stored.
fn edited() -> MusicCraftApp {
    let path = crate::fixtures::scratch_file("edit.wav", &crate::fixtures::silent_wav(64));
    let library = crate::library::Library::in_memory(path.with_extension("artwork"))
        .expect("an in-memory library");
    let mut app = resolve(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        library,
    );

    let metadata = std::fs::metadata(&path).expect("the fixture should exist");
    let mut track = Track::from_path(&path);
    track.id = Some(TrackId(1));
    track.title = "Roygbiv".to_owned();
    track.artist = Some("Boards of Canada".to_owned());
    track.size_bytes = Some(metadata.len());
    track.modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX));

    app.library_mut()
        .expect("a library")
        .repository_mut()
        .upsert_track(&track)
        .expect("the track should be stored");
    app.refresh().expect("the library should refresh");
    assert!(app.open_editor(TrackId(1)));

    app
}

/// An application whose library holds two rows keyed by their paths.
fn imported() -> MusicCraftApp {
    let library =
        crate::library::Library::in_memory(std::env::temp_dir().join("musiccraft-app-artwork"))
            .expect("an in-memory library");
    let mut app = resolve(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        library,
    );

    for (id, path) in [(1, "/music/1.wav"), (2, "/music/2.wav")] {
        let mut track = Track::from_path(path);
        track.id = Some(TrackId(id));
        track.title = format!("Track {id}");
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the track should be stored");
    }

    app.refresh().expect("the library should refresh");

    app
}

/// Select one row of the library table, as a click would.
fn select(app: &mut MusicCraftApp, id: TrackId) {
    let visible = app.browse().visible_track_ids();
    app.browse_mut()
        .selection_mut()
        .apply(&visible, id, crate::library::ClickMode::Replace);
}

#[test]
fn the_output_device_is_only_remembered_once_it_has_opened() {
    let mut app = app();

    let switched = app.set_output_device(Some("nonexistent-host:gone".to_owned()));

    assert!(!switched, "that device is not on this machine");
    assert_eq!(
        app.chosen_output_device(),
        None,
        "a device that could not be opened must not become the preference"
    );
    let error = app.settings_error().expect("the failure is reported");
    assert!(error.contains("nonexistent-host:gone"), "{error}");

    // Reading the device list again is what a user does after plugging
    // something in, and it clears the report.
    app.rescan_output_devices();
    assert!(app.settings_error().is_none());
}

#[test]
fn a_refused_output_device_change_dirts_nothing() {
    let mut app = app();

    // Nothing to switch to on a device-less player, so the failure path is
    // what is exercised: the flag must stay put, because a settings write
    // that lies about what happened is worse than no write at all.
    let switched = app.set_output_device(Some("nonexistent-host:gone".to_owned()));

    assert!(!switched);
    assert!(!app.config_dirty, "a refused change dirties nothing");
}

#[test]
fn forgetting_a_watched_folder_leaves_the_music_alone() {
    let folder = crate::fixtures::scratch("forget-folder");
    std::fs::write(folder.join("Roygbiv.wav"), crate::fixtures::silent_wav(32))
        .expect("a file to index");
    let mut app = imported();
    let before = app.browse().track_count();

    assert!(app.add_library_folder(&folder));
    assert_eq!(app.library_folders().len(), 1);

    assert!(app.remove_library_folder(&folder));

    assert!(app.library_folders().is_empty(), "it is no longer watched");
    assert_eq!(
        app.browse().track_count(),
        before,
        "the tracks it contributed stay in the library"
    );
    assert!(
        folder.join("Roygbiv.wav").is_file(),
        "…and their files stay on disk"
    );
    assert!(
        !app.remove_library_folder(&folder),
        "forgetting it twice changes nothing"
    );
}

#[test]
fn a_rescan_of_the_same_folder_does_not_duplicate_its_tracks() {
    let folder = crate::fixtures::scratch("app-rescan");
    std::fs::write(folder.join("Roygbiv.mp3"), b"").expect("a file to index");
    std::fs::write(folder.join("Xtal.flac"), b"").expect("another file to index");

    let mut app = empty();
    assert!(app.add_library_folder(&folder));
    run_the_scan(&mut app);
    assert_eq!(app.browse().track_count(), 2);

    // The same folders, scanned again: a rescan is about what changed, not
    // about adding everything a second time.
    app.start_scan();
    run_the_scan(&mut app);

    assert_eq!(app.browse().track_count(), 2);
    let report = app.last_import().expect("the scan reported").to_owned();
    assert!(report.contains("Indexed 2 tracks"), "{report}");
    assert!(report.contains("0 new"), "{report}");

    std::fs::remove_dir_all(&folder).ok();
}

#[test]
fn playing_a_queue_walks_to_the_end_and_stops_there() {
    let mut app = imported();
    let tracks: Vec<Track> = app.browse().tracks().to_vec();
    assert_eq!(app.play_tracks(&tracks, 0), 2);

    assert_eq!(
        app.playback().current().map(QueueItem::title),
        Some("Track 1")
    );

    app.next_track();
    assert_eq!(
        app.playback().current().map(QueueItem::title),
        Some("Track 2")
    );

    // Past the end there is nothing to play, and the last track stays
    // visible rather than the bar going blank.
    app.next_track();
    assert_eq!(
        app.playback().current().map(QueueItem::title),
        Some("Track 2")
    );
    assert_eq!(
        app.transport(),
        TransportState::Unavailable,
        "no device here"
    );

    app.previous_track();
    assert_eq!(
        app.playback().current().map(QueueItem::title),
        Some("Track 1"),
        "and stepping back works from the end"
    );
}

#[test]
fn the_diagnostics_point_at_the_library_that_was_actually_opened() {
    let directory = crate::fixtures::scratch("diagnostics");
    let database = directory.join(crate::database::DATABASE_FILE);
    let mut app = app();

    app.open_library(&database, directory.join("artwork"), Vec::new());

    let diagnostics = app.diagnostics();
    assert_eq!(
        diagnostics.database_path.as_deref(),
        Some(database.as_path())
    );
    assert_eq!(
        diagnostics.schema_version,
        Some(crate::database::SCHEMA_VERSION),
        "the version is read from the file, not assumed from the build"
    );
    assert_eq!(diagnostics.tracks, 0);
    assert!(diagnostics.config_error.is_none());

    std::fs::remove_dir_all(&directory).ok();
}

#[test]
fn checking_the_library_reports_a_sound_database() {
    let mut app = imported();

    app.check_library_integrity();

    let message = app.library_integrity().expect("the check answered");
    assert!(message.contains("sound"), "{message}");
    assert!(
        message.contains("library.sqlite") || app.diagnostics().database_path.is_none(),
        "an in-memory library has no file to name: {message}"
    );

    app.dismiss_integrity();
    assert!(app.library_integrity().is_none());
}

#[test]
fn a_check_with_no_library_at_all_says_so_rather_than_doing_nothing() {
    let mut app = MusicCraftApp::from_parts(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        Playback::with_engine(AudioEngine::without_device(), 1.0),
        None,
        Some("the library could not be opened".to_owned()),
    );

    app.check_library_integrity();

    let message = app.library_integrity().expect("the check answered");
    assert!(message.contains("could not be checked"), "{message}");
}

/// Drive a running scan to completion, polling as the frame loop does.
///
/// The deadline is what keeps a scan that never finishes from hanging the
/// test suite: it fails, with the reason, rather than running forever.
fn run_the_scan(app: &mut MusicCraftApp) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.is_scanning() {
        app.poll_library();
        assert!(
            std::time::Instant::now() < deadline,
            "the scan never finished"
        );
    }
}

/// Build an application around a library, with no output device.
fn resolve(startup: Startup, library: crate::library::Library) -> MusicCraftApp {
    MusicCraftApp::with_library(
        startup,
        Playback::with_engine(AudioEngine::without_device(), 1.0),
        library,
    )
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

/// An application over a library of its own, with nothing in it yet.
///
/// Counts in a test are about what the test itself puts in, so a scanning
/// test wants an empty library rather than the two rows `imported()`
/// stores.
fn empty() -> MusicCraftApp {
    let library =
        crate::library::Library::in_memory(std::env::temp_dir().join("musiccraft-app-artwork"))
            .expect("an in-memory library");

    resolve(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        library,
    )
}

#[test]
fn playing_a_track_counts_the_play_and_shows_it_on_the_row() {
    let mut app = imported();
    let path = PathBuf::from("/music/1.wav");
    assert_eq!(
        app.browse().tracks()[0].play_count,
        0,
        "nothing has been played yet"
    );

    assert_eq!(
        app.record_plays(std::slice::from_ref(&path)),
        1,
        "the row that exists is counted"
    );

    let counted = app
        .browse()
        .tracks()
        .iter()
        .find(|track| track.path == path)
        .expect("the row");
    assert_eq!(counted.play_count, 1);
}

#[test]
fn playing_a_file_the_library_does_not_index_counts_nothing() {
    // The play field and the command line both play files that were never
    // scanned; there is no row for the count to belong to, and that is not
    // an error.
    let mut app = imported();

    assert_eq!(app.record_plays(&[PathBuf::from("/music/absent.wav")]), 0);
    assert!(app.library_error().is_none(), "and nothing is reported");
}

#[test]
fn each_play_adds_one_to_the_same_row() {
    let mut app = imported();
    let path = PathBuf::from("/music/2.wav");

    app.record_plays(std::slice::from_ref(&path));
    app.record_plays(std::slice::from_ref(&path));

    let counted = app
        .browse()
        .tracks()
        .iter()
        .find(|track| track.path == path)
        .expect("the row");
    assert_eq!(counted.play_count, 2, "the count accumulates");
}

#[test]
fn a_play_count_survives_the_application_being_restarted() {
    // The count is in the database, so a fresh application over the same
    // file reads it back — the point of counting in the library rather than
    // in memory.
    let directory = std::env::temp_dir().join("musiccraft-app-play-count");
    std::fs::remove_dir_all(&directory).ok();
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    let database = directory.join("library.sqlite");
    let path = PathBuf::from("/music/1.wav");

    {
        let library =
            crate::library::Library::open(&database, directory.join("artwork")).expect("a library");
        let mut app = resolve(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            library,
        );
        let mut track = Track::from_path(&path);
        track.id = Some(TrackId(1));
        track.title = "Track 1".to_owned();
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("stored");
        app.refresh().expect("refresh");

        assert_eq!(app.record_plays(std::slice::from_ref(&path)), 1);
    }

    let library = crate::library::Library::open(&database, directory.join("artwork"))
        .expect("the same library");
    let track = library
        .repository()
        .track_by_path(&path)
        .expect("lookup")
        .expect("stored");
    assert_eq!(track.play_count, 1, "the count outlived the first app");
}

#[test]
fn adding_a_whole_album_to_a_playlist_is_one_transaction() {
    let mut app = imported();
    let ids: Vec<TrackId> = app
        .library_tracks()
        .iter()
        .filter_map(|track| track.id)
        .collect();
    assert_eq!(ids.len(), 2, "the fixture stores two tracks");

    assert_eq!(app.add_tracks_to_playlist("Evening", &ids), 2);

    let made = app.playlists().first().expect("a playlist");
    assert_eq!(made.track_count, 2);
}

#[test]
fn adding_nothing_to_a_playlist_writes_nothing() {
    let mut app = imported();

    assert_eq!(app.add_tracks_to_playlist("Evening", &[]), 0);
    assert!(
        app.playlists().is_empty(),
        "an empty list does not conjure a playlist"
    );
}

#[test]
fn shuffle_and_repeat_are_remembered_between_runs() {
    let mut app = app();
    assert!(!app.config.shuffle);
    assert_eq!(app.config.repeat, RepeatMode::Off);

    assert!(app.toggle_shuffle());
    app.set_repeat_mode(RepeatMode::All);

    assert!(app.config_dirty, "the choice is waiting to be written");
    assert!(app.config.shuffle);
    assert_eq!(app.config.repeat, RepeatMode::All);

    // A second application over the same settings starts how the first
    // one left off.
    let restored = MusicCraftApp::with_playback(
        Startup {
            config: app.config.clone(),
            config_path: None,
            error: None,
        },
        Playback::with_engine(AudioEngine::without_device(), 1.0),
    );

    assert!(restored.playback().shuffle());
    assert_eq!(restored.playback().repeat(), RepeatMode::All);
}

#[test]
fn repeating_the_same_choice_dirties_nothing() {
    let mut app = app();
    app.toggle_shuffle();
    app.set_repeat_mode(RepeatMode::All);

    // Answer the pending write, then make the same two choices again.
    app.persist();
    assert!(!app.config_dirty);

    assert!(!app.toggle_shuffle(), "back to in-order playback");
    app.set_repeat_mode(RepeatMode::All);
    assert!(
        app.config_dirty,
        "turning shuffle off and leaving repeat alone are two changes"
    );
}

#[test]
fn a_queue_walked_through_a_playlist_leaves_the_playlist_alone() {
    let mut app = imported();
    let ids: Vec<TrackId> = app
        .library_tracks()
        .iter()
        .filter_map(|track| track.id)
        .collect();
    app.add_tracks_to_playlist("Evening", &ids);

    let id = app.playlists().first().expect("a playlist").id;
    app.open_playlist(id);
    let before: Vec<TrackId> = app
        .playlist_tracks()
        .iter()
        .filter_map(|track| track.id)
        .collect();

    // Playing the playlist fills the queue; moving and removing entries
    // there is the Queue page's business and must not reach the playlist.
    assert_eq!(app.play_playlist(), 2);
    app.move_in_queue(0, 1);
    app.remove_from_queue(0);
    app.clear_queue();

    let after: Vec<TrackId> = app
        .playlist_tracks()
        .iter()
        .filter_map(|track| track.id)
        .collect();

    assert_eq!(after, before, "the playlist kept its tracks and its order");
    assert_eq!(app.playlists().first().expect("a playlist").track_count, 2);
}

#[test]
fn a_track_can_only_be_in_a_playlist_once_as_a_row_but_twice_as_an_entry() {
    let mut app = imported();
    let ids: Vec<TrackId> = app
        .library_tracks()
        .iter()
        .filter_map(|track| track.id)
        .collect();

    // The same track, added twice: two entries, one library row.
    app.add_tracks_to_playlist("Evening", &ids[..1]);
    assert_eq!(app.add_tracks_to_playlist("Evening", &ids[..1]), 1);

    let made = app.playlists().first().expect("a playlist");
    assert_eq!(made.track_count, 2, "the playlist lists it twice");

    app.open_playlist(made.id);
    app.remove_from_playlist(ids[0]);
    assert_eq!(
        app.playlist_tracks().len(),
        1,
        "and one call takes one of them out"
    );
}

#[test]
fn a_fetched_cover_lands_on_every_track_of_the_record() {
    let library = crate::library::Library::in_memory(crate::fixtures::scratch("app-fetched-cover"))
        .expect("an in-memory library");
    let mut app = resolve(
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        },
        library,
    );

    // One album, two rows, no covers yet.
    for (number, title) in [(1, "One"), (2, "Two")] {
        let mut track = Track::from_path(std::path::Path::new(&format!("/album/{number}.wav")));
        track.title = title.to_owned();
        track.album = Some("Low Budget".to_owned());
        track.album_artist = Some("The Cheap Ones".to_owned());
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&track)
            .expect("the row should be stored");
    }
    app.refresh().expect("the library should refresh");
    assert!(
        app.album_tracks("Low Budget")
            .iter()
            .all(|track| track.artwork_path.is_none()),
        "the record starts bare"
    );

    let stored = app.store_album_artwork("Low Budget", &crate::fixtures::png());

    assert_eq!(stored, 2, "both rows got the cover");
    app.refresh().expect("the library should refresh");
    for track in app.album_tracks("Low Budget") {
        let path = track
            .artwork_path
            .expect("the fetched cover is the row's now");
        assert_eq!(
            std::fs::read(&path).expect("the cache file"),
            crate::fixtures::png(),
            "the bytes on disk are the fetched picture"
        );
    }

    let card = app
        .browse()
        .albums()
        .iter()
        .find(|card| card.album.name == "Low Budget")
        .expect("the shelf card");
    assert!(card.artwork.is_some(), "the shelf card draws a cover now");
}

#[test]
fn a_cover_written_for_a_missing_record_stores_nothing() {
    let mut app = app();
    assert_eq!(
        app.store_album_artwork("Unheard Of", &crate::fixtures::png()),
        0
    );
}

#[test]
fn the_pending_and_notice_answers_stay_out_of_each_others_way() {
    let mut app = app();

    assert!(app.album_artwork_pending().is_none());
    assert!(!app.album_artwork_busy());
    assert!(app.artwork_notice().is_none());

    app.artwork_notice = Some(ArtworkNotice::Missing {
        album: "Low Budget".to_owned(),
        reason: "the archive refused".to_owned(),
    });
    assert!(app.artwork_notice().is_some());
    app.dismiss_artwork_notice();
    assert!(app.artwork_notice().is_none());
}
