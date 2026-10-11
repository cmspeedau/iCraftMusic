//! The Playlists section.
//!
//! Playlists arrive in Stage 5; Stage 1 only shows the shell's empty state so
//! the section is reachable and looks finished.

use super::empty_state;

/// Draw the playlists pane.
pub fn show(ui: &mut egui::Ui) {
    empty_state(
        ui,
        "No playlists yet",
        "Creating and editing playlists is part of Stage 5.",
        &[
            "Playlists are stored locally, not in the cloud",
            "Drag tracks in from the library once it is scanned",
            "Smart playlists built from rules come later",
        ],
    );
}
