//! Which panes are open: the section shown, the queue, Settings,
//! the equalizer and the gear menu.

use super::MusicCraftApp;
use super::settings::{SettingsTab, SettingsUndo};
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
    /// editor does. Opening takes the snapshot its Cancel puts back: the window
    /// writes every change live, so what it opened with is what Cancel returns
    /// to, however many times a slider was moved in between.
    pub fn open_settings(&mut self) {
        self.settings_open = true;
        self.settings_undo = Some(SettingsUndo {
            volume: self.volume(),
            crossfade_seconds: self.crossfade_seconds(),
            output_device: self.chosen_output_device().map(str::to_owned),
        });
    }

    /// Raise the Settings window on a named tab.
    ///
    /// This is what the Help menu does with "Keyboard Shortcuts" and "About":
    /// the question was about one pane, and the window opens on it rather than
    /// on whichever pane the gear happened to leave showing.
    pub fn open_settings_at(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
        self.open_settings();
    }

    /// The tab the Settings window's strip rests on.
    #[must_use]
    pub fn settings_tab(&self) -> SettingsTab {
        self.settings_tab
    }

    /// Move the strip to another tab, the click of the tab itself doing this.
    pub fn set_settings_tab(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
    }

    /// Close the Settings window.
    pub fn close_settings(&mut self) {
        self.settings_open = false;
        self.settings_undo = None;
    }

    /// Close the Settings window, putting back what it changed.
    ///
    /// Cancel returns the three choices the window owns — volume, crossfade,
    /// output device — through the same setters the window itself used, so
    /// undoing goes by the real road and not by scribbling over the config.
    /// A stored device that vanished while the window was open will not come
    /// back; the window already says the chosen device is unavailable.
    pub fn cancel_settings(&mut self) {
        if let Some(undo) = self.settings_undo.take() {
            self.set_volume(undo.volume);
            self.set_crossfade_seconds(undo.crossfade_seconds);
            self.set_output_device(undo.output_device);
        }

        self.close_settings();
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
