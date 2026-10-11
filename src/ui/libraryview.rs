//! The Library section.
//!
//! Stage 1 renders the empty state only. Scanning folders, reading tags and
//! building the artist/album/track tree are Stage 3 work.

use super::empty_state;

/// Draw the library pane.
pub fn show(ui: &mut egui::Ui) {
    empty_state(
        ui,
        "Your library is empty",
        "Adding music folders is part of Stage 3.",
        &[
            "Pick the folders that hold your music",
            "MusicCraft reads tags and artwork automatically",
            "Anything new you drop in is picked up on the next scan",
        ],
    );
}
