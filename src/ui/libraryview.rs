//! The Library section: the music that has been indexed, as tracks, albums or
//! artists.
//!
//! Everything shown here comes from the database through
//! [`MusicCraftApp::browse`], and every click goes back through the application
//! (`play_selection`, `play_album`, …). The view holds no state of its own: it
//! reads a snapshot, paints a frame, and applies at most one action per frame,
//! which is what keeps a click from being handled against rows that changed
//! underneath it.
//!
//! Rows are painted rather than laid out with widgets because the table needs
//! columns that line up exactly, truncated text that cannot overflow into its
//! neighbour, and a whole-row hit area that drives selection, double-click to
//! play and a context menu at the same time.

use std::collections::BTreeSet;

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::library::browse::{AlbumCard, BrowseKind, ClickMode};
use crate::library::{Track, TrackId};
use crate::search::SortKey;
use crate::ui::{empty_state, theme};

/// Height of a track or artist row, in points.
const ROW_HEIGHT: f32 = 24.0;
/// Height of the table's column headings.
const HEADER_HEIGHT: f32 = 22.0;
/// Padding between a column's edge and its text.
const CELL_PADDING: f32 = 8.0;

/// Width of the row-number column.
const INDEX_WIDTH: f32 = 38.0;
/// Width of the duration column.
const DURATION_WIDTH: f32 = 64.0;
/// Width of the play-count column.
///
/// Wide enough for a four-digit count and the heading beside it; a number
/// longer than that is rare enough to be worth truncating rather than spending
/// the artist's width on it.
const PLAYS_WIDTH: f32 = 56.0;

/// Width of the artist, album and genre columns when they fit.
const ARTIST_WIDTH: f32 = 150.0;
const ALBUM_WIDTH: f32 = 170.0;
const GENRE_WIDTH: f32 = 100.0;

/// The narrowest the title column is allowed to get.
const TITLE_MIN_WIDTH: f32 = 140.0;

/// Window widths below which a column is dropped rather than squeezed.
const ARTIST_MIN_WIDTH: f32 = 520.0;
const ALBUM_MIN_WIDTH: f32 = 680.0;
const GENRE_MIN_WIDTH: f32 = 860.0;

/// A rough characters-per-point ratio for the proportional face, used to
/// truncate text to its column without laying it out twice.
const CHARACTERS_PER_POINT: f32 = 1.0 / 6.6;

/// Size of one album tile in the grid.
const ALBUM_TILE: Vec2 = Vec2::new(148.0, 190.0);
/// Height of the cover art inside a tile.
const ALBUM_ART_HEIGHT: f32 = 132.0;

/// Size of the search field drawn beside the Library heading.
const SEARCH_WIDTH: f32 = 220.0;
const SEARCH_HEIGHT: f32 = 22.0;

/// What a click on a row asked for, applied once the frame is drawn.
enum RowAction {
    Select {
        index: usize,
        mode: ClickMode,
    },
    Play {
        index: usize,
    },
    Queue {
        index: usize,
    },
    Edit {
        index: usize,
    },
    Remove {
        index: usize,
    },
    /// Put a track into a playlist that already exists, by name.
    AddToPlaylist {
        track: TrackId,
        name: String,
    },
    /// Name a new playlist and put the track in it.
    NewPlaylist {
        track: TrackId,
    },
}

/// What a click on an album or artist asked for.
enum GroupAction {
    PlayAlbum(String),
    PlayArtist(String),
    QueueAlbum(String),
    QueueArtist(String),
    Filter(String),
}

/// Draw the Library.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    status(app, ui);
    ui.add_space(6.0);

    match app.library() {
        None => error_state(app, ui),
        Some(_) => content(app, ui),
    }
}

/// The search field drawn beside the Library heading.
///
/// The caller lays it out right-to-left, so the Clear button is added first to
/// land on the right of the field. The field is kept narrow on purpose; the
/// hint lists what a query actually matches.
pub fn search_field(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    if app.browse().is_filtered() && ui.small_button("Clear").clicked() {
        app.browse_mut().set_query_text(String::new());
    }

    ui.add_sized(
        [SEARCH_WIDTH, SEARCH_HEIGHT],
        TextEdit::singleline(app.browse_mut().query_text_mut()).hint_text("Search…"),
    )
    .on_hover_text("Matches title, artist, album, genre and path");
}

/// What the selection can do, and whether a filter is hiding anything.
///
/// How many songs, albums and artists the library holds is the status bar's
/// business now — anything that describes the whole collection belongs along the
/// bottom of the window — so this line reports only what the library view itself
/// is narrowing.
fn status(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let shown = app.browse().visible_tracks().len();
    let filtered = app.browse().is_filtered();
    let missing = app.browse().missing_count();
    let missing_only = app.browse().missing_only();

    // A file that has gone is the one library problem a scan cannot fix by
    // itself, so the view names it, offers to show just those rows, and offers
    // to stop indexing them.
    if missing > 0 {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{missing} {} missing from disk",
                    if missing == 1 { "file is" } else { "files are" }
                ))
                .small()
                .color(theme::WARNING),
            );

            if ui
                .small_button(if missing_only { "Show all" } else { "Show them" })
                .on_hover_text("Show only the tracks whose file has vanished")
                .clicked()
            {
                app.browse_mut().toggle_missing_only();
            }

            if ui
                .small_button("Forget them")
                .on_hover_text("Remove those rows; the files are already gone and a scan puts back any that return")
                .clicked()
            {
                app.ask_to_forget_missing();
            }
        });
    }

    if filtered {
        ui.label(
            RichText::new(format!("{shown} shown"))
                .small()
                .color(theme::ACCENT),
        );
    }

    selection_actions(app, ui);
}

/// What the current selection can be played, queued or removed with.
fn selection_actions(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let selected = app.browse().selection().len();
    if selected == 0 {
        return;
    }

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{selected} {} selected",
                if selected == 1 { "track" } else { "tracks" }
            ))
            .small()
            .color(theme::ACCENT),
        );

        if ui.small_button("Play").clicked() {
            app.play_selection();
        }
        if ui.small_button("Add to queue").clicked() {
            app.queue_selection();
        }
        if ui
            .add_enabled(selected == 1, egui::Button::new("Edit details").small())
            .on_hover_text("Change the title, artist, album and cover of the selected track")
            .clicked()
        {
            app.edit_selection();
        }
        if ui
            .small_button("Remove")
            .on_hover_text("Remove from the library; the files stay on disk")
            .clicked()
        {
            // Nothing is forgotten until the user has agreed to it: the
            // question is asked by the confirmation drawn over this view.
            app.ask_to_remove_selection();
        }
        if ui.small_button("Clear").clicked() {
            app.browse_mut().selection_mut().clear();
        }
    });
}

/// The page the view switch selected.
fn content(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    if app.browse().is_empty() {
        return empty_library(app, ui);
    }

    match app.browse().kind() {
        BrowseKind::Tracks => tracks_page(app, ui),
        BrowseKind::Albums => albums_page(app, ui),
        BrowseKind::Artists => artists_page(app, ui),
    }
}

/// Nothing indexed yet: explain what to do about it.
fn empty_library(app: &MusicCraftApp, ui: &mut egui::Ui) {
    if app.is_scanning() {
        empty_state(
            ui,
            "Indexing your music…",
            "Files are being read; tracks appear here as they are added.",
            &[],
        );
    } else if app.library_folders().is_empty() {
        empty_state(
            ui,
            "Your library is empty",
            "Add a folder under Settings → Library management and the tracks inside it are indexed.",
            &[
                "Every supported audio file under the folder is imported",
                "Tags, durations and cover art are read where the file has them",
                "Re-importing a folder updates what changed instead of duplicating",
            ],
        );
    } else {
        empty_state(
            ui,
            "Nothing indexed yet",
            "The folders you watch hold no supported audio files.",
            &[
                "Press Rescan under Settings → Library management to read them again",
                "Only mp3, flac, wav, ogg, m4a, mp4 and aac files are imported",
                "Sub-folders are scanned as well",
            ],
        );
    }
}

/// A query that hides everything: say so, and offer a way back.
fn no_matches(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let query = app.browse().query_text().to_owned();
    let missing_only = app.browse().missing_only();

    if missing_only {
        empty_state(
            ui,
            "No files are missing",
            "Every indexed track still has its file. This filter only hides rows, so you can              see the whole library again at any time.",
            &[],
        );
    } else {
        empty_state(
            ui,
            "Nothing matches",
            &format!("No track matches “{query}” in its title, artist, album, genre or path."),
            &[],
        );
    }

    ui.add_space(10.0);
    // One button, because the two filters can be on at once: clearing either
    // one alone could leave the table just as empty.
    ui.vertical_centered(|ui| {
        if ui.button("Clear the filters").clicked() {
            app.browse_mut().set_query_text(String::new());
            app.browse_mut().set_missing_only(false);
        }
    });
}

/// The library could not be opened: playback still works, browsing does not.
fn error_state(app: &MusicCraftApp, ui: &mut egui::Ui) {
    let detail = app
        .library_error()
        .unwrap_or("The music library is unavailable.")
        .to_owned();

    empty_state(
        ui,
        "The library is not available",
        &detail,
        &[
            "Playback, settings and the play field below still work",
            "The library database and its folder live in the data directory",
        ],
    );
}

/// The All Tracks table: a sortable header, then the rows.
fn tracks_page(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let tracks: Vec<Track> = app.browse().visible_tracks().into_iter().cloned().collect();

    if tracks.is_empty() {
        return no_matches(app, ui);
    }

    let columns = Columns::for_width(ui.available_width());

    if let Some(key) = header(app, ui, &columns) {
        app.browse_mut().toggle_sort(key);
    }

    let playing = app
        .playback()
        .current()
        .map(|item| item.path().to_path_buf());
    let selection: BTreeSet<TrackId> = app.browse().selection().ids().collect();

    let mut action: Option<RowAction> = None;
    let height = ui.available_height().max(ROW_HEIGHT * 3.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(height)
        .show_rows(ui, ROW_HEIGHT, tracks.len(), |ui, range| {
            for index in range {
                let track = &tracks[index];
                let selected = track.id.is_some_and(|id| selection.contains(&id));
                let is_current = playing.as_deref() == Some(track.path.as_path());

                let response = track_row(ui, &columns, index + 1, track, selected, is_current);

                if response.double_clicked() {
                    action = Some(RowAction::Play { index });
                } else if response.clicked() {
                    action = Some(RowAction::Select {
                        index,
                        mode: click_mode(ui),
                    });
                }

                response.context_menu(|ui| {
                    if ui.button("Play").clicked() {
                        action = Some(RowAction::Play { index });
                        ui.close();
                    }
                    if ui.button("Add to queue").clicked() {
                        action = Some(RowAction::Queue { index });
                        ui.close();
                    }
                    if ui
                        .button("Edit details…")
                        .on_hover_text("Change the tags the library shows for this track")
                        .clicked()
                    {
                        action = Some(RowAction::Edit { index });
                        ui.close();
                    }

                    playlist_menu(app, ui, index, &mut action);

                    ui.separator();
                    if ui
                        .button("Remove from library")
                        .on_hover_text("The file on disk is left untouched")
                        .clicked()
                    {
                        action = Some(RowAction::Remove { index });
                        ui.close();
                    }
                });
            }
        });

    let Some(action) = action else {
        return;
    };

    // The visible rows are the rows the click indices refer to, so both the
    // selection and the tracks played come from the same snapshot.
    let visible: Vec<TrackId> = app.browse().visible_track_ids();

    match action {
        RowAction::Select { index, mode } => {
            if let Some(id) = visible.get(index).copied() {
                app.browse_mut().selection_mut().apply(&visible, id, mode);
            }
        }
        RowAction::Play { index } => {
            app.play_tracks(&tracks, index);
        }
        RowAction::Queue { index } => {
            if let Some(track) = tracks.get(index) {
                app.queue_tracks(std::slice::from_ref(track));
            }
        }
        RowAction::Edit { index } => {
            if let Some(id) = visible.get(index).copied() {
                app.browse_mut()
                    .selection_mut()
                    .apply(&visible, id, ClickMode::Replace);
                app.edit_selection();
            }
        }
        RowAction::Remove { index } => {
            if let Some(id) = visible.get(index).copied() {
                app.browse_mut()
                    .selection_mut()
                    .apply(&visible, id, ClickMode::Replace);
            }
            app.ask_to_remove_selection();
        }
        RowAction::AddToPlaylist { track, name } => {
            app.add_tracks_to_playlist(&name, &[track]);
        }
        RowAction::NewPlaylist { track } => {
            app.start_playlist_from_tracks(&[track]);
        }
    }
}

/// The "Add to playlist" entries for one row.
///
/// The current track is the one being put in, not the selection: the menu hangs
/// off a row and a right-click on a row that is not selected would otherwise add
/// whatever was selected before it.
fn playlist_menu(
    app: &MusicCraftApp,
    ui: &mut egui::Ui,
    index: usize,
    action: &mut Option<RowAction>,
) {
    let Some(track) = app.library_tracks().get(index).and_then(|track| track.id) else {
        return;
    };

    ui.menu_button("Add to playlist", |ui| {
        for playlist in app.playlists() {
            if ui.button(playlist.display_name()).clicked() {
                *action = Some(RowAction::AddToPlaylist {
                    track,
                    name: playlist.name.clone(),
                });
                ui.close();
            }
        }

        if !app.playlists().is_empty() {
            ui.separator();
        }

        if ui
            .button("New playlist…")
            .on_hover_text("Name a playlist and put this track in it")
            .clicked()
        {
            *action = Some(RowAction::NewPlaylist { track });
            ui.close();
        }
    });
}

/// Column headings, clickable to sort. Returns the column that was clicked.
fn header(app: &MusicCraftApp, ui: &mut egui::Ui, columns: &Columns) -> Option<SortKey> {
    let sort = app.browse().sort();
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, HEADER_HEIGHT), Sense::hover());

    let font = FontId::proportional(12.0);
    let color = ui.visuals().weak_text_color();
    let mut clicked = None;

    {
        let painter = ui.painter().with_clip_rect(rect);
        painter.text(
            Pos2::new(rect.left() + CELL_PADDING, rect.center().y),
            Align2::LEFT_CENTER,
            "#",
            font.clone(),
            color,
        );

        for key in SortKey::ALL {
            let width = columns.width_for(key);
            if width < 1.0 {
                continue;
            }

            let active = sort.key == key;
            let label = if active {
                format!("{} {}", key.label(), sort.direction.arrow())
            } else {
                key.label().to_owned()
            };

            painter.text(
                Pos2::new(
                    rect.left() + columns.offset_for(key) + CELL_PADDING,
                    rect.center().y,
                ),
                Align2::LEFT_CENTER,
                label,
                font.clone(),
                if active { theme::ACCENT } else { color },
            );
        }
    }

    for key in SortKey::ALL {
        let width = columns.width_for(key);
        if width < 1.0 {
            continue;
        }

        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + columns.offset_for(key), rect.top()),
            Vec2::new(width, HEADER_HEIGHT),
        );

        let response = ui
            .interact(
                cell,
                ui.id().with(("library_header", key.label())),
                Sense::click(),
            )
            .on_hover_text(format!("Sort by {}", key.label()));

        if response.clicked() {
            clicked = Some(key);
        }
    }

    clicked
}

/// One track row: background, five columns of text, and the click target.
fn track_row(
    ui: &mut egui::Ui,
    columns: &Columns,
    number: usize,
    track: &Track,
    selected: bool,
    playing: bool,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click());

    let visuals = ui.visuals();
    let background = if selected {
        visuals.selection.bg_fill
    } else if response.hovered() {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };

    let font = FontId::proportional(13.0);
    let small = FontId::proportional(12.0);
    let strong = visuals.strong_text_color();
    let weak = visuals.weak_text_color();
    let title_color = if track.missing {
        theme::WARNING
    } else if playing {
        theme::ACCENT
    } else {
        strong
    };

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::same(4), background);

    painter.text(
        Pos2::new(
            rect.left() + columns.offset_of(Column::Index) + CELL_PADDING,
            rect.center().y,
        ),
        Align2::LEFT_CENTER,
        number.to_string(),
        small.clone(),
        weak,
    );

    let title = truncate(&title_cell(track), columns.width_of(Column::Title));
    painter.text(
        Pos2::new(
            rect.left() + columns.offset_of(Column::Title) + CELL_PADDING,
            rect.center().y,
        ),
        Align2::LEFT_CENTER,
        title,
        font.clone(),
        title_color,
    );

    for (column, text, size) in [
        (
            Column::Artist,
            track.display_artist().unwrap_or("—").to_owned(),
            small.clone(),
        ),
        (
            Column::Album,
            track.album.clone().unwrap_or_else(|| "—".to_owned()),
            small.clone(),
        ),
        (
            Column::Genre,
            track.genre.clone().unwrap_or_else(|| "—".to_owned()),
            small.clone(),
        ),
        (
            Column::Duration,
            track
                .duration
                .map_or_else(|| "--:--".to_owned(), crate::app::format_duration),
            small.clone(),
        ),
        (Column::Plays, track.play_count.to_string(), small),
    ] {
        if columns.width_of(column) < 1.0 {
            continue;
        }

        painter.text(
            Pos2::new(
                rect.left() + columns.offset_of(column) + CELL_PADDING,
                rect.center().y,
            ),
            Align2::LEFT_CENTER,
            truncate(&text, columns.width_of(column)),
            size,
            weak,
        );
    }

    response.on_hover_text(hover_text(track))
}

/// What marks a row whose tags have been edited here.
///
/// A prefix rather than a suffix, so it cannot be the part a narrow column
/// truncates away — the point of the marker is that it is always visible. The
/// glyph is one the bundled fonts carry (a pencil is not), and the test below
/// keeps that true.
const EDITED_MARKER: &str = "✱";

/// The title as a row shows it.
///
/// An edited row carries [`EDITED_MARKER`]; the row's colour already means
/// "missing" or "playing", so the edit is said in a glyph rather than a shade.
fn title_cell(track: &Track) -> String {
    if track.tags_dirty {
        format!("{EDITED_MARKER} {}", track.title)
    } else {
        track.title.clone()
    }
}

/// The line shown when the pointer rests on a row.
fn hover_text(track: &Track) -> String {
    let mut text = String::new();

    if track.missing {
        text.push_str("This file is missing from disk\n");
    }

    if track.tags_dirty {
        text.push_str("Edited in the library; the file's own tags still hold the old values\n");
    }

    text.push_str(&track.path.display().to_string());

    if let Some(number) = track.track_number {
        text.push_str(&format!("\nTrack {number}"));
    }

    text
}

/// The album grid.
fn albums_page(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let cards: Vec<AlbumCard> = app.browse().visible_albums().into_iter().cloned().collect();

    if cards.is_empty() {
        return no_matches(app, ui);
    }

    let mut action: Option<GroupAction> = None;

    ui.horizontal_wrapped(|ui| {
        for card in &cards {
            let texture = card
                .artwork
                .as_deref()
                .and_then(|path| app.artwork_mut().texture(ui.ctx(), path))
                .map(|texture| texture.id());

            let response = album_tile(ui, texture, card);

            if response.double_clicked() {
                action = Some(GroupAction::PlayAlbum(card.album.name.clone()));
            } else if response.clicked() {
                action = Some(GroupAction::Filter(card.album.name.clone()));
            }

            response.context_menu(|ui| {
                if ui.button("Play album").clicked() {
                    action = Some(GroupAction::PlayAlbum(card.album.name.clone()));
                    ui.close();
                }
                if ui.button("Add album to queue").clicked() {
                    action = Some(GroupAction::QueueAlbum(card.album.name.clone()));
                    ui.close();
                }
                ui.separator();
                if ui.button("Show its tracks").clicked() {
                    action = Some(GroupAction::Filter(card.album.name.clone()));
                    ui.close();
                }
            });
        }
    });

    apply_group_action(app, action);
}

/// One album: cover art, name, and the artist it is credited to.
fn album_tile(
    ui: &mut egui::Ui,
    texture: Option<egui::TextureId>,
    card: &AlbumCard,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(ALBUM_TILE, Sense::click());

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(6),
            ui.visuals().widgets.hovered.weak_bg_fill,
        );
    }

    let art = Rect::from_min_size(
        rect.min + Vec2::new(CELL_PADDING, CELL_PADDING),
        Vec2::new(rect.width() - CELL_PADDING * 2.0, ALBUM_ART_HEIGHT),
    );

    match texture {
        Some(texture) => {
            ui.painter().image(
                texture,
                art,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            // A placeholder rather than an empty hole: the tile keeps its shape
            // and the album's initial stands in for the missing cover.
            ui.painter()
                .rect_filled(art, CornerRadius::same(4), ui.visuals().faint_bg_color);
            let initial = card.album.name.chars().next().map_or_else(
                || "?".to_owned(),
                |letter| letter.to_uppercase().to_string(),
            );

            ui.painter().text(
                art.center(),
                Align2::CENTER_CENTER,
                initial,
                FontId::proportional(28.0),
                ui.visuals().weak_text_color(),
            );
        }
    }

    let name_y = art.bottom() + 12.0;
    ui.painter().text(
        Pos2::new(rect.left() + CELL_PADDING, name_y),
        Align2::LEFT_CENTER,
        truncate(&card.album.name, rect.width() - CELL_PADDING * 2.0),
        FontId::proportional(13.0),
        ui.visuals().strong_text_color(),
    );

    let credit = match card.album.album_artist.as_deref() {
        Some(artist) => format!("{artist} · {} tracks", card.album.track_count),
        None => format!("{} tracks", card.album.track_count),
    };
    ui.painter().text(
        Pos2::new(rect.left() + CELL_PADDING, name_y + 16.0),
        Align2::LEFT_CENTER,
        truncate(&credit, rect.width() - CELL_PADDING * 2.0),
        FontId::proportional(12.0),
        ui.visuals().weak_text_color(),
    );

    response
}

/// The artist list.
fn artists_page(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let artists: Vec<crate::library::Artist> = app
        .browse()
        .visible_artists()
        .into_iter()
        .cloned()
        .collect();

    if artists.is_empty() {
        return no_matches(app, ui);
    }

    let mut action: Option<GroupAction> = None;

    // Virtualised like the track table: only the rows on screen are drawn, so
    // an artist list of any size costs the same per frame.
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, ROW_HEIGHT, artists.len(), |ui, range| {
            for index in range {
                let artist = &artists[index];
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), ROW_HEIGHT),
                    Sense::click(),
                );

                if response.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(4),
                        ui.visuals().widgets.hovered.weak_bg_fill,
                    );
                }

                ui.painter().text(
                    Pos2::new(rect.left() + CELL_PADDING, rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate(&artist.name, rect.width() * 0.5),
                    FontId::proportional(13.0),
                    ui.visuals().strong_text_color(),
                );

                ui.painter().text(
                    Pos2::new(rect.left() + rect.width() * 0.5, rect.center().y),
                    Align2::LEFT_CENTER,
                    format!(
                        "{} {} · {} {}",
                        artist.album_count,
                        if artist.album_count == 1 {
                            "album"
                        } else {
                            "albums"
                        },
                        artist.track_count,
                        if artist.track_count == 1 {
                            "track"
                        } else {
                            "tracks"
                        }
                    ),
                    FontId::proportional(12.0),
                    ui.visuals().weak_text_color(),
                );

                if response.double_clicked() {
                    action = Some(GroupAction::PlayArtist(artist.name.clone()));
                } else if response.clicked() {
                    action = Some(GroupAction::Filter(artist.name.clone()));
                }

                response.context_menu(|ui| {
                    if ui.button("Play").clicked() {
                        action = Some(GroupAction::PlayArtist(artist.name.clone()));
                        ui.close();
                    }
                    if ui.button("Add to queue").clicked() {
                        action = Some(GroupAction::QueueArtist(artist.name.clone()));
                        ui.close();
                    }
                    if ui.button("Show their tracks").clicked() {
                        action = Some(GroupAction::Filter(artist.name.clone()));
                        ui.close();
                    }
                });
            }
        });

    apply_group_action(app, action);
}

/// Run whatever an album or artist click asked for.
fn apply_group_action(app: &mut MusicCraftApp, action: Option<GroupAction>) {
    match action {
        Some(GroupAction::PlayAlbum(name)) => {
            app.play_album(&name);
        }
        Some(GroupAction::PlayArtist(name)) => {
            app.play_artist(&name);
        }
        Some(GroupAction::QueueAlbum(name)) => {
            app.queue_album(&name);
        }
        Some(GroupAction::QueueArtist(name)) => {
            app.queue_artist(&name);
        }
        Some(GroupAction::Filter(name)) => {
            app.browse_mut().set_query_text(name);
            app.browse_mut().set_kind(BrowseKind::Tracks);
        }
        None => {}
    }
}

/// The columns of the track table, sized for the width available.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Columns {
    index: f32,
    title: f32,
    artist: f32,
    album: f32,
    genre: f32,
    duration: f32,
    plays: f32,
}

/// One column of the track table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Index,
    Title,
    Artist,
    Album,
    Genre,
    Duration,
    Plays,
}

/// The column a sortable heading belongs to.
fn column_of(key: SortKey) -> Option<Column> {
    match key {
        SortKey::Title => Some(Column::Title),
        SortKey::Artist => Some(Column::Artist),
        SortKey::Album => Some(Column::Album),
        SortKey::Genre => Some(Column::Genre),
        SortKey::Duration => Some(Column::Duration),
        SortKey::PlayCount => Some(Column::Plays),
        // The table has no track-number or path column; those keys are still
        // available to the sort itself, just not through a heading.
        SortKey::TrackNumber | SortKey::Path => None,
    }
}

impl Columns {
    /// Lay the table out for a window this wide.
    ///
    /// The number, duration and play count columns are always shown; artist,
    /// album and genre are dropped from the right as the window narrows, and
    /// the title takes whatever is left so a long name is still readable.
    fn for_width(width: f32) -> Self {
        let artist = if width >= ARTIST_MIN_WIDTH {
            ARTIST_WIDTH
        } else {
            0.0
        };
        let album = if width >= ALBUM_MIN_WIDTH {
            ALBUM_WIDTH
        } else {
            0.0
        };
        let genre = if width >= GENRE_MIN_WIDTH {
            GENRE_WIDTH
        } else {
            0.0
        };

        let fixed = INDEX_WIDTH + artist + album + genre + DURATION_WIDTH + PLAYS_WIDTH;
        let title = (width - fixed - CELL_PADDING).max(TITLE_MIN_WIDTH);

        Self {
            index: INDEX_WIDTH,
            title,
            artist,
            album,
            genre,
            duration: DURATION_WIDTH,
            plays: PLAYS_WIDTH,
        }
    }

    /// How wide a column is.
    fn width_of(&self, column: Column) -> f32 {
        match column {
            Column::Index => self.index,
            Column::Title => self.title,
            Column::Artist => self.artist,
            Column::Album => self.album,
            Column::Genre => self.genre,
            Column::Duration => self.duration,
            Column::Plays => self.plays,
        }
    }

    /// How wide the column a heading sorts by is.
    fn width_for(&self, key: SortKey) -> f32 {
        column_of(key).map_or(0.0, |column| self.width_of(column))
    }

    /// How far from the left edge a column starts.
    fn offset_of(&self, column: Column) -> f32 {
        match column {
            Column::Index => 0.0,
            Column::Title => self.index,
            Column::Artist => self.index + self.title,
            Column::Album => self.index + self.title + self.artist,
            Column::Genre => self.index + self.title + self.artist + self.album,
            Column::Duration => self.index + self.title + self.artist + self.album + self.genre,
            Column::Plays => {
                self.index + self.title + self.artist + self.album + self.genre + self.duration
            }
        }
    }

    /// How far from the left edge the column a heading sorts by starts.
    fn offset_for(&self, key: SortKey) -> f32 {
        column_of(key).map_or(0.0, |column| self.offset_of(column))
    }
}

/// Which selection click the pressed modifiers ask for.
fn click_mode(ui: &egui::Ui) -> ClickMode {
    let (additive, range) = ui.input(|input| {
        (
            input.modifiers.command || input.modifiers.ctrl,
            input.modifiers.shift,
        )
    });

    ClickMode::from_modifiers(additive, range)
}

/// Shorten `text` so it fits a column `width` points wide.
///
/// A width estimate is used rather than a text layout because the row is drawn
/// once per row and frame; the text is clipped to its column as well, so an
/// over-long name can never spill into the column beside it.
fn truncate(text: &str, width: f32) -> String {
    let budget = (width * CHARACTERS_PER_POINT) as usize;
    if budget == 0 {
        return String::new();
    }

    if text.chars().count() <= budget {
        return text.to_owned();
    }

    let kept: String = text.chars().take(budget.saturating_sub(1)).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{Library, TrackId};

    #[test]
    fn the_table_shows_the_title_first_and_drops_columns_as_it_narrows() {
        let wide = Columns::for_width(1000.0);
        assert!(wide.artist > 0.0);
        assert!(wide.album > 0.0);
        assert!(wide.genre > 0.0);

        let medium = Columns::for_width(700.0);
        assert!(medium.artist > 0.0);
        assert!(medium.album > 0.0);
        assert_eq!(medium.genre, 0.0, "genre goes first");

        let narrow = Columns::for_width(500.0);
        assert_eq!(narrow.artist, 0.0);
        assert_eq!(narrow.album, 0.0);
        assert!(
            narrow.title >= TITLE_MIN_WIDTH,
            "the title keeps its minimum width"
        );
    }

    #[test]
    fn the_columns_are_laid_out_left_to_right_without_gaps() {
        let columns = Columns::for_width(1000.0);

        assert_eq!(columns.offset_of(Column::Index), 0.0);
        assert_eq!(columns.offset_of(Column::Title), columns.index);
        assert_eq!(
            columns.offset_of(Column::Duration),
            columns.index + columns.title + columns.artist + columns.album + columns.genre
        );
    }

    #[test]
    fn every_sortable_heading_has_a_column() {
        for key in SortKey::ALL {
            assert!(column_of(key).is_some(), "{key:?} has no column");
        }

        assert!(column_of(SortKey::Path).is_none());
        assert_eq!(column_of(SortKey::Artist), Some(Column::Artist));
    }

    #[test]
    fn text_is_shortened_to_its_column() {
        let long = "a".repeat(200);

        let shortened = truncate(&long, 100.0);
        assert!(shortened.chars().count() < 200);
        assert!(shortened.ends_with('…'));

        assert_eq!(truncate("Xtal", 100.0), "Xtal");
        assert_eq!(truncate("Xtal", 0.0), "");
    }

    #[test]
    fn modifiers_pick_the_selection_mode() {
        assert_eq!(ClickMode::from_modifiers(false, false), ClickMode::Replace);
        assert_eq!(ClickMode::from_modifiers(true, false), ClickMode::Toggle);
        assert_eq!(ClickMode::from_modifiers(false, true), ClickMode::Range);
    }

    #[test]
    fn an_edited_row_is_marked_in_front_of_its_title() {
        let clean = Track::from_path("/music/Roygbiv.mp3");
        assert_eq!(title_cell(&clean), "Roygbiv");

        let mut edited = clean.clone();
        edited.tags_dirty = true;
        assert_eq!(title_cell(&edited), "✱ Roygbiv");
    }

    #[test]
    fn the_edit_marker_is_a_glyph_the_fonts_can_draw() {
        // A marker none of egui's fonts carry would be dropped silently, and
        // the row would look untouched while claiming to be edited.
        let ctx = egui::Context::default();
        let mut drawable = false;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            drawable = ui
                .ctx()
                .fonts_mut(|fonts| fonts.has_glyphs(&FontId::proportional(13.0), EDITED_MARKER));
        });
        output.drop_without_applying_deltas();

        assert!(drawable, "none of egui's fonts can draw {EDITED_MARKER}");
    }

    #[test]
    fn hovering_an_edited_track_says_the_file_does_not_know_yet() {
        let mut track = Track::from_path("/music/Roygbiv.mp3");
        track.tags_dirty = true;

        let text = hover_text(&track);

        assert!(text.contains("Edited"), "{text}");
        assert!(text.contains("tags"), "{text}");
    }

    #[test]
    fn hovering_a_missing_track_says_so() {
        let mut track = Track::from_path("/music/gone.mp3");
        track.missing = true;

        let text = hover_text(&track);
        assert!(text.contains("missing"), "{text}");
        assert!(text.contains("/music/gone.mp3"));
    }

    #[test]
    fn drawing_the_library_is_harmless_when_it_is_empty() {
        let mut app = app();
        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn drawing_a_populated_library_shows_every_page() {
        let mut app = imported();
        assert_eq!(app.browse().track_count(), 2);

        for kind in BrowseKind::ALL {
            app.browse_mut().set_kind(kind);
            egui::__run_test_ui(|ui| show(&mut app, ui));
        }
    }

    #[test]
    fn drawing_a_filtered_library_that_matches_nothing_is_harmless() {
        let mut app = imported();
        app.browse_mut().set_query_text("nothing matches this");

        assert!(app.browse().visible_tracks().is_empty());
        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn the_play_field_accepts_a_folder_and_fills_the_queue() {
        let folder = fixture_folder("library-play-field");
        std::fs::write(folder.join("song.mp3"), b"not really audio")
            .expect("the fixture should be writable");
        let mut app = app();

        *app.path_input_mut() = folder.display().to_string();
        let added = app.play_path_input();

        assert_eq!(added, 1);
        assert_eq!(app.playback().queue().len(), 1);
        assert_eq!(app.path_input(), "", "the field clears after a success");
    }

    #[test]
    fn a_folder_with_no_audio_leaves_the_input_alone() {
        let folder = fixture_folder("library-no-audio");
        std::fs::write(folder.join("readme.txt"), b"text").expect("the fixture should be writable");
        let mut app = app();

        *app.path_input_mut() = folder.display().to_string();

        assert_eq!(app.play_path_input(), 0);
        assert!(app.playback().queue().is_empty());
        assert_eq!(
            app.path_input(),
            folder.display().to_string(),
            "a path that added nothing stays put so it can be corrected"
        );
    }

    /// An empty folder of our own, so the tests never share fixture state.
    fn fixture_folder(name: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("musiccraft-{name}"));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("the fixture folder should be creatable");

        folder
    }

    /// An application with an in-memory library and no output device.
    fn app() -> MusicCraftApp {
        MusicCraftApp::with_playback(startup(), playback())
    }

    fn startup() -> Startup {
        Startup {
            config: AppConfig::default(),
            config_path: None,
            error: None,
        }
    }

    fn playback() -> Playback {
        Playback::with_engine(AudioEngine::without_device(), 1.0)
    }

    /// An application whose library already holds two tagged tracks.
    fn imported() -> MusicCraftApp {
        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-view-artwork"))
            .expect("an in-memory library");

        let mut app = MusicCraftApp::with_library(startup(), playback(), library);

        let tracks = [
            tagged(1, "Xtal", "Aphex Twin", "Selected Ambient Works"),
            tagged(2, "Roygbiv", "Boards of Canada", "Music Has the Right"),
        ];
        {
            let library = app.library_mut().expect("a library");
            let repository = library.repository_mut();
            for track in &tracks {
                repository
                    .upsert_track(track)
                    .expect("the track should be stored");
            }
        }

        app.refresh().expect("the library should refresh");

        app
    }

    /// A track with an identifier, ready to be stored.
    fn tagged(id: i64, title: &str, artist: &str, album: &str) -> Track {
        let mut track = Track::from_path(format!("/music/{title}.mp3"));
        track.id = Some(TrackId(id));
        track.title = title.to_owned();
        track.artist = Some(artist.to_owned());
        track.album = Some(album.to_owned());

        track
    }

    /// Every string the Library table paints, in one offscreen frame.
    ///
    /// The table draws its rows by hand rather than laying out widgets, so a
    /// column that is added but never painted looks correct in a layout test
    /// and is invisible in the window. Running the real view into an egui
    /// context and collecting the text shapes is what closes that gap: this is
    /// the same code the window calls, read back as what it drew.
    fn painted_text(app: &mut MusicCraftApp) -> Vec<String> {
        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0));

        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        let mut output = context.run_ui(input, |ui| {
            super::show(app, ui);
        });

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        // Nothing is drawing to a real screen here, so the font atlas the first
        // frame wants uploaded has no home; egui panics if it is dropped
        // unconsumed, and the text has already been read out by now.
        output.textures_delta.clear();

        painted
    }

    #[test]
    fn the_table_paints_a_play_count_beside_each_track() {
        let mut app = imported();
        let id = app.browse().tracks()[0].id.expect("a row");

        // Seven, because the row-number column already paints a bare "1" and a
        // count of one would make this pass without the column existing at all.
        for _ in 0..7 {
            app.library_mut()
                .expect("a library")
                .record_play(id)
                .expect("a play");
        }
        app.refresh().expect("refresh");

        let painted = painted_text(&mut app);

        assert!(
            painted.iter().any(|text| text == "Plays"),
            "the heading should be painted: {painted:?}"
        );
        assert!(
            painted.iter().any(|text| text == "7"),
            "the count should be painted beside its row: {painted:?}"
        );
    }

    #[test]
    fn the_play_count_column_paints_nothing_that_was_not_drawn_before() {
        // The other half of the assertion above: a library nothing has been
        // played from must show a zero, not the heading and a stray number.
        let mut app = imported();

        let painted = painted_text(&mut app);

        assert!(painted.iter().any(|text| text == "0"));
        assert_eq!(
            painted.iter().filter(|text| *text == "0").count(),
            2,
            "one zero per row: {painted:?}"
        );
    }

    #[test]
    fn the_play_count_column_is_never_dropped_as_the_window_narrows() {
        // Artist, album and genre give way to the title; the play count is one
        // of the columns that always stays, like the duration.
        for width in [400.0, 500.0, 760.0, 1000.0, 1600.0] {
            let columns = Columns::for_width(width);

            assert!(
                columns.width_of(Column::Plays) > 0.0,
                "the play count was dropped at {width} points wide"
            );
            assert!(columns.width_of(Column::Duration) > 0.0);
        }
    }
}
