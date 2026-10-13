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

use egui::{Button, Panel, RichText, ScrollArea};

use crate::app::MusicCraftApp;
use crate::library::PlaylistId;
use crate::playlists::PlaylistSummary;

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

    Panel::left("musiccraft_sidebar")
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

    if let Some(id) = open {
        app.set_section(NavSection::Playlists);
        app.open_playlist(id);
    }
}

/// The sidebar's entries: the places you browse, and the playlists under a
/// heading of their own.
fn sidebar_items(app: &mut MusicCraftApp, ui: &mut egui::Ui, open: &mut Option<PlaylistId>) {
    ui.add_space(2.0);
    ui.label(RichText::new("Browse").small().weak());
    ui.add_space(4.0);

    for section in NavSection::TOP {
        let selected = app.section() == section;
        if nav_item(ui, section, selected) {
            app.set_section(section);
        }
    }

    playlist_items(app, ui, open);
}

/// Draw a single sidebar entry, returning `true` when it was clicked.
fn nav_item(ui: &mut egui::Ui, section: NavSection, selected: bool) -> bool {
    let size = [ui.available_width(), theme::NAV_ITEM_HEIGHT];
    let button = Button::selectable(selected, RichText::new(section.label()));

    ui.add_sized(size, button)
        .on_hover_text(section.description())
        .clicked()
}

/// The playlists, under a heading that sits level with "Browse".
///
/// A click on one reports the playlist through `open`; switching to the section
/// and opening it happens once the sidebar has finished reading the app. The
/// list is capped and scrolls in place — the sidebar itself does not scroll, so
/// an uncapped list would run out of the window and take the rest of the
/// sidebar with it.
fn playlist_items(app: &MusicCraftApp, ui: &mut egui::Ui, open: &mut Option<PlaylistId>) {
    ui.add_space(12.0);
    ui.label(RichText::new("Playlists").small().weak());
    ui.add_space(4.0);

    if app.playlists().is_empty() {
        ui.label(RichText::new("None yet").small().weak());
        return;
    }

    ScrollArea::vertical()
        .id_salt("sidebar_playlists_scroll")
        .max_height(theme::PLAYLIST_LIST_MAX_HEIGHT)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for playlist in app.playlists() {
                if playlist_item(app, ui, playlist) {
                    *open = Some(playlist.id);
                }
            }
        });
}

/// One playlist in the sidebar. Returns `true` when it was clicked.
///
/// The open playlist carries the accent fill, the same as the selected entry of
/// any other list, so the sidebar shows which one the section is showing.
fn playlist_item(app: &MusicCraftApp, ui: &mut egui::Ui, playlist: &PlaylistSummary) -> bool {
    let selected = app.selected_playlist() == Some(playlist.id);
    let size = [ui.available_width(), theme::PLAYLIST_ITEM_HEIGHT];
    let button = Button::selectable(selected, RichText::new(playlist.display_name()));

    ui.add_sized(size, button)
        .on_hover_text(playlist.totals())
        .clicked()
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
    /// the entry coming back; the count is the literal because that is what the
    /// screen has to show.
    #[test]
    fn the_sidebar_names_the_playlists_once_over_the_list() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let painted = painted(&mut app);

        assert_eq!(
            painted
                .iter()
                .filter(|text| text.as_str() == "Playlists")
                .count(),
            1,
            "the sidebar names the playlists more than once, so an entry is back \
             above the heading: {painted:?}"
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
            painted.iter().any(|text| text == "Playlists"),
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

    /// A playlist is a place you go, so it lines up with the section entries
    /// rather than sitting inset under the heading above it.
    #[test]
    fn a_playlist_is_level_with_the_section_entries() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));

        let context = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0));
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let output = context.run_ui(input, |ui| sidebar(&mut app, ui));

        // The Library is the one entry left under the Browse heading; a
        // playlist has to line up with it rather than with the heading over it.
        let section = painted_name(&output.shapes, "Library").expect("no Library entry");
        let entry = painted_name(&output.shapes, "Evening").expect("no playlist entry");
        output.drop_without_applying_deltas();

        // Both names are centred in their own full-width button, so it is the
        // centres that line up; the left edges differ by the width of the word.
        // An indented entry would sit a whole indent step further right.
        assert!(
            (entry.center().x - section.center().x).abs() <= 1.0,
            "the playlist is indented under the heading: {entry:?} vs {section:?}"
        );
        assert!(
            entry.top() > section.top(),
            "the playlist is listed above the Library entry: {entry:?} vs {section:?}"
        );
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
