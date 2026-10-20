//! The Playlists section: the open playlist, and what is in it.
//!
//! The whole section is drawn from what the last [`crate::app::MusicCraftApp::refresh`]
//! loaded — the summaries and the open playlist's tracks — so nothing here
//! queries the database. Actions are collected into a single [`Action`] and
//! applied once the frame is drawn, because a click on a row cannot borrow the
//! application mutably while the list it came from is still being read.
//!
//! Choosing a playlist is not done here. The list of them is in the sidebar,
//! under the Playlists heading, where it is reachable without the section being
//! open; what this section owns is the playlist you have chosen and the order
//! of its tracks.
//!
//! Deleting a playlist asks first, through the same confirmation the library
//! uses: what goes is the list and its order, never a file.

use egui::{Button, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Vec2};

use crate::app::MusicCraftApp;
use crate::library::{PlaylistId, Track, TrackId};
use crate::playlists::PlaylistEditKind;

use super::keyboard_activated;
use super::theme;

/// Height of one track row, in points.
const ROW_HEIGHT: f32 = 34.0;

/// Padding between a row's edge and its text.
const CELL_PADDING: f32 = 8.0;

/// What a click in the section asked for, applied once the frame is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Ask to delete a playlist.
    Delete(PlaylistId),
    /// Start renaming a playlist.
    Rename(PlaylistId),
    /// Start copying a playlist.
    Duplicate(PlaylistId),
    /// Open the rule editor over a smart playlist.
    EditRules(PlaylistId),
    /// Play the open playlist from the top.
    Play,
    /// Add the open playlist to the queue.
    Queue,
    /// Take one track out of the open playlist.
    RemoveTrack(TrackId),
    /// Play the open playlist from one of its entries.
    PlayFrom(usize),
    /// Move an entry one place earlier.
    MoveEarlier(usize),
    /// Move an entry one place later.
    MoveLater(usize),
}

/// Draw the playlist pane: the open playlist, and what can be done with it.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mut action = None;

    // A name already in use, or any other refusal, is shown once at the top of
    // the section rather than left to the row that caused it.
    if let Some(message) = app.playlist_error().map(str::to_owned) {
        error_banner(app, ui, &message);
    }

    detail_column(app, ui, &mut action);

    if let Some(action) = action {
        apply(app, action);
    }
}

/// What the open playlist can do, drawn at the right of the section's heading.
///
/// The buttons sit on the heading line rather than on a row of their own: they
/// act on the playlist the heading names, so putting them beside it says which
/// playlist they are about, and the row they used to take went to the tracks.
///
/// Making a playlist is not here. It is a `+` in the status bar's left corner,
/// reachable from every section rather than only from this one, and out of the
/// way of the list it was making.
///
/// The action is applied straight away: unlike the track rows, nothing this
/// draws is being read out of the application while the buttons are up, so
/// there is no borrow to wait for.
pub fn controls(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mut action = None;

    // Copied out first, so the application is free to be borrowed mutably by
    // `apply` once a button has been clicked.
    let open = app.selected_playlist();
    let has_tracks = !app.playlist_tracks().is_empty();
    let smart = app
        .selected_playlist_summary()
        .is_some_and(|summary| summary.smart);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if let Some(id) = open {
            // Added right to left, so they read left to right as Edit rules,
            // Rename, Duplicate, Delete from the inside out. Edit rules is the
            // smart playlist's own edit — the one thing a hand-made playlist
            // has no equivalent of — so it goes on the end of the added order
            // and the beginning of the read order.
            if ui
                .button(RichText::new("Delete").color(theme::WARNING))
                .on_hover_text("Remove the playlist; the tracks and their files stay")
                .clicked()
            {
                action = Some(Action::Delete(id));
            }

            if ui
                .button("Duplicate")
                .on_hover_text("Copy this playlist under a new name")
                .clicked()
            {
                action = Some(Action::Duplicate(id));
            }

            if ui
                .button("Rename")
                .on_hover_text("Give this playlist another name")
                .clicked()
            {
                action = Some(Action::Rename(id));
            }

            if smart
                && ui
                    .button("Edit rules…")
                    .on_hover_text("Change the rules that fill this playlist")
                    .clicked()
            {
                action = Some(Action::EditRules(id));
            }

            ui.separator();
        }

        ui.add_enabled_ui(has_tracks, |ui| {
            if ui
                .button("Add to queue")
                .on_hover_text("Put this playlist on the end of the queue")
                .clicked()
            {
                action = Some(Action::Queue);
            }

            if ui
                .button("Play")
                .on_hover_text("Play this playlist from the top")
                .clicked()
            {
                action = Some(Action::Play);
            }
        });
    });

    if let Some(action) = action {
        apply(app, action);
    }
}

/// The open playlist: the tracks in play order.
///
/// The playlist's name and totals are not repeated here: the section's heading
/// is the open playlist's name and its subtitle is the totals, so a second copy
/// directly under them said the same thing twice.
fn detail_column(app: &MusicCraftApp, ui: &mut egui::Ui, action: &mut Option<Action>) {
    ui.vertical(|ui| {
        ui.set_min_width(ui.available_width());

        if app.selected_playlist_summary().is_none() {
            ui.label(
                RichText::new("Choose a playlist from the sidebar, or make one with the + below.")
                    .weak(),
            );
            return;
        }

        if app.playlist_tracks().is_empty() {
            if app
                .selected_playlist_summary()
                .is_some_and(|summary| summary.smart)
            {
                ui.label(RichText::new("No tracks match the rules right now.").weak());
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Nothing in the library answers to these rules yet. \"Edit rules…\" \
                         on the heading widens them.",
                    )
                    .small()
                    .weak(),
                );
                return;
            }

            ui.label(RichText::new("This playlist is empty.").weak());
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Open Music, select tracks, and use \"Add to playlist\" on the \
                     right-click menu.",
                )
                .small()
                .weak(),
            );
            return;
        }

        track_rows(app, ui, action);
    });
}

/// The open playlist's tracks, in play order.
///
/// A smart playlist's rows carry no position controls and no "remove" entry:
/// its order belongs to the query and its membership is not something the
/// section holds. Everything else a row can do — play from here, say what the
/// file is — works on both kinds, so both draw the same rows.
fn track_rows(app: &MusicCraftApp, ui: &mut egui::Ui, action: &mut Option<Action>) {
    let tracks = app.playlist_tracks();
    let last = tracks.len().saturating_sub(1);
    let smart = app
        .selected_playlist_summary()
        .is_some_and(|summary| summary.smart);

    for (index, track) in tracks.iter().enumerate() {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());

        let visuals = ui.visuals();
        let background = if response.hovered() {
            visuals.widgets.hovered.weak_bg_fill
        } else if index % 2 == 1 {
            visuals.faint_bg_color
        } else {
            Color32::TRANSPARENT
        };

        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, CornerRadius::same(4), background);

        painter.text(
            Pos2::new(rect.left() + CELL_PADDING, rect.center().y),
            egui::Align2::LEFT_CENTER,
            (index + 1).to_string(),
            FontId::proportional(12.0),
            visuals.weak_text_color(),
        );

        // The title is coloured like the library's rows: a track whose file has
        // gone is the one thing that needs to stand out from the list.
        painter.text(
            Pos2::new(rect.left() + 34.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            truncate(&track.title, ui.available_width() - 150.0),
            FontId::proportional(13.0),
            if track.missing {
                theme::WARNING
            } else {
                visuals.strong_text_color()
            },
        );

        if !smart {
            position_controls(ui, rect, index, last, action);
        }

        // The gesture the Library's table taught: double-click plays from the
        // row, and so does Enter on the row wearing the focus ring — the one
        // keyboard route into a playlist that does not go through the heading.
        if keyboard_activated(&response, ui) || response.double_clicked() {
            *action = Some(Action::PlayFrom(index));
        }

        theme::focus_ring_if_focused(ui, &response);

        let track_id = track.id;

        if smart {
            response.on_hover_text(hover_text(track));
            continue;
        }

        response
            .on_hover_text(hover_text(track))
            .context_menu(|ui| {
                if ui.button("Remove from this playlist").clicked() {
                    if let Some(id) = track_id {
                        *action = Some(Action::RemoveTrack(id));
                    }
                    ui.close();
                }
            });
    }
}

/// The ⏶/⏷ buttons at the right of a track row.
fn position_controls(
    ui: &mut egui::Ui,
    rect: Rect,
    index: usize,
    last: usize,
    action: &mut Option<Action>,
) {
    let size = Vec2::splat(22.0);
    let right = rect.right() - CELL_PADDING;

    for (offset, earlier, label, fallback) in [
        (0.0, true, "⏶", "Move earlier in the playlist"),
        (24.0, false, "⏷", "Move later in the playlist"),
    ] {
        let enabled = if earlier { index > 0 } else { index < last };

        let at = Rect::from_min_size(
            Pos2::new(right - size.x - offset, rect.center().y - size.y / 2.0),
            size,
        );

        ui.add_enabled_ui(enabled, |ui| {
            let response = ui.put(
                at,
                Button::new(RichText::new(label).size(12.0)).frame(false),
            );

            // ⏶ and ⏷ say nothing to a screen reader on their own, and a
            // keyboard user cannot see which of them the row's arrow focus has
            // reached without a ring; both take the words the pointer already
            // gets as hover text.
            theme::accessible_name(ui, &response, fallback);
            theme::focus_ring_if_focused(ui, &response);

            if response.on_hover_text(fallback).clicked() {
                *action = Some(if earlier {
                    Action::MoveEarlier(index)
                } else {
                    Action::MoveLater(index)
                });
            }
        });
    }
}

/// Carry out what a click asked for, now that no borrow of the section is live.
fn apply(app: &mut MusicCraftApp, action: Action) {
    match action {
        Action::Delete(id) => app.ask_to_delete_playlist(id),
        Action::Rename(id) => app.start_playlist_edit(PlaylistEditKind::Rename(id)),
        Action::Duplicate(id) => app.start_playlist_edit(PlaylistEditKind::Duplicate(id)),
        Action::EditRules(id) => {
            app.start_smart_playlist_edit(id);
        }
        Action::Play => {
            app.play_playlist();
        }
        Action::Queue => {
            app.queue_playlist();
        }
        Action::RemoveTrack(id) => {
            app.remove_from_playlist(id);
        }
        Action::PlayFrom(index) => {
            // Copied out first: `play_tracks` takes the rows it queues while
            // the application is mutably borrowed to start playback.
            let tracks: Vec<Track> = app.playlist_tracks().to_vec();
            app.play_tracks(&tracks, index);
        }
        // A move is a swap of one row with its neighbour, which is how the
        // database orders playlist entries.
        Action::MoveEarlier(index) => {
            app.move_in_playlist(index, index.saturating_sub(1));
        }
        Action::MoveLater(index) => {
            app.move_in_playlist(index, index + 1);
        }
    }
}

/// The one-line refusal at the top of the section, with a way to dismiss it.
fn error_banner(app: &mut MusicCraftApp, ui: &mut egui::Ui, message: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(message).color(theme::WARNING));
        if ui.small_button("Dismiss").clicked() {
            app.dismiss_playlist_error();
        }
    });

    ui.add_space(6.0);
}

/// What the pointer resting on a track row says about it.
fn hover_text(track: &Track) -> String {
    let mut text = track.path.display().to_string();

    if track.missing {
        text.push_str("\nThe file is no longer on disk");
    }

    if let Some(duration) = track.duration {
        text.push_str(&format!("\n{}", crate::app::format_duration(duration)));
    }

    text
}

/// Clip text to a width, roughly, using the same ratio the library view does.
///
/// A rough cut rather than a real layout: the row is painted by hand, and a
/// second text-shaping pass per row costs more than the ellipsis is worth.
fn truncate(text: &str, width: f32) -> String {
    let characters = (width / 6.5).max(4.0) as usize;
    if text.chars().count() <= characters {
        return text.to_owned();
    }

    let kept: String = text.chars().take(characters.saturating_sub(1)).collect();

    format!("{}…", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::{MusicCraftApp, Startup};
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{Library, Track};
    use crate::playlists::PlaylistSummary;

    #[test]
    fn drawing_the_section_is_harmless_with_nothing_in_it() {
        let mut app = app();

        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn a_playlist_can_be_made_and_opened_from_the_section() {
        let mut app = imported();

        app.start_playlist_edit(PlaylistEditKind::Create);
        *app.playlist_draft_mut().expect("an editor").name_mut() = "Evening".to_owned();
        app.confirm_playlist_edit();

        let made = app.playlists().first().cloned().expect("a playlist");
        assert_eq!(made.display_name(), "Evening");
        assert_eq!(
            app.selected_playlist(),
            Some(made.id),
            "the new playlist is the one on screen"
        );

        // …and the section draws it without complaint.
        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn a_second_playlist_cannot_take_a_name_that_is_taken() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        app.start_playlist_edit(PlaylistEditKind::Create);
        *app.playlist_draft_mut().expect("an editor").name_mut() = "Evening".to_owned();
        app.confirm_playlist_edit();

        let draft = app.playlist_draft().expect("the editor stays open");
        let reason = draft.error().expect("the name is refused");
        assert!(reason.contains("Evening"), "{reason}");
        assert_eq!(app.playlists().len(), 1, "nothing was made");
    }

    #[test]
    fn a_adding_a_selection_to_a_playlist_stores_the_tracks() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let made = app.playlists().first().cloned().expect("a playlist");
        assert_eq!(made.track_count, 2, "both tracks went in");

        app.open_playlist(made.id);
        assert_eq!(app.playlist_tracks().len(), 2);
    }

    #[test]
    fn a_track_moves_within_a_playlist_and_the_rest_keeps_its_order() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let id = app.playlists().first().expect("a playlist").id;
        app.open_playlist(id);

        let before: Vec<String> = app
            .playlist_tracks()
            .iter()
            .map(|track| track.title.clone())
            .collect();
        assert_eq!(before.len(), 2);

        assert!(app.move_in_playlist(1, 0));
        let after: Vec<String> = app
            .playlist_tracks()
            .iter()
            .map(|track| track.title.clone())
            .collect();

        assert_eq!(after, vec![before[1].clone(), before[0].clone()]);
    }

    #[test]
    fn a_move_that_goes_nowhere_changes_nothing() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        assert!(!app.move_in_playlist(0, 0));
        assert!(!app.move_in_playlist(9, 3));
        assert_eq!(app.playlist_tracks().len(), 2, "nothing was lost");
    }

    #[test]
    fn a_track_can_be_taken_out_of_the_open_playlist() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        let first = app.playlist_tracks()[0].id.expect("an indexed track");
        assert!(app.remove_from_playlist(first));

        assert_eq!(app.playlist_tracks().len(), 1);
        assert_eq!(
            app.playlists().first().expect("a playlist").track_count,
            1,
            "the sidebar count follows"
        );
    }

    #[test]
    fn playing_from_a_row_queues_the_playlist_from_that_entry_on() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        app.open_playlist(app.playlists().first().expect("a playlist").id);

        let second = app.playlist_tracks()[1].title.clone();
        apply(&mut app, Action::PlayFrom(1));

        assert_eq!(
            app.playback().current().map(|item| item.title().to_owned()),
            Some(second),
            "the row the ring was on becomes the track that plays"
        );
        assert_eq!(
            app.playback().queue().len(),
            2,
            "the whole playlist still goes on the queue — the row only moves the start"
        );
    }

    #[test]
    fn enter_on_the_focused_playlist_row_plays_from_there() {
        // Driven the way the keyboard drives it: a frame, a Tab to put the
        // ring on the first row, an Enter. egui 0.36 gives focus to the
        // keyboard only, so this — not a synthetic click — is the honest path.
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        app.open_playlist(app.playlists().first().expect("a playlist").id);
        let first = app.playlist_tracks()[0].title.clone();

        let ctx = egui::Context::default();
        paint(&ctx, &mut app, vec![]);
        let shapes = paint(
            &ctx,
            &mut app,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert!(
            rings(&shapes) == 1,
            "Tab did not put a focus ring on a playlist row"
        );

        paint(
            &ctx,
            &mut app,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );

        assert_eq!(
            app.playback().current().map(|item| item.title().to_owned()),
            Some(first),
            "Enter on the ringed row should have started the playlist at its top"
        );
    }

    /// Draw one frame of the section with `events` in front of it.
    fn paint(
        ctx: &egui::Context,
        app: &mut MusicCraftApp,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        let mut output = ctx.run_ui(
            egui::RawInput {
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 808.0),
                )),
                ..Default::default()
            },
            |ui| show(app, ui),
        );
        output.textures_delta.clear();
        output.shapes
    }

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    /// How many painted focus-ring accent bands the frame carries.
    fn rings(shapes: &[egui::epaint::ClippedShape]) -> usize {
        shapes
            .iter()
            .filter(|clipped| match &clipped.shape {
                egui::Shape::Rect(boxed) => {
                    boxed.stroke.color == theme::FOCUS_RING
                        && (boxed.stroke.width - theme::FOCUS_RING_WIDTH).abs() < 0.01
                }
                _ => false,
            })
            .count()
    }

    #[test]
    fn deleting_a_playlist_is_asked_about_and_leaves_the_tracks_alone() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        let id = app.playlists().first().expect("a playlist").id;

        app.ask_to_delete_playlist(id);
        assert!(
            app.pending_confirmation().is_some(),
            "the user is asked first"
        );

        app.confirm_pending();

        assert!(app.playlists().is_empty(), "the playlist is gone");
        assert_eq!(
            app.browse().track_count(),
            2,
            "the tracks it listed are still in the library"
        );
    }

    #[test]
    fn a_text_is_cut_to_its_column() {
        assert_eq!(truncate("short", 200.0), "short");
        assert!(truncate(&"x".repeat(200), 40.0).ends_with('…'));
    }

    #[test]
    fn a_playlists_totals_are_phrased_like_the_rest_of_the_interface() {
        let empty = PlaylistSummary {
            id: PlaylistId(1),
            name: "New".to_owned(),
            track_count: 0,
            duration: None,
            smart: false,
        };
        let one = PlaylistSummary {
            id: PlaylistId(2),
            name: "Evening".to_owned(),
            track_count: 1,
            duration: Some(std::time::Duration::from_secs(61)),
            smart: false,
        };

        assert_eq!(empty.totals(), "No tracks yet");
        assert_eq!(one.totals(), "1 track — 1:01");
    }

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

    /// Every library track's row identifier, as the menu would hand them over.
    fn library_ids(app: &MusicCraftApp) -> Vec<TrackId> {
        app.browse()
            .tracks()
            .iter()
            .filter_map(|track| track.id)
            .collect()
    }

    /// An application with two indexed tracks and no playlists.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-playlists-artwork"))
            .expect("an in-memory library");
        let mut app = MusicCraftApp::with_library(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
            library,
        );

        for (index, title) in ["Roygbiv", "Olson"].iter().enumerate() {
            let mut track = Track::from_path(format!("/music/{title}.mp3"));
            track.id = Some(TrackId(i64::try_from(index + 1).expect("small")));
            track.title = (*title).to_owned();
            track.duration = Some(std::time::Duration::from_secs(61));
            app.library_mut()
                .expect("a library")
                .repository_mut()
                .upsert_track(&track)
                .expect("the track should be stored");
        }

        app.refresh().expect("the library should refresh");
        app
    }
}
