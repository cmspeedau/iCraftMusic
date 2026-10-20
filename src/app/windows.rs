//! Which panes are open: the section shown, the queue, Settings,
//! the equalizer and the gear menu.

use super::MusicCraftApp;
use crate::ui::navigation::NavSection;

impl MusicCraftApp {
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

    /// Whether the Settings window is showing.
    pub fn settings_open(&self) -> bool {
        self.settings_open
    }

    /// Raise the Settings window, opening it if it is closed.
    ///
    /// The window is a floating panel, not a section: opening it leaves the
    /// Music or Playlists page underneath untouched, the way the Edit-details
    /// editor does.
    pub fn open_settings(&mut self) {
        self.settings_open = true;
    }

    /// Close the Settings window.
    pub fn close_settings(&mut self) {
        self.settings_open = false;
    }

    /// Whether the Equalizer window is showing.
    pub fn equalizer_open(&self) -> bool {
        self.equalizer_open
    }

    /// Open the Equalizer window, or close it if it is already up.
    ///
    /// The equalizer rides in its own window rather than a settings pane so it
    /// can stay open beside the music while a band is trimmed by ear — a pane
    /// inside a page you have to keep scrolling back to cannot do that.
    pub fn toggle_equalizer(&mut self) {
        self.equalizer_open = !self.equalizer_open;
    }

    /// Close the Equalizer window.
    pub fn close_equalizer(&mut self) {
        self.equalizer_open = false;
    }

    /// Whether the gear on the status bar is showing its menu.
    #[must_use]
    pub fn gear_menu_open(&self) -> bool {
        self.gear_menu_open
    }

    /// Open or close the gear's menu.
    pub fn toggle_gear_menu(&mut self) {
        self.gear_menu_open = !self.gear_menu_open;
    }

    /// Close the gear's menu.
    pub fn close_gear_menu(&mut self) {
        self.gear_menu_open = false;
    }
}
