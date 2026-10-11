//! Left-hand navigation sidebar and the list of top-level sections.

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
    pub const TOP: [NavSection; 2] = [Self::Library, Self::Playlists];

    /// Sections pinned to the bottom of the sidebar by their own heading.
    pub const BOTTOM: [NavSection; 1] = [Self::Settings];

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
    let frame = theme::sidebar_frame(ui.style().as_ref());

    Panel::left("musiccraft_sidebar")
        .resizable(false)
        .exact_size(theme::SIDEBAR_WIDTH)
        .frame(frame)
        .show(ui, |ui| {
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

            // Pin the remaining sections (Settings) to the foot of the sidebar so
            // navigation options stay at the top and preferences sit out of the way.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.add_space(2.0);
                for section in NavSection::BOTTOM {
                    let selected = app.section() == section;
                    if nav_item(ui, section, selected) {
                        app.set_section(section);
                    }
                }
            });
        });
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
    fn top_and_bottom_partition_every_section() {
        let mut seen: Vec<NavSection> = NavSection::TOP
            .into_iter()
            .chain(NavSection::BOTTOM)
            .collect();
        seen.sort_by_key(|s| NavSection::ALL.iter().position(|a| a == s));

        assert_eq!(seen, NavSection::ALL.to_vec());
    }

    #[test]
    fn settings_sits_in_the_bottom_group() {
        assert!(NavSection::BOTTOM.contains(&NavSection::Settings));
        assert!(!NavSection::TOP.contains(&NavSection::Settings));
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
