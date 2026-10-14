//! The Queue page: what is lined up to play, and in which order.
//!
//! This is the one place the queue is edited. The Library adds tracks to it and
//! the playback bar moves through it, but putting the list on screen is what
//! makes reordering, removing and clearing possible. Editing the queue never
//! starts or stops anything: the transport keeps running and simply follows the
//! track it was already on wherever it ends up in the list.

use egui::{Align, Button, Layout, RichText};

use crate::app::MusicCraftApp;
use crate::ui::{empty_state, theme};

/// Draw the queue.
pub fn show(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    if app.playback().queue().is_empty() {
        empty_state(
            ui,
            "Nothing queued",
            "Tracks you play or queue from the Library line up here.",
            &[],
        );
        return;
    }

    // Rows carry indices, so editing the list in the middle of the loop would
    // make the rest of the frame refer to the old positions. The row records
    // what was asked for instead, and it is applied once the list is drawn.
    let mut action = None;

    header(app, ui, &mut action);

    for index in 0..app.playback().queue().len() {
        row(app, ui, index, &mut action);
    }

    if let Some(action) = action {
        apply(app, action);
    }
}

/// Something a row asked the queue to do, applied after the frame is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Start the track at this index.
    Play(usize),
    /// Move the track at `from` to `to`.
    Move { from: usize, to: usize },
    /// Take the track at this index out of the queue.
    Remove(usize),
    /// Empty the queue.
    Clear,
}

/// Apply a queued-up row action to the player.
fn apply(app: &mut MusicCraftApp, action: Action) {
    match action {
        Action::Play(index) => app.play_queue_index(index),
        Action::Move { from, to } => {
            app.move_in_queue(from, to);
        }
        Action::Remove(index) => {
            app.remove_from_queue(index);
        }
        Action::Clear => app.clear_queue(),
    }
}

/// How many tracks are waiting, and the button that empties the list.
fn header(app: &MusicCraftApp, ui: &mut egui::Ui, action: &mut Option<Action>) {
    let len = app.playback().queue().len();
    let noun = if len == 1 { "track" } else { "tracks" };

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{len} {noun} queued")).small().weak());

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(Button::new(RichText::new("Clear queue").small()))
                .on_hover_text("Remove every track and stop playback")
                .clicked()
            {
                *action = Some(Action::Clear);
            }
        });
    });

    ui.add_space(6.0);
}

/// One queued track: its place in the queue, its title, and its controls.
fn row(app: &MusicCraftApp, ui: &mut egui::Ui, index: usize, action: &mut Option<Action>) {
    let len = app.playback().queue().len();
    let is_current = app.playback().queue().current_index() == Some(index);
    let title = app.playback().queue().items()[index].title().to_owned();

    ui.horizontal(|ui| {
        // Clicking the title jumps to the track, which is the same gesture as
        // clicking a row with nothing else on it.
        let label = RichText::new(format!("{}. {title}", index + 1));
        let label = if is_current {
            label.strong().color(theme::ACCENT)
        } else {
            label
        };

        let label_response = ui
            .selectable_label(is_current, label)
            .on_hover_text("Play this track");
        theme::focus_ring_if_focused(ui, &label_response);
        if label_response.clicked() {
            *action = Some(Action::Play(index));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let remove_hover = if is_current {
                "Playing now — skip past it before removing it"
            } else {
                "Remove from the queue"
            };
            let remove = ui
                .add_enabled(!is_current, Button::new("✕").small())
                .on_hover_text(remove_hover);
            theme::accessible_name(ui, &remove, "Remove from the queue");
            theme::focus_ring_if_focused(ui, &remove);
            if remove.clicked() {
                *action = Some(Action::Remove(index));
            }

            let down = ui
                .add_enabled(index + 1 < len, Button::new("↓").small())
                .on_hover_text("Move later in the queue");
            theme::accessible_name(ui, &down, "Move later in the queue");
            theme::focus_ring_if_focused(ui, &down);
            if down.clicked() {
                *action = Some(Action::Move {
                    from: index,
                    to: index + 1,
                });
            }

            let up = ui
                .add_enabled(index > 0, Button::new("↑").small())
                .on_hover_text("Move earlier in the queue");
            theme::accessible_name(ui, &up, "Move earlier in the queue");
            theme::focus_ring_if_focused(ui, &up);
            if up.clicked() {
                *action = Some(Action::Move {
                    from: index,
                    to: index - 1,
                });
            }

            if is_current {
                ui.label(RichText::new("Playing").small().color(theme::ACCENT));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback, QueueItem};
    use crate::config::AppConfig;

    #[test]
    fn drawing_the_queue_is_harmless_for_every_row() {
        let mut app = app();
        app.playback_mut().queue_items(items(&["a", "b", "c"]));

        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn drawing_an_empty_queue_is_harmless() {
        let mut app = app();

        egui::__run_test_ui(|ui| show(&mut app, ui));
    }

    #[test]
    fn playing_a_row_targets_that_track() {
        let mut app = app();
        app.playback_mut().queue_items(items(&["a", "b", "c"]));

        apply(&mut app, Action::Play(2));

        assert_eq!(current_title(&app), Some("c"));
    }

    #[test]
    fn moving_a_row_reorders_the_queue_without_changing_the_transport() {
        let mut app = app();
        app.playback_mut().queue_items(items(&["a", "b", "c"]));
        app.stop_playback();
        let transport = app.transport();

        apply(&mut app, Action::Move { from: 2, to: 0 });

        assert_eq!(titles(&app), ["c", "a", "b"]);
        // The cursor tracks the track, not the position: "a" is still current,
        // it has simply moved down one place.
        assert_eq!(current_title(&app), Some("a"));
        assert_eq!(
            app.transport(),
            transport,
            "reordering is not a transport action"
        );
    }

    #[test]
    fn removing_the_playing_track_is_refused() {
        let mut app = app();
        app.playback_mut().queue_items(items(&["a", "b"]));

        assert!(app.playback().queue().current_index().is_some());
        apply(&mut app, Action::Remove(0));
        assert_eq!(titles(&app), ["a", "b"], "the playing track stays");

        apply(&mut app, Action::Remove(1));
        assert_eq!(titles(&app), ["a"]);
    }

    #[test]
    fn clearing_empties_the_queue_and_stops() {
        let mut app = app();
        app.playback_mut().queue_items(items(&["a", "b"]));

        apply(&mut app, Action::Clear);

        assert!(app.playback().queue().is_empty());
        assert_eq!(current_title(&app), None);
    }

    fn items(titles: &[&str]) -> Vec<QueueItem> {
        titles
            .iter()
            .map(|title| QueueItem::new(format!("/music/{title}.mp3"), *title))
            .collect()
    }

    fn titles(app: &MusicCraftApp) -> Vec<&str> {
        app.playback()
            .queue()
            .items()
            .iter()
            .map(QueueItem::title)
            .collect()
    }

    fn current_title(app: &MusicCraftApp) -> Option<&str> {
        app.playback().current().map(QueueItem::title)
    }

    /// An application with no output device and an empty queue.
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
