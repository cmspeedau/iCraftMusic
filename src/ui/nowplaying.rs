//! The bottom playback bar.
//!
//! Stage 1 draws the transport in a disabled state: there is no audio backend
//! attached yet, and showing live controls that silently do nothing would be
//! misleading. Stage 2 enables them once a track can actually be loaded.

use egui::{Align, Button, Layout, Panel, RichText};

use crate::app::{MusicCraftApp, TransportState};

use super::theme;

/// Draw the playback bar across the bottom of the window.
pub fn bar(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let frame = theme::playback_bar_frame(ui.style().as_ref());

    Panel::bottom("musiccraft_playback_bar")
        .exact_size(theme::PLAYBACK_BAR_HEIGHT)
        .frame(frame)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                track_summary(app, ui);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(app.transport().label()).small().weak());
                    ui.add_space(8.0);
                    transport_controls(app, ui);
                });
            });
        });
}

/// Title/artist line on the left of the bar.
fn track_summary(app: &MusicCraftApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| match app.now_playing() {
        Some(track) => {
            ui.label(RichText::new(&track.title).strong());
            let subtitle = format!("{} — {}", track.artist, track.album);
            ui.label(RichText::new(subtitle).small().weak());
        }
        None => {
            ui.label(RichText::new("Nothing playing").strong());
            ui.label(
                RichText::new("Audio playback arrives in Stage 2")
                    .small()
                    .weak(),
            );
        }
    });
}

/// Previous / play-pause / next, disabled until Stage 2 wires up audio.
fn transport_controls(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let ready = app.transport() != TransportState::Unavailable;
    let hint = if ready {
        String::new()
    } else {
        " — no audio backend is attached yet".to_owned()
    };

    // Laid out right-to-left, so the glyphs are listed in reverse order.
    ui.horizontal(|ui| {
        for (glyph, name) in [("⏭", "Next"), ("⏸", "Pause"), ("⏮", "Previous")] {
            let response = ui.add_enabled(ready, Button::new(glyph));
            response.on_hover_text(format!("{name}{hint}"));
        }
    });
}
