//! Left-hand navigation sidebar and the list of top-level sections.
//!
//! Settings is deliberately absent: its entry is the gear at the right end of
//! the status bar, so the sidebar holds only the places you browse, and the
//! sections the app can be in stay a separate list from the ones shown here.

use egui::{Button, Panel, RichText};

use crate::app::MusicCraftApp;
use crate::library::browse::BrowseKind;

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
    pub const TOP: [NavSection; 2] = [Self::Library, Self::Playlists];

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
    Panel::left("musiccraft_sidebar")
        .resizable(false)
        .exact_size(theme::SIDEBAR_WIDTH)
        .frame(theme::sidebar_frame())
        .show(ui, |ui| {
            // The panel paints its own light surface, so the entries drawn on it
            // take the sidebar's ink instead of the theme's foreground.
            ui.scope(|ui| {
                theme::apply_sidebar_visuals(&mut ui.style_mut().visuals);
                sidebar_items(app, ui);
            });
        });
}

/// The sidebar's entries: the section list and the Library's pages.
fn sidebar_items(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.label(RichText::new("Browse").small().weak());
    ui.add_space(4.0);

    for section in NavSection::TOP {
        let selected = app.section() == section;
        if nav_item(ui, section, selected) {
            app.set_section(section);
        }

        // The Library's pages hang off its entry, so the sidebar shows
        // where you are inside it as well as which section is open.
        if selected && section == NavSection::Library {
            browse_items(app, ui);
        }
    }
}

/// Draw a single sidebar entry, returning `true` when it was clicked.
fn nav_item(ui: &mut egui::Ui, section: NavSection, selected: bool) -> bool {
    let size = [ui.available_width(), theme::NAV_ITEM_HEIGHT];
    let button = Button::selectable(selected, RichText::new(section.label()));

    ui.add_sized(size, button)
        .on_hover_text(section.description())
        .clicked()
}

/// The Library's pages, drawn indented under its sidebar entry.
///
/// Clicking one switches the page; the Library section is already open when
/// these are drawn, so no section change is needed.
fn browse_items(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.indent("library_browse", |ui| {
        for kind in BrowseKind::ALL {
            let selected = app.browse().kind() == kind;
            let size = [ui.available_width(), theme::NAV_ITEM_HEIGHT];
            let button = Button::selectable(selected, RichText::new(kind.label()));

            if ui.add_sized(size, button).clicked() {
                app.browse_mut().set_kind(kind);
            }
        }
    });
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

    #[test]
    fn the_sidebar_lists_the_places_you_browse() {
        assert_eq!(
            NavSection::TOP,
            [NavSection::Library, NavSection::Playlists]
        );
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

    #[test]
    fn drawing_the_sidebar_is_harmless_for_every_section() {
        let mut app = app();

        for section in NavSection::ALL {
            app.set_section(section);
            egui::__run_test_ui(|ui| sidebar(&mut app, ui));
        }
    }

    #[test]
    fn drawing_the_library_pages_is_harmless_for_every_page() {
        let mut app = app();
        app.set_section(NavSection::Library);

        for kind in BrowseKind::ALL {
            app.browse_mut().set_kind(kind);
            egui::__run_test_ui(|ui| browse_items(&mut app, ui));
        }
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
}
