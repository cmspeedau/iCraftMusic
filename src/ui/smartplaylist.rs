//! The smart-playlist editor: a name, and the rules that would fill it.
//!
//! The window answers the same question iTunes 7 asked on the same screen —
//! *match [all/any] of the following rules*, with a "limit to" line under it —
//! using only controls the application can honour: every field in a rule's
//! menus names a column the library actually stores, every operator menu lists
//! only what the chosen field can take, and a rule with nothing usable in its
//! value says so under its own row rather than being saved and matching
//! nothing quietly.
//!
//! The draft and its rules live in [`crate::playlists`]; this module draws
//! them and reports which button was pressed, so the half that can be typed
//! stays testable without a window.

use egui::{Align2, ComboBox, RichText, TextEdit, Vec2};

use crate::app::MusicCraftApp;
use crate::playlists::{FieldKind, Limit, LimitOrder, Rule, RuleField, RuleOperator};
use crate::ui::theme;

/// Width of the window, in points.
const WIDTH: f32 = 480.0;
/// Height of a text field.
const FIELD_HEIGHT: f32 = 22.0;

/// Draw the rule editor, when a smart playlist is being made or edited.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if app.smart_draft().is_none() {
        return;
    }

    let mut open = true;
    let mut confirm = false;
    let mut cancel = false;

    let title = app
        .smart_draft()
        .map_or("Smart playlist", |draft| draft.kind().title())
        .to_owned();
    let confirm_label = app
        .smart_draft()
        .map_or("Save", |draft| draft.kind().confirm_label());

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            body(app, ui);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(confirm_label).clicked() {
                    confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });

            // The draft's own last refusal — a name in use, or a rule the
            // database or the draft would not take — comes back beside the
            // buttons rather than closing the window: what was typed is still
            // there, and the next move is obvious.
            if let Some(message) = app.smart_draft().and_then(|draft| draft.error()) {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(message.to_owned())
                        .small()
                        .color(theme::WARNING),
                );
            }
        });

    if confirm {
        app.confirm_smart_edit();
    } else if cancel || !open {
        app.cancel_smart_edit();
    }
}

/// Draw the name field and the rule list.
fn body(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.set_max_width(WIDTH);

    let Some(draft) = app.smart_draft_mut() else {
        return;
    };

    ui.label(RichText::new("Name").small().weak());
    let name = ui.add_sized(
        [ui.available_width(), FIELD_HEIGHT],
        TextEdit::singleline(draft.name_mut()).hint_text("Music I Keep"),
    );

    ui.add_space(8.0);
    rules(draft.rules_mut(), ui);

    // One line under the whole rule list, in the same words the rows use: the
    // draft refuses on save for exactly this, so the warning and the refusal
    // never disagree.
    if let Some(problem) = draft.rules().problem() {
        ui.add_space(4.0);
        ui.label(RichText::new(problem).small().color(theme::WARNING));
    }

    // Enter does what the confirm button does, but only from the name field —
    // inside a rule's value, Enter would be an interruption in the middle of
    // a thought.
    if name.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        app.confirm_smart_edit();
    }
}

/// The rule list itself, from the "match all/any" line down to the `+`.
fn rules(rules: &mut crate::playlists::SmartRules, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Match");

        ComboBox::from_id_salt("musiccraft_smart_match")
            .selected_text(if rules.match_all { "all" } else { "any" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut rules.match_all, true, "all");
                ui.selectable_value(&mut rules.match_all, false, "any");
            });

        ui.label("of the following rules:");
    });

    if rules.rules.is_empty() {
        ui.label(
            RichText::new("No rules yet — as it stands, this playlist holds every track.")
                .small()
                .weak(),
        );
    }

    let mut remove = None;

    for (index, rule) in rules.rules.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            ui.horizontal(|ui| {
                field_menu(rule, ui);
                operator_menu(rule, ui);
                value_field(rule, ui);

                let crossed = ui.small_button("✕");
                theme::accessible_name(ui, &crossed, "Remove this rule");

                if crossed.clicked() {
                    remove = Some(index);
                }
            });
        });
    }

    if let Some(index) = remove {
        rules.rules.remove(index);
    }

    let add = ui.small_button("+");
    theme::accessible_name(ui, &add, "Add rule");

    if add.clicked() {
        rules.rules.push(Rule::new(RuleField::Artist));
    }

    limit(rules, ui);
}

/// The field menu: every column the library keeps rules about.
fn field_menu(rule: &mut Rule, ui: &mut egui::Ui) {
    ComboBox::from_id_salt("musiccraft_smart_rule_field")
        .width(130.0)
        .selected_text(rule.field.label())
        .show_ui(ui, |ui| {
            for field in RuleField::ALL {
                if ui
                    .selectable_label(rule.field == *field, field.label())
                    .clicked()
                {
                    rule.set_field(*field);
                }
            }
        });
}

/// The operator menu: only what the chosen field can take.
fn operator_menu(rule: &mut Rule, ui: &mut egui::Ui) {
    ComboBox::from_id_salt("musiccraft_smart_rule_operator")
        .width(120.0)
        .selected_text(rule.operator.label())
        .show_ui(ui, |ui| {
            for operator in RuleOperator::for_field(rule.field) {
                if ui
                    .selectable_label(rule.operator == operator, operator.label())
                    .clicked()
                {
                    rule.operator = operator;
                }
            }
        });
}

/// The value: words for a text field, a count for a number, and the days for
/// date added with the word after it, so the row reads as a sentence.
fn value_field(rule: &mut Rule, ui: &mut egui::Ui) {
    match rule.field.kind() {
        FieldKind::Text => {
            ui.add_sized(
                [190.0, FIELD_HEIGHT],
                TextEdit::singleline(&mut rule.value).hint_text("Squid"),
            );
        }
        FieldKind::Number => {
            ui.add_sized(
                [70.0, FIELD_HEIGHT],
                TextEdit::singleline(&mut rule.value).hint_text("3"),
            );
        }
        FieldKind::Days => {
            ui.add_sized(
                [56.0, FIELD_HEIGHT],
                TextEdit::singleline(&mut rule.value).hint_text("30"),
            );
            ui.label("days");
        }
    }
}

/// "Limit to N tracks selected by …", which is what keeps a smart playlist a
/// size someone can listen to.
fn limit(rules: &mut crate::playlists::SmartRules, ui: &mut egui::Ui) {
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        let mut ticked = rules.limit.is_some();

        if ui.checkbox(&mut ticked, "Limit to").changed() {
            rules.limit = ticked.then_some(Limit {
                count: 25,
                order: LimitOrder::MostPlayed,
            });
        }

        let Some(limit) = rules.limit.as_mut() else {
            return;
        };

        let mut count = limit.count.to_string();
        ui.add_sized(
            [52.0, FIELD_HEIGHT],
            TextEdit::singleline(&mut count).hint_text("25"),
        );

        if let Ok(number) = count.trim().parse::<i64>() {
            limit.count = number;
        }

        ui.label("tracks selected by");

        ComboBox::from_id_salt("musiccraft_smart_limit_order")
            .selected_text(limit.order.label())
            .show_ui(ui, |ui| {
                for order in LimitOrder::ALL {
                    ui.selectable_value(&mut limit.order, *order, order.label());
                }
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::{MusicCraftApp, Startup};
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::library::{Library, Track, TrackId};

    #[test]
    fn drawing_without_an_editor_is_harmless() {
        let mut app = app();

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(app.smart_draft().is_none());
    }

    #[test]
    fn drawing_the_editor_changes_nothing_until_a_button_is_pressed() {
        let mut app = imported();
        app.start_smart_playlist_create();

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(
            app.smart_draft().is_some(),
            "the editor survives its own frame"
        );
        assert!(app.playlists().is_empty(), "and nothing was written");
    }

    #[test]
    fn drawing_a_rule_editor_with_real_rules_draws_them_without_complaint() {
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Recently played".to_owned();

            let mut played = Rule::new(RuleField::PlayCount);
            played.operator = RuleOperator::GreaterThan;
            played.value = "0".to_owned();
            draft.rules_mut().rules.push(played);
            draft.rules_mut().limit = Some(Limit {
                count: 25,
                order: LimitOrder::RecentlyAdded,
            });
        }

        egui::__run_test_ui(|ui| window(&mut app, ui));

        assert!(app.smart_draft().is_some(), "the editor is still open");
    }

    #[test]
    fn a_saved_editor_makes_a_smart_playlist_that_lists_the_librarys_own_answer() {
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Played".to_owned();

            let mut played = Rule::new(RuleField::PlayCount);
            played.operator = RuleOperator::GreaterThan;
            played.value = "0".to_owned();
            draft.rules_mut().rules.push(played);
        }

        app.confirm_smart_edit();

        assert!(app.smart_draft().is_none(), "the editor closed on success");
        let made = app.playlists().first().expect("a playlist");
        assert!(made.smart);
        assert_eq!(made.track_count, 0, "nothing has been played yet");
        assert!(app.playlist_tracks().is_empty());
    }

    #[test]
    fn a_saved_edit_is_asked_of_the_library_again() {
        // The live half, end to end: make the playlist, index a track that
        // matches, refresh — the open playlist grows without anything being
        // added to it by hand.
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Broadcast only".to_owned();
            draft
                .rules_mut()
                .rules
                .push(Rule::text(RuleField::Artist, "Broadcast"));
        }
        app.confirm_smart_edit();

        let mut extra = Track::from_path("/music/haiphon.mp3");
        extra.title = "Haiphon".to_owned();
        extra.artist = Some("Broadcast".to_owned());
        app.library_mut()
            .expect("a library")
            .repository_mut()
            .upsert_track(&extra)
            .expect("the track should be stored");
        app.refresh().expect("refresh");

        assert_eq!(
            app.playlist_tracks().len(),
            1,
            "the track indexed after the playlist answers to its rules"
        );
    }

    #[test]
    fn editing_rules_offers_the_stored_ones_and_saving_replaces_them() {
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Broadcast only".to_owned();
            draft
                .rules_mut()
                .rules
                .push(Rule::text(RuleField::Artist, "Broadcast"));
        }
        app.confirm_smart_edit();

        let id = app.playlists().first().expect("a playlist").id;
        assert!(
            app.start_smart_playlist_edit(id),
            "the editor opens over the stored rules"
        );

        {
            let draft = app.smart_draft_mut().expect("an editor");
            assert_eq!(
                draft.rules().rules.len(),
                1,
                "it starts from what is stored"
            );
            draft.rules_mut().rules[0].value = "Squid".to_owned();
        }
        app.confirm_smart_edit();

        let rules = {
            let library = app.library_mut().expect("a library");
            crate::playlists::Playlists::new(library.repository_mut())
                .rules(id)
                .expect("rules")
                .expect("smart")
        };
        assert_eq!(rules.rules[0].value, "Squid", "the edit landed");
    }

    #[test]
    fn a_hand_made_playlist_has_no_rules_to_edit_and_says_so() {
        let mut app = imported();
        app.add_tracks_to_playlist("Evening", &library_ids(&app));
        let id = app.playlists().first().expect("a playlist").id;

        assert!(
            !app.start_smart_playlist_edit(id),
            "a hand-made playlist refuses an editor it has nothing for"
        );
        assert!(app.playlist_error().is_some());
    }

    #[test]
    fn tracks_cannot_be_added_to_a_smart_playlist_by_name() {
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Ones".to_owned();
            let mut everything = Rule::text(RuleField::Title, "Track");
            everything.operator = RuleOperator::Contains;
            draft.rules_mut().rules.push(everything);
        }
        app.confirm_smart_edit();

        let ids: Vec<TrackId> = app.browse().tracks().iter().filter_map(|t| t.id).collect();
        assert_eq!(app.add_tracks_to_playlist("Ones", &ids), 0);
        assert!(
            app.playlist_error()
                .is_some_and(|message| message.contains("smart")),
            "{:?}",
            app.playlist_error()
        );
    }

    #[test]
    fn a_half_written_rule_is_refused_where_it_was_written() {
        let mut app = imported();

        app.start_smart_playlist_create();
        {
            let draft = app.smart_draft_mut().expect("an editor");
            *draft.name_mut() = "Broken".to_owned();
            draft.rules_mut().rules.push(Rule::new(RuleField::Artist));
        }

        app.confirm_smart_edit();

        assert!(
            app.smart_draft().is_some(),
            "the editor stays open with what was typed still in it"
        );
        assert_eq!(
            app.smart_draft().expect("an editor").error(),
            Some("needs some text to match against")
        );
        assert!(app.playlists().is_empty(), "nothing was made");
    }

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

    /// Every library track's row identifier.
    fn library_ids(app: &MusicCraftApp) -> Vec<TrackId> {
        app.browse()
            .tracks()
            .iter()
            .filter_map(|track| track.id)
            .collect()
    }

    /// An application with two indexed tracks and no playlists.
    fn imported() -> MusicCraftApp {
        let library =
            Library::in_memory(std::env::temp_dir().join("musiccraft-smart-editor-artwork"))
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
            track.duration = Some(std::time::Duration::from_secs(61));
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
