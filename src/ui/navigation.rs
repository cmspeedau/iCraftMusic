//! Left-hand navigation sidebar and the list of top-level sections.
//!
//! The sidebar holds the places you browse. Under the 'Browse' heading is the
//! Library; under a 'Playlists' heading of its own, level with it, are the
//! playlists themselves. There is no Playlists *entry*: the
//! section is about one of the user's lists at a time and nothing else, so a
//! row for it led to an empty page whose only content was a line asking you to
//! choose one — which is the list drawn right under the row. The heading stays
//! because a list of names needs a name; what went is the row.
//!
//! The Library's three pages — Tracks, Albums and Artists — used to hang here,
//! indented under the Library entry, and they no longer do: they are buttons on
//! the playback bar now, where the icons that tell them apart (a note, a jewel
//! case, a person) have the room the sidebar's rows did not. The sidebar keeps
//! the one row that is a place rather than a page.
//!
//! The gear at the right end of the status bar is the one destination that is
//! not here — Settings is opened from there, so the sections the app can be in
//! stay a separate list from the ones shown.
//!
//! Clicking a playlist opens the Playlists section on it. That is the one
//! action here that is not just a section switch, so the list reports what was
//! clicked and the sidebar applies it once it has finished reading the app.
//!
//! The list wears iTunes 7's source-list dress: grey capital headings, a flat
//! blue bar — lit along its top, deepening downward — across the panel's whole
//! width for whichever row is open and a paler one under the pointer's row, a
//! note for the library, ruled cards for the playlists, and a hairline between
//! the panel and the page beside it. The dress is hand-painted; the clicks,
//! Tab stops and spoken names stay with the egui widgets underneath it.

use egui::{Button, Color32, Panel, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Vec2};

use crate::app::MusicCraftApp;
use crate::library::PlaylistId;

use super::theme;

/// A top-level area of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavSection {
    /// The user's music library.
    #[default]
    Library,
    /// User-created playlists.
    Playlists,
    /// Application preferences.
    Settings,
}

impl NavSection {
    /// Every section, in the order shown in the sidebar.
    pub const ALL: [NavSection; 3] = [Self::Library, Self::Playlists, Self::Settings];

    /// Sections pinned to the top of the sidebar, under the "Browse" heading.
    ///
    /// Settings is not among them: it is opened from the gear in the status
    /// bar, and `ALL` is what the app can be in rather than what is listed.
    /// Playlists is not either: its section only ever shows one of the lists
    /// drawn under it, so the entry was a row that led nowhere of its own —
    /// the lists themselves are the way in, along with the `+` that makes one.
    pub const TOP: [NavSection; 1] = [Self::Library];

    /// Name shown in the sidebar.
    pub fn label(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Playlists => "Playlists",
            Self::Settings => "Settings",
        }
    }

    /// One-line description shown under the content heading.
    pub fn description(self) -> &'static str {
        match self {
            Self::Library => "Your music, by artist, album and track",
            Self::Playlists => "Collections you have put together",
            Self::Settings => "Appearance and playback preferences",
        }
    }
}

/// Draw the sidebar and switch sections when an entry is clicked.
pub fn sidebar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    // Which playlist a click in the list asked for, applied once the list is no
    // longer being read — the app cannot be borrowed mutably while it is.
    let mut open: Option<PlaylistId> = None;

    let panel = Panel::left("musiccraft_sidebar")
        .resizable(false)
        .exact_size(theme::SIDEBAR_WIDTH)
        .frame(theme::sidebar_frame())
        .show(ui, |ui| {
            // The panel paints its own light surface, so the entries drawn on it
            // take the sidebar's ink instead of the theme's foreground.
            ui.scope(|ui| {
                theme::apply_sidebar_visuals(&mut ui.style_mut().visuals);
                sidebar_items(app, ui, &mut open);
            });
        });

    // The source list's one divider: a hairline drawn over the panel's own
    // right edge after the panel has painted, so it stands on the fill rather
    // than under it. Without a line the pale panel and the page beside it end
    // in nothing.
    let edge = panel.response.rect;
    ui.painter().vline(
        edge.right() - 0.5,
        edge.y_range(),
        Stroke::new(1.0, theme::SIDEBAR_EDGE),
    );

    if let Some(id) = open {
        app.set_section(NavSection::Playlists);
        app.open_playlist(id);
    }
}

/// The sidebar's entries: the places you browse, and the playlists under a
/// heading of their own.
fn sidebar_items(app: &mut MusicCraftApp, ui: &mut egui::Ui, open: &mut Option<PlaylistId>) {
    ui.add_space(2.0);
    heading(ui, "Browse");

    // Read once: a click below is about to change it, and the playlists are
    // drawn from the section that was showing while this frame was composed.
    let section = app.section();

    // The rows sit nearly flush, as the source list's did: the bars read as
    // bands across the panel because the next row starts on the last one's
    // heels, not an indent-step and a half below it.
    ui.spacing_mut().item_spacing.y = 1.0;

    for place in NavSection::TOP {
        let selected = section == place;
        if source_row(
            ui,
            place.label(),
            RowIcon::Note,
            place.description(),
            selected,
            theme::NAV_ITEM_HEIGHT,
        ) {
            app.set_section(place);
        }
    }

    playlist_items(app, ui, open, section);
}

/// A group's heading: grey capitals, level with the rows under it.
///
/// iTunes 7 named its source-list groups — "LIBRARY", "DEVICES" — in grey
/// capitals rather than in the list's own ink, so a heading reads as a name
/// for what follows and not as one more row waiting to be clicked. The
/// player's headings also wore a disclosure triangle; none is worn here
/// because a triangle would promise a collapse this sidebar does not do.
fn heading(ui: &mut egui::Ui, word: &str) {
    ui.label(
        RichText::new(word.to_uppercase())
            .small()
            .strong()
            .color(theme::SIDEBAR_HEADING),
    );
    ui.add_space(4.0);
}

/// The playlists, under a heading that sits level with "Browse".
///
/// A click on one reports the playlist through `open`; switching to the section
/// and opening it happens once the sidebar has finished reading the app. The
/// list is capped and scrolls in place — the sidebar itself does not scroll, so
/// an uncapped list would run out of the window and take the rest of the
/// sidebar with it.
///
/// The current section is passed in rather than read off the app, because the
/// playlist the section has open keeps its bar only while the section is the
/// one showing: leaving it leaves the row resting like the others.
/// `selected_playlist` deliberately survives leaving the section — it is what
/// reopens the playlist you were on when you come back — so a bar keyed on
/// that alone would fill a playlist row while the Library's own bar is full,
/// and the panel would read as two places open at once.
fn playlist_items(
    app: &MusicCraftApp,
    ui: &mut egui::Ui,
    open: &mut Option<PlaylistId>,
    section: NavSection,
) {
    ui.add_space(12.0);
    heading(ui, "Playlists");

    if app.playlists().is_empty() {
        // The placeholder is a row that is not there, so its line begins
        // where the rows' labels do — not back at the panel's edge.
        ui.horizontal(|ui| {
            ui.add_space(theme::SIDEBAR_LABEL_INSET);
            ui.label(RichText::new("None yet").small().weak());
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("sidebar_playlists_scroll")
        .max_height(theme::PLAYLIST_LIST_MAX_HEIGHT)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for playlist in app.playlists() {
                let selected = section == NavSection::Playlists
                    && app.selected_playlist() == Some(playlist.id);

                if source_row(
                    ui,
                    playlist.display_name(),
                    RowIcon::Card,
                    &playlist.totals(),
                    selected,
                    theme::PLAYLIST_ITEM_HEIGHT,
                ) {
                    *open = Some(playlist.id);
                }
            }
        });
}

/// One row of the source list, painted by hand the way the playback bar's
/// buttons are: whichever row is open wears a flat blue bar lit along its
/// top and spanning the panel's whole width, the pointer's row wears a paler
/// one, and an icon at the left carries the label's column.
///
/// The click, the Tab stop and the spoken name stay egui's: a full-width
/// button lies over the painted surface with its fill and stroke overridden to
/// nothing for every state it walks through. Painting the bars can therefore
/// never fight the button over the same pixels, and — unlike dyeing the shared
/// style — overriding one button's paint costs the scroll bar's handle nothing.
///
/// The bars read the pointer directly rather than a `Response`, for the queue
/// button's reason: a bar is painted before the button whose face it is, and
/// the same input says the same thing a frame later when the button reads it.
fn source_row(
    ui: &mut egui::Ui,
    name: &str,
    icon: RowIcon,
    note: &str,
    selected: bool,
    height: f32,
) -> bool {
    let (row, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

    let hovered = ui
        .input(|input| input.pointer.hover_pos())
        .is_some_and(|pos| row.contains(pos));

    if selected {
        ui.painter().add(egui::epaint::Shape::gradient_rect(
            row,
            egui::epaint::Direction::TopDown,
            [theme::SIDEBAR_SELECTED_TOP, theme::SIDEBAR_SELECTED_BOTTOM],
        ));
    } else if hovered {
        ui.painter().add(egui::epaint::Shape::gradient_rect(
            row,
            egui::epaint::Direction::TopDown,
            [theme::SIDEBAR_HOVER_TOP, theme::SIDEBAR_HOVER_BOTTOM],
        ));
    }

    icon.paint(ui, row, selected);

    // The label steps onto white exactly when the blue arrives under it, the
    // way the source list's did.
    let ink = if selected {
        theme::SIDEBAR_INK_SELECTED
    } else {
        theme::SIDEBAR_INK
    };

    // The label begins past the icon column, and the row's height is the
    // button's minimum, which is what centres the label inside the bar.
    ui.style_mut().spacing.button_padding = Vec2::new(theme::SIDEBAR_LABEL_INSET, 0.0);

    let response = ui
        .put(
            row,
            Button::selectable(selected, ())
                .left_text(RichText::new(name).color(ink))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .min_size(Vec2::new(0.0, height)),
        )
        .on_hover_text(note);
    // The ring is the part egui never draws — the only sign Tab has stopped
    // on the row.
    theme::focus_ring_if_focused(ui, &response);

    response.clicked()
}

/// The two marks the sidebar paints where the player's icon artwork sat: a
/// note for the library itself, and a ruled card for each playlist — the same
/// two ideas its source list carried, without its bitmaps.
enum RowIcon {
    Note,
    Card,
}

impl RowIcon {
    /// Paint the mark in the row's icon column.
    ///
    /// The note takes the row's ink, which turns it white on the selection
    /// bar: the blue it is drawn in would sink into the blue of the bar, and
    /// the player kept its note readable there with a white edge that the
    /// same swap stands in for. The card's paper stays paper — white with
    /// blue ruling reads on the pale panel and on the deep bar alike.
    fn paint(self, ui: &mut egui::Ui, row: Rect, selected: bool) {
        let painter = ui.painter();
        let left = row.left() + theme::SIDEBAR_ICON_INSET;
        let middle = row.center().y;

        match self {
            RowIcon::Note => {
                let ink = if selected {
                    theme::SIDEBAR_INK_SELECTED
                } else {
                    theme::SIDEBAR_ICON
                };

                // An eighth note — head, stem and flag, the player's mark for
                // the music library. The flag is square where the player's
                // curled; at this size nothing misreads it.
                let head = Pos2::new(left + 3.6, middle + 4.4);
                painter.circle_filled(head, 2.8, ink);

                let stem = Rect::from_min_max(
                    Pos2::new(head.x + 2.6, middle - 7.2),
                    Pos2::new(head.x + 4.0, head.y),
                );
                painter.rect_filled(stem, 0.0, ink);
                painter.rect_filled(
                    Rect::from_min_max(
                        stem.right_top(),
                        Pos2::new(stem.right() + 4.4, stem.top() + 2.8),
                    ),
                    0.0,
                    ink,
                );
            }
            RowIcon::Card => {
                let card =
                    Rect::from_min_size(Pos2::new(left, middle - 7.2), Vec2::new(11.0, 14.4));
                painter.rect_filled(card, 2.0, Color32::WHITE);
                painter.rect_stroke(
                    card,
                    2.0,
                    Stroke::new(1.0, theme::SIDEBAR_ICON),
                    egui::StrokeKind::Inside,
                );
                for line in 0..3u8 {
                    painter.hline(
                        card.x_range().shrink(2.6),
                        card.top() + 3.6 + f32::from(line) * 3.6,
                        Stroke::new(1.0, theme::SIDEBAR_ICON),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;

    #[test]
    fn library_is_the_default_section() {
        assert_eq!(NavSection::default(), NavSection::Library);
    }

    #[test]
    fn every_section_is_in_all_exactly_once() {
        for section in NavSection::ALL {
            let count = NavSection::ALL.iter().filter(|s| **s == section).count();
            assert_eq!(count, 1, "{section:?} appears more than once");
        }
    }

    /// The sidebar lists the one place it holds. The Library's three pages —
    /// Tracks, Albums and Artists — are buttons on the playback bar now, so a
    /// page name painted here would be the indented list coming back.
    #[test]
    fn the_sidebar_lists_the_library_and_no_pages() {
        assert_eq!(NavSection::TOP, [NavSection::Library]);

        let mut app = imported();
        app.set_section(NavSection::Library);

        let painted = painted(&mut app);

        for page in ["Tracks", "Albums", "Artists"] {
            assert!(
                !painted.iter().any(|text| text == page),
                "{page} is still listed in the sidebar: {painted:?}"
            );
        }
    }

    #[test]
    fn settings_has_no_sidebar_entry() {
        assert!(
            !NavSection::TOP.contains(&NavSection::Settings),
            "the status bar's gear opens Settings now, not the sidebar"
        );
        assert!(
            NavSection::ALL.contains(&NavSection::Settings),
            "…but the app can still be in Settings"
        );
    }

    /// The Playlists section is one of the user's own lists or nothing, so the
    /// sidebar reaches it through the named lists rather than through a row for
    /// the section itself.
    #[test]
    fn the_playlists_have_no_entry_of_their_own() {
        assert!(
            !NavSection::TOP.contains(&NavSection::Playlists),
            "the lists under the heading are the way into the section"
        );
        assert!(
            NavSection::ALL.contains(&NavSection::Playlists),
            "…but the app can still be in the Playlists section, opened from a list"
        );
    }

    /// The word 'Playlists' is painted once on an imported library: as the
    /// heading over the list, not as a section entry above it. Two copies is
    /// the entry coming back; the literal is uppercase because the heading is
    /// the source list's grey capitals, and the absence of the mixed-case
    /// word is what proves no row for the section itself came back.
    #[test]
    fn the_sidebar_names_the_playlists_once_over_the_list() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let painted = painted(&mut app);

        assert_eq!(
            painted
                .iter()
                .filter(|text| text.as_str() == "PLAYLISTS")
                .count(),
            1,
            "the sidebar names the playlists more than once, so an entry is back \
             above the heading: {painted:?}"
        );
        assert!(
            !painted.iter().any(|text| text == "Playlists"),
            "a section entry wearing the heading's word came back: {painted:?}"
        );
    }

    #[test]
    fn drawing_the_sidebar_is_harmless_for_every_section() {
        let mut app = app();

        for section in NavSection::ALL {
            app.set_section(section);
            egui::__run_test_ui(|ui| sidebar(&mut app, ui));
        }
    }

    /// Every string the sidebar paints, in one offscreen frame.
    ///
    /// The same trick the Library table uses: run the real view into an egui
    /// context and read back the text it actually drew, so a list that is
    /// composed but never painted cannot pass.
    fn painted(app: &mut MusicCraftApp) -> Vec<String> {
        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0));
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        let mut output = context.run_ui(input, |ui| sidebar(app, ui));
        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        // Nothing here draws to a real screen, so the font atlas the first
        // frame wants uploaded has no home and egui panics if it is dropped
        // unconsumed; the text has been read out by now.
        output.textures_delta.clear();

        painted
    }

    #[test]
    fn the_playlists_are_listed_under_their_heading() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let painted = painted(&mut app);

        assert!(
            painted.iter().any(|text| text == "PLAYLISTS"),
            "the heading is missing: {painted:?}"
        );
        assert!(
            painted.iter().any(|text| text == "Evening"),
            "the playlist is not in the sidebar: {painted:?}"
        );
    }

    #[test]
    fn the_playlists_are_listed_whatever_section_is_open() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        // The list is the point of putting it here: it is reachable without the
        // Playlists section being the one on screen.
        app.set_section(NavSection::Library);
        let from_library = painted(&mut app);

        app.set_section(NavSection::Playlists);
        let from_playlists = painted(&mut app);

        for (section, painted) in [("Library", from_library), ("Playlists", from_playlists)] {
            assert!(
                painted.iter().any(|text| text == "Evening"),
                "no playlist in the sidebar while {section} is open: {painted:?}"
            );
        }
    }

    #[test]
    fn an_empty_library_says_there_are_no_playlists_rather_than_going_quiet() {
        let mut app = app();

        let painted = painted(&mut app);

        assert!(
            painted.iter().any(|text| text == "None yet"),
            "the sidebar says nothing about playlists: {painted:?}"
        );
    }

    /// A playlist is a place you go, so its label lines up with the section
    /// entries' rather than sitting inset under the heading above it.
    #[test]
    fn a_playlist_is_level_with_the_section_entries() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let shapes = frame(&mut app, None);

        // The Library is the one entry left under the Browse heading; a
        // playlist's label has to line up with it rather than with the
        // heading over it.
        let section = painted_name(&shapes, "Library").expect("no Library entry");
        let entry = painted_name(&shapes, "Evening").expect("no playlist entry");

        // The source list hangs every label off the same line, at the right
        // edge of the icon column, so it is the labels' left edges that line
        // up now — an indented entry would start a whole indent step in.
        assert!(
            (entry.left() - section.left()).abs() <= 1.0,
            "the playlist's label is indented under the Library's: {entry:?} vs {section:?}"
        );
        assert!(
            entry.top() > section.top(),
            "the playlist is listed above the Library entry: {entry:?} vs {section:?}"
        );
    }

    /// The selection is not a button inset in the panel but a bar across it:
    /// as wide as the panel's inside, both ends of the ramp on it, and under
    /// the open row's label. And it is the open row's alone — nothing is lit
    /// without a selection or a pointer.
    #[test]
    fn the_open_row_wears_a_bar_across_the_whole_panel() {
        let mut app = imported(); // the Library is the open row

        let shapes = frame(&mut app, None);
        let selected = bars(
            &shapes,
            [theme::SIDEBAR_SELECTED_TOP, theme::SIDEBAR_SELECTED_BOTTOM],
        );

        assert_eq!(selected.len(), 1, "the open row wears exactly one bar");

        // The panel's inside: its width less the frame's eight-point margins.
        let inside = theme::SIDEBAR_WIDTH - 16.0;
        assert!(
            (selected[0].width() - inside).abs() <= 2.0,
            "the bar spans {:.1} points instead of the panel's {inside}-point inside",
            selected[0].width(),
        );

        let label = painted_name(&shapes, "Library").expect("the Library row");
        assert!(
            selected[0].contains(label.center()),
            "the bar is not under the row it selects"
        );

        let hovering = bars(
            &shapes,
            [theme::SIDEBAR_HOVER_TOP, theme::SIDEBAR_HOVER_BOTTOM],
        );
        assert!(
            hovering.is_empty(),
            "a ramp is lit with neither a selection nor a pointer"
        );
    }

    /// The pointer's ramp is the paler one, it lies only under the pointed-at
    /// row, and — painting a hover being no click — it moves nothing.
    #[test]
    fn a_pointed_at_row_lights_the_paler_ramp_only_under_it() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        app.set_section(NavSection::Playlists); // nothing open: any ramp is the pointer's

        let shapes = frame(&mut app, None);
        let row = painted_name(&shapes, "Evening").expect("the playlist row");

        let hovered = frame(&mut app, Some(row.center()));
        let ramps = bars(
            &hovered,
            [theme::SIDEBAR_HOVER_TOP, theme::SIDEBAR_HOVER_BOTTOM],
        );
        assert_eq!(ramps.len(), 1, "one row pointed at, one pale ramp");
        assert!(
            ramps[0].contains(row.center()),
            "the ramp is lit under a different row"
        );

        let selected = bars(
            &hovered,
            [theme::SIDEBAR_SELECTED_TOP, theme::SIDEBAR_SELECTED_BOTTOM],
        );
        assert!(selected.is_empty(), "hovering selected a row");
        assert_eq!(
            app.section(),
            NavSection::Playlists,
            "hovering switched the section"
        );
    }

    /// The label steps onto white exactly when the blue arrives under it,
    /// and a resting label keeps the panel's own ink.
    #[test]
    fn the_open_label_turns_white_on_the_bar() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let shapes = frame(&mut app, None); // the Library is the open row

        assert_eq!(
            label_colour(&shapes, "Library"),
            theme::SIDEBAR_INK_SELECTED,
            "the open row's label did not turn white on the blue"
        );
        assert_eq!(
            label_colour(&shapes, "Evening"),
            theme::SIDEBAR_INK,
            "a resting row's label left the panel's ink"
        );
    }

    /// The library's row carries a note and a playlist's a ruled card, each in
    /// the column left for icons — before the labels' line, level with its
    /// row — and the note turns white on the bar the label's white steps onto,
    /// back to its own blue when the row rests.
    #[test]
    fn each_row_kind_carries_its_mark() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        // Library open: the note is on the bar, so it wears white.
        let shapes = frame(&mut app, None);
        let library = painted_name(&shapes, "Library").expect("the Library row");
        let note = shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Circle(circle) => Some(circle),
                _ => None,
            })
            .expect("the library row carries no note");
        assert_eq!(
            note.fill,
            theme::SIDEBAR_INK_SELECTED,
            "the note on the bar is not white"
        );
        assert!(
            note.center.x < library.left(),
            "the note sits on the label's side of the row"
        );
        assert!(
            (note.center.y - library.center().y).abs() <= 6.0,
            "the note is not level with its row"
        );

        // Leave the Library and the note is its own blue again.
        app.set_section(NavSection::Playlists);
        let resting = frame(&mut app, None);
        assert!(
            resting.iter().any(|clipped| matches!(
                &clipped.shape,
                egui::Shape::Circle(circle) if circle.fill == theme::SIDEBAR_ICON
            )),
            "a resting library row's note lost its own blue"
        );

        // The playlist's card: paper before the label's line.
        let evening = painted_name(&resting, "Evening").expect("the playlist row");
        let card = resting
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == egui::Color32::WHITE && rect.rect.width() < 14.0 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("the playlist row carries no card");
        assert!(
            card.right() <= evening.left(),
            "the card sits on the label's side of the row"
        );
        assert!(
            (card.center().y - evening.center().y).abs() <= 3.0,
            "the card is not level with its label"
        );
    }

    /// The headings are the source list's grey capitals — they name what
    /// follows and do not invite the click a list row waits for.
    #[test]
    fn the_headings_are_grey_capitals() {
        let mut app = app();

        let shapes = frame(&mut app, None);

        for word in ["BROWSE", "PLAYLISTS"] {
            assert_eq!(
                label_colour(&shapes, word),
                theme::SIDEBAR_HEADING,
                "{word} is not painted in the heading grey"
            );
        }
    }

    /// The panel keeps its one divider against the page: a vertical hairline
    /// in the edge colour, on the panel's own edge, and one only.
    #[test]
    fn the_sidebar_divides_itself_from_the_page() {
        let mut app = app();

        let shapes = frame(&mut app, None);

        let dividers: Vec<[egui::Pos2; 2]> = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::LineSegment { points, stroke }
                    if stroke.color == theme::SIDEBAR_EDGE =>
                {
                    Some(*points)
                }
                _ => None,
            })
            .collect();

        assert_eq!(dividers.len(), 1, "the panel has exactly one divider");
        let [a, b] = dividers[0];
        assert!((a.x - b.x).abs() <= 0.01, "the divider is not vertical");
        assert!(
            (a.x - (theme::SIDEBAR_WIDTH - 0.5)).abs() <= 1.0,
            "the divider is not on the panel's edge: x = {:.1}",
            a.x
        );
        assert!(
            (a.y - b.y).abs() > 100.0,
            "the divider does not run the panel's height"
        );
    }

    /// The colour a row's or heading's label was painted with — read off the
    /// frame so an ink that only the style believes in cannot pass.
    fn label_colour(shapes: &[egui::epaint::ClippedShape], needle: &str) -> egui::Color32 {
        shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) if text.galley.text() == needle => {
                    Some(text.galley.job.sections[0].format.color)
                }
                _ => None,
            })
            .expect("the label is painted")
    }

    #[test]
    fn clicking_a_playlist_opens_the_section_on_it() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        let id = app.playlists().first().expect("a playlist").id;

        // Start somewhere else, so the click is what moves the section.
        app.set_section(NavSection::Library);
        assert_eq!(app.selected_playlist(), None);

        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0));
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        // Frame 1: lay the sidebar out and find the playlist the way a pointer
        // would, by the text it painted.
        let first = context.run_ui(input.clone(), |ui| sidebar(&mut app, ui));
        let name = painted_name(&first.shapes, "Evening").expect("the sidebar lists no playlist");
        first.drop_without_applying_deltas();

        // Frame 2: press and release on it.
        input.events = vec![egui::Event::PointerMoved(name.center())];
        input.events.push(egui::Event::PointerButton {
            pos: name.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        let down = context.run_ui(input.clone(), |ui| sidebar(&mut app, ui));
        down.drop_without_applying_deltas();

        input.events = vec![egui::Event::PointerButton {
            pos: name.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }];
        let up = context.run_ui(input, |ui| sidebar(&mut app, ui));
        up.drop_without_applying_deltas();

        assert_eq!(
            app.section(),
            NavSection::Playlists,
            "clicking a playlist did not bring its section up"
        );
        assert_eq!(
            app.selected_playlist(),
            Some(id),
            "the section opened without opening the playlist that was clicked"
        );
    }

    /// Leaving the Playlists section takes the selection bar off the row.
    ///
    /// `selected_playlist` deliberately survives leaving the section — it is
    /// what reopens the playlist you were on — so if the row barred up on that
    /// alone, the playlist and the Library entry would both wear a bar at
    /// once, which reads as two places being open. The section is what says
    /// which one is showing.
    #[test]
    fn leaving_the_playlists_takes_the_bar_off_its_row() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        let id = app.playlists().first().expect("a playlist").id;

        app.set_section(NavSection::Playlists);
        app.open_playlist(id);
        app.set_section(NavSection::Library);

        // The state survives, which is the reason the bar cannot key on it.
        assert_eq!(
            app.selected_playlist(),
            Some(id),
            "leaving the section should remember the playlist, not forget it"
        );

        let barred = selection_bars(&mut app, "Evening");
        assert!(
            barred.is_empty(),
            "the playlist row still wears a bar while the Library is open: {barred:?}"
        );

        // And back in its own section it bars over again, so the fix is a
        // guard on the section rather than the bar being dropped altogether.
        app.set_section(NavSection::Playlists);
        let barred = selection_bars(&mut app, "Evening");
        assert!(
            !barred.is_empty(),
            "the open playlist's row wears no bar in its own section"
        );
    }

    /// The selection bars painted over the sidebar row whose text is `needle`,
    /// so "is this row the open one" is answered from the frame rather than
    /// from `selected`, which is the thing under test.
    ///
    /// The bar is a mesh gradient — the source list's ramp, not a widget's
    /// flat fill — so it is found by its two end colours and narrowed to the
    /// one lying under the row's label.
    fn selection_bars(app: &mut MusicCraftApp, needle: &str) -> Vec<egui::Rect> {
        let shapes = frame(app, None);
        let row = painted_name(&shapes, needle).expect("the row is painted");

        bars(
            &shapes,
            [theme::SIDEBAR_SELECTED_TOP, theme::SIDEBAR_SELECTED_BOTTOM],
        )
        .into_iter()
        .filter(|rect| rect.expand(2.0).intersects(row))
        .collect()
    }

    /// The sidebar's shapes from one offscreen frame, with the pointer
    /// optionally parked at `at` first — so a hover-painted frame is drawn
    /// with the pointer where the test means it to be, and the rows' ramps
    /// are read from what was actually painted.
    fn frame(app: &mut MusicCraftApp, at: Option<egui::Pos2>) -> Vec<egui::epaint::ClippedShape> {
        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0));
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        if let Some(at) = at {
            input.events = vec![egui::Event::PointerMoved(at)];
        }

        let mut output = context.run_ui(input, |ui| sidebar(app, ui));
        let shapes = output.shapes;

        // Nothing here draws to a real screen, so the font atlas the frame
        // wants uploaded has no home; the shapes have been read out by now.
        output.textures_delta.clear();

        shapes
    }

    /// Bounds of the gradient meshes painted between a bar's two colours —
    /// the selection's ramp, or the hover's.
    fn bars(
        shapes: &[egui::epaint::ClippedShape],
        [top, bottom]: [egui::Color32; 2],
    ) -> Vec<egui::Rect> {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh)
                    if mesh.vertices.iter().any(|v| v.color == top)
                        && mesh.vertices.iter().any(|v| v.color == bottom) =>
                {
                    Some(mesh.calc_bounds())
                }
                _ => None,
            })
            .collect()
    }

    /// The rectangle the text shape containing `needle` covers.
    ///
    /// A galley that is centred on its line is painted *from* its centre, so its
    /// shape's position is the centre of the line rather than its top left.
    fn painted_name(shapes: &[egui::epaint::ClippedShape], needle: &str) -> Option<egui::Rect> {
        shapes.iter().find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.text() == needle => {
                let size = text.galley.size();
                let min = match text.galley.job.halign {
                    egui::Align::LEFT => text.pos,
                    egui::Align::Center => egui::pos2(text.pos.x - size.x / 2.0, text.pos.y),
                    egui::Align::RIGHT => egui::pos2(text.pos.x - size.x, text.pos.y),
                };
                Some(egui::Rect::from_min_size(min, size))
            }
            _ => None,
        })
    }

    /// An application with an in-memory library and no output device.
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

    /// Every library track's row identifier, as a menu would hand them over.
    fn library_ids(app: &MusicCraftApp) -> Vec<crate::library::TrackId> {
        app.browse()
            .tracks()
            .iter()
            .filter_map(|track| track.id)
            .collect()
    }

    /// An application with two indexed tracks and no playlists.
    fn imported() -> MusicCraftApp {
        use crate::library::{Library, Track, TrackId};

        let library = Library::in_memory(std::env::temp_dir().join("musiccraft-sidebar-artwork"))
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
