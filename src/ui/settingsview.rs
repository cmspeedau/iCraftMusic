//! The Settings window.
//!
//! The gear in the status bar opens it and `Ctrl/⌘ ,` brings it up: the
//! preferences ride in their own window rather than a page, because wanting
//! them never means wanting to stop looking at the music — a page made
//! changing one volume slider a trip out of the library and back.
//!
//! The sheet is the measured dialog: 573 by 648, its one dark title strip,
//! a row of tabs standing on the white pane's top edge — the chosen one
//! taller and white, running into the pane through the border it stands
//! on — the pane's groups parted by hairlines, and a footer carrying OK and
//! Cancel. The dialog is light in every theme, the same way the content page
//! is: a dark widget floating on a white sheet is neither theme's.
//!
//! The window's buttons tell the truth about a window that applies
//! everything live. OK closes; Cancel closes and puts back the three choices
//! the window owns — volume, crossfade, output device — which is what its
//! snapshot, taken the moment the window opened, exists for. Everything else
//! here is read-only, or an action already done by the time its button is up
//! again; a scan started is not unstarted by a dismissed dialog, and this one
//! does not pretend it is.
//!
//! Every control writes straight through to [`MusicCraftApp`], which is
//! what marks the settings dirty and schedules the write to disk at the end of
//! the frame. The shortcuts are listed rather than configured: the keys are a
//! fixed part of the interface, and the list is here so a user can find them
//! without hunting through the menus.
//!
//! One pane is for when something is wrong: Diagnostics reports what this
//! copy is, where it keeps everything and what the audio side is playing
//! through, and offers the one check the application can run on its own
//! library. It reads rather than writes — the only action there is asking
//! SQLite to check itself, and even that changes nothing.

use egui::{
    Button, Color32, CornerRadius, FontId, Frame, Rect, RichText, ScrollArea, Sense, Slider,
    Stroke, StrokeKind, TextEdit, UiBuilder, Vec2, Window, WindowDrag, pos2,
};

use crate::app::{MusicCraftApp, SettingsTab};
use crate::audio::MAX_CROSSFADE_SECONDS;
use crate::config::DEFAULT_VOLUME;
use crate::ui::shortcuts::Command;
use crate::ui::theme;

/// Width the Settings window opens at, in points.
///
/// Measured off the reference: its Preferences dialog is 573 across.
const WINDOW_WIDTH: f32 = 573.0;
/// Height the Settings window opens at, in points.
///
/// The other half of that measured dialog — 573×648 — which is why the
/// window is no longer sized against the room the main window has to spare
/// and no longer resizable: the reference dialog is a fixed sheet.
const WINDOW_FALLBACK_HEIGHT: f32 = 648.0;

/// The sheet's rounding: at the top corners only, as measured. The dialog's
/// foot is square on the desk it stands on.
const SHEET_RADIUS: CornerRadius = CornerRadius {
    nw: 6,
    ne: 6,
    sw: 0,
    se: 0,
};
/// The one dark strip, and its type.
const BAR_HEIGHT: f32 = 28.0;
const BAR_FONT: f32 = 13.0;
/// The close mark's hit size, and its distance from the sheet's right edge.
const CLOSE_SIZE: f32 = 16.0;
const CLOSE_RIGHT: f32 = 10.0;

/// Where the tabs stand: their feet make the pane's top edge, and the sheet's
/// face shows for the seven points between the bar and their heads.
const TAB_TOP: f32 = 35.0;
const TAB_LEFT: f32 = 15.0;
const TAB_ON_HEIGHT: f32 = 20.0;
const TAB_OFF_HEIGHT: f32 = 17.0;
const TAB_FONT: f32 = 11.5;
/// A tab's type takes nine points of face on each side…
const TAB_PADDING: f32 = 9.0;
/// …and one point of rule on each side makes the rest of its measured width.
const TAB_BORDER: f32 = 2.0;
/// Joining the tabs, as the reference joins them: the next tab starts one
/// point inside the last one's edge, so the pair draws one rule between them.
const TAB_JOIN: f32 = -1.0;

/// The white pane: fifteen in from the left, fourteen from the right, its
/// head where the tabs' feet are and its foot above the footer.
const PANE_LEFT: f32 = 15.0;
const PANE_RIGHT: f32 = 14.0;
const FOOTER_HEIGHT: f32 = 42.0;
/// The pane's inside, one point inside its rule, and the points the sheet's
/// measured page leaves above and below it.
const CONTENT_INSET: f32 = 1.0;
/// The hairline's measured reach inside a pane: 38 in from the sheet's left,
/// 14 shy of its right — the page's own left is 16, so 22 in, 14 out.
const HAIR_IN: f32 = 22.0;
const HAIR_OUT: f32 = 14.0;

/// The footer: ten points of face above the buttons, and the buttons' own
/// measured places — OK 97 from the right edge, Cancel 14.
const BUTTON_TOP: f32 = 10.0;
const OK_RIGHT: f32 = 97.0;
const OK_WIDTH: f32 = 75.0;
const CANCEL_RIGHT: f32 = 14.0;
const CANCEL_WIDTH: f32 = 76.0;
const BUTTON_HEIGHT: f32 = 21.0;
const BUTTON_FONT: f32 = 11.5;

/// Draw the Settings window, when it is open.
pub fn window(app: &mut MusicCraftApp, ui: &egui::Ui) {
    if !app.settings_open() {
        return;
    }

    // The window opens with its top-left where the middle of a first-run
    // screen leaves room for it, and after that stays wherever the user put
    // it — `default_pos` only speaks on the first opening. An anchor would
    // nail it to the centre and fight every drag, which suits a question but
    // not a window one lives beside.
    let mut open = true;
    Window::new("Settings")
        // The measured dialog's title strip is dark, centred, and carries its
        // own close mark — drawn by `chrome`, not by egui.
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        // No fade: a dialog that blooms into view over its first frames is
        // egui's courtesy, not the measured behaviour, and a half-faded sheet
        // is a sheet whose rules paint at two-thirds strength.
        .fade_in(false)
        // With no title bar there is no title bar to drag by, so the sheet's
        // own face moves it. Every control takes the pointer first: what is
        // left to drag on is face, and face drags like chrome.
        .drag_area(WindowDrag::Anywhere)
        .default_pos([310.0, 90.0])
        .default_size([WINDOW_WIDTH, WINDOW_FALLBACK_HEIGHT])
        // `chrome` paints the measured sheet — face, rule and rounding — itself,
        // so egui's default frame stands down: it would lay a theme-coloured
        // square under a fixed-colour one. What it keeps is the shadow the
        // capture carries, 30 points of blur ten below.
        .frame(Frame::new().inner_margin(0.0).shadow(egui::epaint::Shadow {
            offset: [0, 10],
            blur: 30,
            spread: 0,
            color: Color32::from_black_alpha(115),
        }))
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            chrome(app, ui);
        });

    // The ✕ closes the window; the gear asks for it again, and `Ctrl/⌘ ,`
    // asks in the middle of typing nowhere near it.
    if !open {
        app.close_settings();
    }
}

/// The measured sheet: bar, tabs, pane, footer — and this window's one dark
/// rule that applies to everything below it: it is light in every theme.
fn chrome(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.style_mut().visuals = theme::page_visuals();

    let sheet = ui.max_rect();
    let painter = ui.painter().clone();

    painter.rect_filled(sheet, SHEET_RADIUS, theme::SETTINGS_FACE);
    painter.rect_stroke(
        sheet,
        SHEET_RADIUS,
        Stroke::new(1.0, theme::SETTINGS_EDGE),
        StrokeKind::Inside,
    );

    title_bar(app, ui, sheet);

    // The pane goes down before the tabs: they stand on its edge, so its rule
    // has to be under them for the chosen one to run through it.
    let pane = Rect::from_min_max(
        pos2(
            sheet.left() + PANE_LEFT,
            sheet.top() + TAB_TOP + TAB_ON_HEIGHT,
        ),
        pos2(sheet.right() - PANE_RIGHT, sheet.bottom() - FOOTER_HEIGHT),
    );
    painter.rect_filled(pane, 0.0, Color32::WHITE);
    painter.rect_stroke(
        pane,
        0.0,
        Stroke::new(1.0, theme::SETTINGS_LINE),
        StrokeKind::Inside,
    );

    tabs(app, ui, sheet);

    // The pane's inside, measured: one point inside its rule, and the point
    // the page leaves at its head and foot.
    let content = Rect::from_min_max(
        pos2(pane.left() + CONTENT_INSET, pane.top() + CONTENT_INSET),
        pos2(pane.right() - CONTENT_INSET, pane.bottom() - CONTENT_INSET),
    );
    let tab = app.settings_tab();
    let mut pane_ui = ui.new_child(
        UiBuilder::new()
            .id_salt("musiccraft_settings_pane")
            .max_rect(content),
    );
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut pane_ui, |ui| show_tab(app, ui, tab));
    drop(pane_ui);

    footer(app, ui, sheet);
}

/// The one dark strip: the name centred, and the mark that closes the window.
fn title_bar(app: &mut MusicCraftApp, ui: &egui::Ui, sheet: Rect) {
    let painter = ui.painter().clone();
    let bar = Rect::from_min_max(sheet.min, pos2(sheet.right(), sheet.top() + BAR_HEIGHT));

    painter.rect_filled(
        bar,
        CornerRadius {
            nw: SHEET_RADIUS.nw,
            ne: SHEET_RADIUS.ne,
            sw: 0,
            se: 0,
        },
        theme::SETTINGS_BAR,
    );
    painter.text(
        bar.center(),
        egui::Align2::CENTER_CENTER,
        "Settings",
        FontId::proportional(BAR_FONT),
        theme::SETTINGS_BAR_INK,
    );

    let close = Rect::from_min_max(
        pos2(
            sheet.right() - CLOSE_RIGHT - CLOSE_SIZE,
            bar.center().y - CLOSE_SIZE / 2.0,
        ),
        pos2(
            sheet.right() - CLOSE_RIGHT,
            bar.center().y + CLOSE_SIZE / 2.0,
        ),
    );
    let response = ui.interact(
        close,
        ui.id().with("musiccraft_settings_close"),
        Sense::click(),
    );
    // The mark, drawn rather than typed: the bundled font has no ✕ codepoint,
    // and a typed one shows up on screen as a hollow tofu box. Two strokes
    // across an eight-point square is what the title bar's mark measured like
    // anyway, and it lightens under the pointer the way the reference's does.
    let ink = if response.hovered() {
        Color32::WHITE
    } else {
        theme::SETTINGS_BAR_CLOSE
    };
    let centre = close.center();
    let arm = 4.0;
    let mark = Stroke::new(1.2, ink);
    painter.line_segment(
        [
            pos2(centre.x - arm, centre.y - arm),
            pos2(centre.x + arm, centre.y + arm),
        ],
        mark,
    );
    painter.line_segment(
        [
            pos2(centre.x - arm, centre.y + arm),
            pos2(centre.x + arm, centre.y - arm),
        ],
        mark,
    );
    theme::accessible_name(ui, &response, "Close Settings");

    if response.clicked() {
        app.close_settings();
    }
}

/// The row of tabs, standing on the pane's top edge: the chosen one white and
/// taller, its foot the sheet's own white run through the rule it stands on;
/// the rest pale, shallow and waiting.
fn tabs(app: &mut MusicCraftApp, ui: &mut egui::Ui, sheet: Rect) {
    let painter = ui.painter().clone();
    let font = FontId::proportional(TAB_FONT);
    let chosen = app.settings_tab();
    let feet = sheet.top() + TAB_TOP + TAB_ON_HEIGHT;

    let mut x = sheet.left() + TAB_LEFT;
    let mut chosen_rect = Rect::NOTHING;

    for tab in SettingsTab::ALL {
        let label = tab_label(tab);
        let text_width = ui.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(label.to_owned(), font.clone(), theme::SETTINGS_INK)
                .size()
                .x
        });
        let width = text_width + 2.0 * TAB_PADDING + TAB_BORDER;
        let height = if tab == chosen {
            TAB_ON_HEIGHT
        } else {
            TAB_OFF_HEIGHT
        };
        let rect = Rect::from_min_max(pos2(x, feet - height), pos2(x + width, feet));

        if tab == chosen {
            chosen_rect = rect;
            painter.rect_filled(rect, 0.0, Color32::WHITE);
        } else {
            painter.add(egui::epaint::Shape::gradient_rect(
                rect,
                egui::epaint::Direction::TopDown,
                [theme::SETTINGS_TAB_TOP, theme::SETTINGS_TAB_BOTTOM],
            ));
        }

        // Three sides: a tab's foot is the edge it stands on — the rule, or
        // the chosen one's white where it breaks through.
        let rule = Stroke::new(1.0, theme::SETTINGS_LINE);
        painter.line_segment([rect.left_top(), rect.left_bottom()], rule);
        painter.line_segment([rect.left_top(), rect.right_top()], rule);
        painter.line_segment([rect.right_top(), rect.right_bottom()], rule);

        let response = ui.put(
            rect,
            Button::new(
                RichText::new(label)
                    .size(TAB_FONT)
                    .color(theme::SETTINGS_INK),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE),
        );
        if response.clicked() {
            app.set_settings_tab(tab);
        }

        x += width + TAB_JOIN;
    }

    // The chosen tab's foot, painted after the pane's rule it stands on: the
    // sheet's own white, so the tab and its pane are one surface.
    if chosen_rect != Rect::NOTHING {
        painter.hline(
            chosen_rect.x_range(),
            chosen_rect.bottom() + 0.5,
            Stroke::new(1.0, Color32::WHITE),
        );
    }
}

/// The footer's two buttons: OK, the default, ringed in blue, and Cancel.
/// Both are painted here — a widget fill of egui's would sit over the
/// measured faces like a tile over a keycap.
fn footer(app: &mut MusicCraftApp, ui: &mut egui::Ui, sheet: Rect) {
    let painter = ui.painter().clone();
    let top = sheet.bottom() - FOOTER_HEIGHT + BUTTON_TOP;

    let cancel = Rect::from_min_max(
        pos2(sheet.right() - CANCEL_RIGHT - CANCEL_WIDTH, top),
        pos2(sheet.right() - CANCEL_RIGHT, top + BUTTON_HEIGHT),
    );
    painter.add(egui::epaint::Shape::gradient_rect(
        cancel,
        egui::epaint::Direction::TopDown,
        [theme::SETTINGS_BUTTON_TOP, theme::SETTINGS_BUTTON_BOTTOM],
    ));
    painter.rect_stroke(
        cancel,
        4.0,
        Stroke::new(1.0, theme::SETTINGS_BUTTON_EDGE),
        StrokeKind::Inside,
    );

    // OK first on the sheet but painted over the plain face: the halo stands
    // a point off the ring, the ring is two points of blue, and the face
    // between them runs almost-blue-white.
    let ok = Rect::from_min_max(
        pos2(sheet.right() - OK_RIGHT - OK_WIDTH, top),
        pos2(sheet.right() - OK_RIGHT, top + BUTTON_HEIGHT),
    );
    painter.rect_stroke(
        ok.expand(1.0),
        6.0,
        Stroke::new(1.0, theme::SETTINGS_DEFAULT_HALO),
        StrokeKind::Inside,
    );
    painter.rect_filled(ok, 5.0, theme::SETTINGS_DEFAULT_TOP);
    painter.add(egui::epaint::Shape::gradient_rect(
        ok.shrink(2.0),
        egui::epaint::Direction::TopDown,
        [theme::SETTINGS_DEFAULT_TOP, theme::SETTINGS_DEFAULT_BOTTOM],
    ));
    painter.rect_stroke(
        ok,
        5.0,
        Stroke::new(2.0, theme::SETTINGS_DEFAULT_RING),
        StrokeKind::Inside,
    );

    let response = ui.put(
        ok,
        Button::new(
            RichText::new("OK")
                .size(BUTTON_FONT)
                .color(theme::SETTINGS_INK),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE),
    );
    theme::accessible_name(ui, &response, "Close the Settings window");
    if response.clicked() {
        app.close_settings();
    }

    let response = ui.put(
        cancel,
        Button::new(
            RichText::new("Cancel")
                .size(BUTTON_FONT)
                .color(theme::SETTINGS_INK),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE),
    );
    theme::accessible_name(
        ui,
        &response,
        "Close the Settings window, putting back its changes",
    );
    if response.clicked() {
        app.cancel_settings();
    }
}

/// The words the strip wears for each pane.
fn tab_label(tab: SettingsTab) -> &'static str {
    match tab {
        SettingsTab::Library => "Library",
        SettingsTab::Playback => "Playback",
        SettingsTab::Shortcuts => "Shortcuts",
        SettingsTab::Storage => "Storage",
        SettingsTab::Diagnostics => "Diagnostics",
        SettingsTab::About => "About",
    }
}

/// The measured rule that parts one group of settings from the next: the
/// page's hairline, 22 points in from the sheet's left and 14 shy of its
/// right — shorter on the right than on the left, which is how it measured.
fn hairline(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter().hline(
        (rect.left() + HAIR_IN)..=(rect.right() - HAIR_OUT),
        rect.top() + 0.5,
        Stroke::new(1.0, theme::SETTINGS_HAIR),
    );
}

/// Draw one pane of the Settings sheet.
pub fn show_tab(app: &mut MusicCraftApp, ui: &mut egui::Ui, tab: SettingsTab) {
    match tab {
        SettingsTab::Library => {
            ui.add_space(4.0);
            library_management(app, ui);
        }
        SettingsTab::Playback => {
            ui.add_space(4.0);
            playback(app, ui);
        }
        SettingsTab::Shortcuts => {
            ui.add_space(4.0);
            shortcuts(ui);
        }
        SettingsTab::Storage => {
            ui.add_space(4.0);
            storage(app, ui);
        }
        SettingsTab::Diagnostics => {
            ui.add_space(4.0);
            diagnostics(app, ui);
        }
        SettingsTab::About => {
            ui.add_space(4.0);
            about(app, ui);
        }
    }
}

/// Width of the two buttons beside the media home's path, in points. Both
/// share it so the pair reads as one control group.
const HOME_BUTTON_WIDTH: f32 = 76.0;
/// Width of the button that rescans the watched folders.
const RESCAN_WIDTH: f32 = 68.0;
/// Width of the button that stops a running scan.
const STOP_WIDTH: f32 = 54.0;

/// Playback preferences: how loud, and through what.
fn playback(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Playback").strong());
    ui.add_space(6.0);

    let mut volume = app.volume();
    let response = ui.add(Slider::new(&mut volume, 0.0..=1.0).text("Volume"));
    if response.changed() {
        app.set_volume(volume);
    }

    ui.add_space(2.0);
    ui.label(
        RichText::new(format!("Default volume is {:.0}%.", DEFAULT_VOLUME * 100.0))
            .small()
            .weak(),
    );

    // The three groups the pane runs — level, crossfade, device — parted by
    // the measured hairline rather than the gap a separator leaves.
    ui.add_space(10.0);
    hairline(ui);
    ui.add_space(10.0);
    crossfade(app, ui);

    ui.add_space(10.0);
    hairline(ui);
    ui.add_space(10.0);
    output_device(app, ui);
}

/// How much one track overlaps into the next.
///
/// This is a single control because the two behaviours are two ends of one
/// setting: at zero the next track is queued behind the current one and starts
/// the instant it ends, and above zero the two are mixed for that many seconds
/// instead.
fn crossfade(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    let mut seconds = app.crossfade_seconds();
    let response = ui.add(
        Slider::new(&mut seconds, 0.0..=MAX_CROSSFADE_SECONDS)
            .text("Crossfade (s)")
            .fixed_decimals(1),
    );
    if response.changed() {
        app.set_crossfade_seconds(seconds);
    }

    let hint = if seconds <= 0.0 {
        "Off: tracks change over gaplessly, with no gap.".to_owned()
    } else {
        format!("The last {seconds:.1} s of each track is mixed into the next.")
    };
    ui.label(RichText::new(hint).small().weak());
}

/// Which device the music comes out of, and what it came out at.
///
/// The choice is stored by identifier and honoured on the next run, so a user
/// whose desktop default is the wrong one does not have to fix it twice. The
/// list is only read when this pane is drawn and when the user asks for it
/// again, because enumerating devices asks the audio backend.
fn output_device(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Output device").small().weak());
    ui.add_space(4.0);

    // The devices are copied out of the application before the combo box is
    // built: choosing one calls back into the application, which cannot happen
    // while its own device list is borrowed.
    let devices = app.output_devices().to_vec();
    let chosen = app.chosen_output_device().map(str::to_owned);
    let mut requested: Option<Option<String>> = None;

    ui.horizontal(|ui| {
        let selected = chosen
            .as_ref()
            .and_then(|id| {
                devices
                    .iter()
                    .find(|device| &device.id == id)
                    .map(|device| device.name.clone())
            })
            .unwrap_or_else(|| match &chosen {
                // A stored device that is not in the list: say so rather than
                // silently showing the default, which would look like the
                // preference had been forgotten.
                Some(id) => format!("{id} (not available)"),
                None => DEFAULT_OUTPUT_LABEL.to_owned(),
            });

        egui::ComboBox::from_id_salt("musiccraft_output_device")
            .selected_text(RichText::new(selected).small())
            .width(DEVICE_PICKER_WIDTH)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(chosen.is_none(), DEFAULT_OUTPUT_LABEL)
                    .on_hover_text("Follow the desktop's own output setting")
                    .clicked()
                {
                    requested = Some(None);
                }

                for device in &devices {
                    let label = if device.is_default {
                        format!("{} (system default)", device.name)
                    } else {
                        device.name.clone()
                    };

                    if ui
                        .selectable_label(chosen.as_deref() == Some(&device.id), label)
                        .on_hover_text(&device.id)
                        .clicked()
                    {
                        requested = Some(Some(device.id.clone()));
                    }
                }
            });

        if ui
            .small_button("Rescan")
            .on_hover_text("Read the list of output devices again")
            .clicked()
        {
            app.rescan_output_devices();
        }
    });

    if let Some(device) = requested {
        app.set_output_device(device);
    }

    ui.add_space(4.0);
    match app.playback().output_description() {
        Some(description) => ui.label(
            RichText::new(format!("Playing through {description}."))
                .small()
                .weak(),
        ),
        None => ui.label(
            RichText::new("No output device is open; the library and the queue still work.")
                .small()
                .color(theme::WARNING),
        ),
    };

    if let Some(error) = app.settings_error().map(str::to_owned) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(error).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_settings_error();
            }
        });
    }
}

/// What the device picker says when the system default is chosen.
const DEFAULT_OUTPUT_LABEL: &str = "System default";

/// Width of the device picker, in points.
const DEVICE_PICKER_WIDTH: f32 = 260.0;

/// The keys the player answers to.
///
/// Two columns, one line per command: what it does on the left, the keys on the
/// right, so the list can be read down either way.
fn shortcuts(ui: &mut egui::Ui) {
    ui.label(RichText::new("Keyboard shortcuts").strong());
    ui.label(
        RichText::new("These are ignored while you are typing into a text field.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    egui::Grid::new("musiccraft_shortcuts")
        .num_columns(2)
        .spacing([24.0, 4.0])
        .show(ui, |ui| {
            for command in Command::ALL {
                ui.label(RichText::new(command.description()).small());
                ui.label(RichText::new(command.keys()).small().weak().monospace());
                ui.end_row();
            }
        });

    // Not a shortcut table entry: Tab and the arrow keys are egui moving
    // focus, and Enter is answered by whichever row wears the ring — the
    // Library table, the album grid, the artists, a playlist, the queue —
    // rather than by a command run over the whole application. It belongs in
    // the pane so the gesture is findable, and stated as what it is.
    ui.add_space(8.0);
    ui.label(
        RichText::new(
            "Tab and the arrow keys move the focus ring; Enter acts on the row \
             it rests on, as a double-click would.",
        )
        .small()
        .weak(),
    );
}

/// Where settings live on disk.
fn storage(app: &MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Storage").strong());
    ui.add_space(6.0);

    match app.config_path() {
        Some(path) => {
            ui.label(RichText::new("Settings file").small().weak());
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .monospace(),
            );
        }
        None => {
            ui.label(
                RichText::new("No configuration directory is available on this platform.")
                    .small()
                    .weak(),
            );
        }
    }
}

/// What this build is, and whether the settings on disk could be read.
///
/// The window has no header strip, so the name, the version and any load
/// failure live here.
fn about(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("About").strong());
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::APP_NAME).heading());
        ui.label(RichText::new(version_line()).small().weak());
    });

    if let Some(message) = app.config_error().map(str::to_owned) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(format!("Settings could not be loaded: {message}")).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_config_error();
            }
        });
    }
}

/// The version line the About pane shows.
fn version_line() -> String {
    format!("v{}", crate::APP_VERSION)
}

/// Where the library lives and how music comes in — laid out the way a music
/// application's advanced preferences lay it out: the media folder's location
/// in a bordered box of its own, the buttons that move it stacked at its side,
/// and everything else stated plainly underneath.
fn library_management(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Library management").strong());
    ui.label(
        RichText::new(
            "The media home is where the library's own music lives. Change moves the \
             library's watch to a folder of your choice; Reset returns it to the folder \
             the application was born with. Music comes in from the gear at the corner \
             of the bar — Add files… or Add folder… — or by dragging it onto the window.",
        )
        .small()
        .weak(),
    );
    ui.add_space(6.0);

    // The location box. The path inside it is not editable: a library pointed
    // at a mistyped path is worse than one pointed at the wrong button, so the
    // two buttons at its side are the only ways this box ever changes.
    egui::Frame::new()
        .stroke(ui.visuals().window_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(12, 12))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("{} Media folder location", crate::APP_NAME))
                    .small()
                    .strong(),
            );

            let fixed = HOME_BUTTON_WIDTH + ui.spacing().item_spacing.x;
            let width = (ui.available_width() - fixed).max(140.0);

            ui.horizontal(|ui| {
                let mut path = match app.media_home() {
                    Some(home) => home.display().to_string(),
                    None => "no location — Reset picks one".to_owned(),
                };
                ui.add_sized(
                    [width, 24.0],
                    TextEdit::singleline(&mut path)
                        .interactive(false)
                        .font(egui::FontId::monospace(12.0)),
                );

                ui.vertical(|ui| {
                    if ui
                        .add_sized([HOME_BUTTON_WIDTH, 24.0], egui::Button::new("Change…"))
                        .on_hover_text(
                            "Move the library to a folder of your choice. The library follows \
                             the new folder; the files you have are not moved.",
                        )
                        .clicked()
                    {
                        app.change_media_home();
                    }

                    if ui
                        .add_sized([HOME_BUTTON_WIDTH, 24.0], egui::Button::new("Reset"))
                        .on_hover_text(format!("Return the media home to ~/{}", crate::APP_NAME))
                        .clicked()
                    {
                        app.reset_media_home();
                    }
                });
            });
        });

    ui.add_space(6.0);

    let scanning = app.is_scanning();
    let can_scan = !scanning && !app.library_folders().is_empty();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                can_scan,
                egui::Button::new("Rescan").min_size(Vec2::new(RESCAN_WIDTH, 24.0)),
            )
            .on_hover_text("Scan the watched folders again")
            .clicked()
        {
            app.start_scan();
        }

        if scanning
            && ui
                .add_sized([STOP_WIDTH, 24.0], egui::Button::new("Stop"))
                .on_hover_text("Stop the scan after the file it is reading")
                .clicked()
        {
            app.cancel_scan();
        }
    });

    if let Some(message) = app.library_error().map(str::to_owned) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(message).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_library_error();
            }
        });
    }

    if app.is_scanning() {
        let progress = app.scan_progress();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("Scanning… {} files read", progress.processed))
                    .small()
                    .color(theme::ACCENT),
            );

            if let Some(current) = progress.current.as_ref() {
                ui.label(
                    RichText::new(current.display().to_string())
                        .small()
                        .weak()
                        .monospace(),
                );
            }
        });
    } else if app.is_copying() {
        let progress = app.copy_progress();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "Copying… {} of {} files",
                    progress.copied, progress.total
                ))
                .small()
                .color(theme::ACCENT),
            );

            if let Some(current) = progress.current.as_ref() {
                ui.label(
                    RichText::new(current.display().to_string())
                        .small()
                        .weak()
                        .monospace(),
                );
            }
        });
    } else if let Some(message) = app.last_import().map(str::to_owned) {
        ui.label(RichText::new(message).small().weak());
    }

    ui.add_space(4.0);

    // The list starts after the home: the home is the box above, and what is
    // named here is what the user added alongside it — each with the way to
    // stop watching it.
    let extra: Vec<std::path::PathBuf> = app.library_folders().iter().skip(1).cloned().collect();
    if extra.is_empty() {
        ui.label(
            RichText::new("No folders are watched besides the media home.")
                .small()
                .weak(),
        );
    } else {
        ui.label(RichText::new("Other watched folders").small().weak());

        // Copied out before the buttons are drawn: forgetting a folder edits
        // the list the loop is walking.
        for folder in extra {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(folder.display().to_string())
                        .small()
                        .monospace(),
                );

                if ui
                    .small_button("Forget")
                    .on_hover_text(
                        "Stop scanning this folder. The tracks it contributed stay in the \
                         library and their files stay on disk.",
                    )
                    .clicked()
                {
                    app.remove_library_folder(&folder);
                }
            });
        }
    }
}

/// Where the application keeps everything, and whether the database is sound.
///
/// This is the "something is wrong and I do not know what" pane: what the user
/// reads out when asking for help, and the one check the application can run on
/// its own library.
fn diagnostics(app: &mut MusicCraftApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Diagnostics").strong());
    ui.label(
        RichText::new(
            "What this copy of iCraftMusic is, and where it keeps your library, your settings \
             and its log.",
        )
        .small()
        .weak(),
    );
    ui.add_space(6.0);

    egui::Grid::new("musiccraft_diagnostics")
        .num_columns(2)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            for (label, value) in app.diagnostics().lines() {
                ui.label(RichText::new(label).small().weak());
                ui.label(RichText::new(value).small().monospace());
                ui.end_row();
            }
        });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui
            .button("Check the library database")
            .on_hover_text(
                "Reads every page of the database and reports anything wrong with it. \
                 Nothing is changed.",
            )
            .clicked()
        {
            app.check_library_integrity();
        }

        if let Some(folder) = app.diagnostics().data_directory.as_ref() {
            ui.label(
                RichText::new(format!("Data: {}", folder.display()))
                    .small()
                    .weak()
                    .monospace(),
            );
        }
    });

    if let Some(message) = app.library_integrity().map(str::to_owned) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("!").strong().color(theme::WARNING));
            ui.label(RichText::new(message).small());
            if ui.small_button("Dismiss").clicked() {
                app.dismiss_integrity();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::Startup;
    use crate::audio::{AudioEngine, Playback};
    use crate::config::AppConfig;
    use crate::error::AppError;

    #[test]
    fn the_location_box_and_scan_buttons_fit_the_pane_they_are_drawn_in() {
        let mut app = app(None);
        app.open_settings();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        let texts = painted(&mut app, screen);

        for label in ["Change…", "Reset", "Rescan"] {
            let ink = texts
                .iter()
                .find(|(text, _)| text == label)
                .unwrap_or_else(|| panic!("{label} is not drawn: {texts:?}"))
                .1;

            // The margin is the button's own padding: the label is centred in
            // its box, so a label that ends within eight points of the edge
            // means the box around it does not.
            assert!(
                ink.right() <= screen.right() - 8.0,
                "{label} runs off the pane: {ink:?}"
            );
        }
    }

    /// Every line of text the pane paints, with the rectangle its ink covers.
    fn painted(app: &mut MusicCraftApp, screen: egui::Rect) -> Vec<(String, egui::Rect)> {
        let ctx = egui::Context::default();
        let mut output = None;

        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ui| crate::ui::render(app, ui),
            );
            frame.textures_delta.clear();
            output = Some(frame);
        }

        let output = output.expect("`run_ui` ran at least once");
        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }

        texts
    }

    /// Collect a text shape's line and the ink it covers.
    fn collect(shape: &egui::Shape, texts: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Text(text) => {
                let size = text.galley.size();
                let min = match text.galley.job.halign {
                    egui::Align::LEFT => text.pos,
                    egui::Align::Center => egui::pos2(text.pos.x - size.x / 2.0, text.pos.y),
                    egui::Align::RIGHT => egui::pos2(text.pos.x - size.x, text.pos.y),
                };
                texts.push((
                    text.galley.text().to_owned(),
                    egui::Rect::from_min_size(min, size),
                ));
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, texts);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn the_library_management_pane_offers_a_folder_chooser() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| show_tab(&mut app, ui, SettingsTab::Library));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text.contains("Change…")),
            "the location box does not offer its chooser: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Media folder location")),
            "the location box does not name itself: {painted:?}"
        );
        // Drawing it must not open the dialog: a native chooser blocks the
        // thread it is called from, and a test that called it would hang.
        assert!(
            app.library_folders().is_empty(),
            "drawing the button waits for a click"
        );
        assert!(app.library_error().is_none());
    }

    #[test]
    fn the_shortcuts_list_names_every_command_and_its_keys() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| show_tab(&mut app, ui, SettingsTab::Shortcuts));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text == "Keyboard shortcuts"),
            "the list is not there: {painted:?}"
        );

        for command in Command::ALL {
            assert!(
                painted.iter().any(|text| text == command.keys()),
                "{} is missing from the list",
                command.keys()
            );
            assert!(
                painted.iter().any(|text| text == command.description()),
                "{command:?} has no line in the list"
            );
        }
    }

    #[test]
    fn the_settings_pane_draws_with_its_about_section() {
        let mut app = app(None);

        egui::__run_test_ui(|ui| show_tab(&mut app, ui, SettingsTab::About));

        assert!(app.config_error().is_none());
    }

    #[test]
    fn the_about_pane_names_the_build() {
        assert_eq!(version_line(), format!("v{}", crate::APP_VERSION));
        assert!(!crate::APP_NAME.is_empty());
    }

    #[test]
    fn a_settings_load_failure_is_still_reported_from_the_about_pane() {
        let mut app = app(Some(AppError::NoConfigDir));
        let expected = AppError::NoConfigDir.to_string();

        egui::__run_test_ui(|ui| show_tab(&mut app, ui, SettingsTab::About));

        // Drawing it does not dismiss it: the message outlives the frame until
        // the user acts on it.
        assert_eq!(
            app.config_error(),
            Some(expected.as_str()),
            "the failure the header strip used to show must survive the move"
        );
    }

    #[test]
    fn the_about_pane_paints_the_name_and_version() {
        let mut app = app(None);
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| show_tab(&mut app, ui, SettingsTab::About));
        });
        output.textures_delta.clear();

        let painted: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();

        assert!(
            painted.iter().any(|text| text == crate::APP_NAME),
            "the settings pane should name the application"
        );
        assert!(
            painted.iter().any(|text| text == &version_line()),
            "the settings pane should show the version"
        );
        assert!(
            painted.iter().any(|text| text == "About"),
            "the pane should be labelled"
        );
    }

    #[test]
    fn the_diagnostics_pane_reports_where_everything_lives() {
        let mut app = app(None);
        let painted = everything(&mut app, SettingsTab::Diagnostics);

        for label in [
            "Diagnostics",
            "Application",
            "Check the library database",
            "Audio output",
            "Indexed",
        ] {
            assert!(
                painted.iter().any(|text| text == label),
                "{label} is not on the pane: {painted:?}"
            );
        }

        assert!(
            painted
                .iter()
                .any(|text| text == &format!("{} {}", crate::APP_NAME, crate::APP_VERSION)),
            "the snapshot should name the build: {painted:?}"
        );
    }

    #[test]
    fn drawing_the_diagnostics_pane_does_not_run_the_check() {
        // The check reads the whole database, so it answers a press, never a
        // repaint — and a pane that ran it every frame would make Settings the
        // slowest screen in the application.
        let mut app = app(None);

        let _ = everything(&mut app, SettingsTab::Diagnostics);

        assert!(
            app.library_integrity().is_none(),
            "no check was asked for: {:?}",
            app.library_integrity()
        );
    }

    #[test]
    fn a_check_the_user_asked_for_is_reported_on_the_pane() {
        let mut app = app(None);
        app.check_library_integrity();

        let painted = everything(&mut app, SettingsTab::Diagnostics);

        let message = app.library_integrity().expect("the check said something");
        assert!(
            painted.iter().any(|text| text.contains(message)),
            "the answer is not on the pane: {painted:?}"
        );
    }

    #[test]
    fn a_watched_folder_can_be_forgotten_from_the_pane() {
        // Forgetting a folder is about the watched list, so the application
        // needs a library for the folder to be watched by. The first folder is
        // the media home — it lives in its own box — so the forgettable one is
        // the folder added alongside it.
        let mut app = watched();
        let home = std::env::temp_dir().join("musiccraft-settings-home");
        let folder = std::env::temp_dir().join("musiccraft-settings-forget");
        std::fs::create_dir_all(&home).expect("a folder to be the home");
        std::fs::create_dir_all(&folder).expect("a folder to watch");
        assert!(app.add_library_folder(&home));
        assert!(app.add_library_folder(&folder));

        let painted = everything(&mut app, SettingsTab::Library);
        assert!(
            painted.iter().any(|text| text == "Forget"),
            "the folder has no way to be dropped: {painted:?}"
        );
        assert!(
            painted
                .iter()
                .any(|text| text == &folder.display().to_string()),
            "the extra folder is not listed: {painted:?}"
        );

        // Drawing the button is not pressing it.
        assert_eq!(
            app.library_folders().len(),
            2,
            "the folder is forgotten on a click, not on a repaint"
        );
    }

    /// The whole dialog as a few steady frames painted it, shapes and all.
    fn sheet(app: &mut MusicCraftApp) -> Vec<egui::epaint::ClippedShape> {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));
        let ctx = egui::Context::default();
        let mut shapes = Vec::new();

        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ui| crate::ui::render(app, ui),
            );
            frame.textures_delta.clear();
            shapes = std::mem::take(&mut frame.shapes);
        }

        shapes
    }

    /// The rectangle the widest painted stroke of `colour` rules. One shape
    /// per measured colour is the dialog's own arrangement: the sheet's edge,
    /// the pane's, the ring's, the Cancel's — none shares its colour with
    /// another, and finding the widest guards against a coincidence elsewhere
    /// on the screen rather than papering over one.
    fn stroked(shapes: &[egui::epaint::ClippedShape], colour: egui::Color32) -> egui::Rect {
        shapes
            .iter()
            .filter_map(|clipped| stroked_in(&clipped.shape, colour))
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .unwrap_or_else(|| panic!("nothing on the sheet wears a {colour:?} rule"))
    }

    /// One rule's rectangle, searched through the shape and any `Vec` it is
    /// packed into: the painter runs what shares a clip into one `Vec`, so a
    /// sheet's rules rarely sit at the top on their own.
    fn stroked_in(shape: &egui::Shape, colour: egui::Color32) -> Option<egui::Rect> {
        match shape {
            egui::Shape::Rect(painted) if painted.stroke.color == colour => Some(painted.rect),
            egui::Shape::Vec(shapes) => shapes
                .iter()
                .filter_map(|one| stroked_in(one, colour))
                .max_by(|a, b| a.area().total_cmp(&b.area())),
            _ => None,
        }
    }

    /// Press and release, where a mouse would. The frames go through a real
    /// context, because a click that never ran through `run` clicked nothing.
    fn click(app: &mut MusicCraftApp, ctx: &egui::Context, pos: egui::Pos2) {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));

        for events in [
            vec![egui::Event::PointerMoved(pos)],
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
        ] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::render(app, ui),
            );
            output.textures_delta.clear();
        }
    }

    /// One throwaway frame. A window the context has only just heard about
    /// spends its first frame sizing itself and paints nothing — egui holds
    /// that frame back from the painter — so a test that aims at the strip,
    /// or reads the panes, passes one frame first. In the real application
    /// that frame is one sixtieth of a second, and the dialog is up before
    /// the eye notices it.
    fn warm(app: &mut MusicCraftApp, ctx: &egui::Context) {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| crate::ui::render(app, ui),
        );
        output.textures_delta.clear();
    }

    /// The sheet, the pane, the tabs' feet and the footer's floor, all where
    /// the capture measured them: 573×648; the pane fifteen in from the left,
    /// fourteen from the right, its head at the tabs' feet and its foot forty-
    /// two below the sheet's.
    #[test]
    fn the_sheet_and_its_pane_stand_where_the_dialog_measured_them() {
        let mut app = app(None);
        app.open_settings();
        let shapes = sheet(&mut app);

        let face = stroked(&shapes, theme::SETTINGS_EDGE);
        let pane = stroked(&shapes, theme::SETTINGS_LINE);

        assert!(
            (face.width() - WINDOW_WIDTH).abs() < 1.0
                && (face.height() - WINDOW_FALLBACK_HEIGHT).abs() < 1.0,
            "the sheet is {:?}, not the measured 573 by 648",
            face.size()
        );
        assert!(
            (pane.left() - face.left() - PANE_LEFT).abs() < 1.0
                && (face.right() - pane.right() - PANE_RIGHT).abs() < 1.0,
            "the pane is not the measured fifteen-and-fourteen: {pane:?} in {face:?}"
        );
        assert!(
            (pane.top() - face.top() - (TAB_TOP + TAB_ON_HEIGHT)).abs() < 1.0,
            "the tabs' feet do not make the pane's head: pane at {}, sheet at {}",
            pane.top(),
            face.top()
        );
        assert!(
            (face.bottom() - pane.bottom() - FOOTER_HEIGHT).abs() < 1.0,
            "the footer is not the measured forty-two points: {pane:?} in {face:?}"
        );
    }

    /// The footer's buttons, pinned through the paint: the measured seventy-
    /// five and seventy-six wide, twenty-one tall, standing on the same line
    /// ten points below the footer's head, ninety-seven and fourteen in from
    /// the sheet's right edge.
    #[test]
    fn the_footer_buttons_stand_at_their_measured_places() {
        let mut app = app(None);
        app.open_settings();
        let shapes = sheet(&mut app);

        let face = stroked(&shapes, theme::SETTINGS_EDGE);
        let ok = stroked(&shapes, theme::SETTINGS_DEFAULT_RING);
        let cancel = stroked(&shapes, theme::SETTINGS_BUTTON_EDGE);

        assert!(
            (ok.width() - OK_WIDTH).abs() < 1.0 && (ok.height() - BUTTON_HEIGHT).abs() < 1.0,
            "OK is {ok:?}, not the measured {OK_WIDTH} by {BUTTON_HEIGHT}"
        );
        assert!(
            (cancel.width() - CANCEL_WIDTH).abs() < 1.0
                && (cancel.height() - BUTTON_HEIGHT).abs() < 1.0,
            "Cancel is {cancel:?}, not the measured {CANCEL_WIDTH} by {BUTTON_HEIGHT}"
        );
        assert!(
            (face.right() - ok.right() - OK_RIGHT).abs() < 1.0
                && (face.right() - cancel.right() - CANCEL_RIGHT).abs() < 1.0,
            "the pair has drifted from the measured ninety-seven and fourteen: \
             {ok:?} and {cancel:?} in {face:?}"
        );
        assert!(
            (ok.top() - cancel.top()).abs() < 0.1
                && (ok.top() - (face.bottom() - FOOTER_HEIGHT + BUTTON_TOP)).abs() < 1.0,
            "the buttons are not level on the footer's measured line: {ok:?} vs {cancel:?}"
        );
    }

    /// The strip names every pane, and the sheet holds exactly one of them:
    /// the tabs and the footer are always on, but a pane's own words — the
    /// ones no tab wears — show for the open pane and for no other.
    #[test]
    fn the_strip_names_every_pane_and_the_sheet_holds_one() {
        let mut app = app(None);
        app.open_settings();

        let mut painted = Vec::new();
        for clipped in sheet(&mut app) {
            collect(&clipped.shape, &mut painted);
        }
        let painted: Vec<String> = painted.into_iter().map(|(text, _)| text).collect();

        assert!(
            painted.iter().any(|text| text == "Settings"),
            "no title: {painted:?}"
        );
        for tab in SettingsTab::ALL {
            let label = tab_label(tab).to_owned();
            assert!(
                painted.contains(&label),
                "the strip has no {label:?} tab: {painted:?}"
            );
        }
        for button in ["OK", "Cancel"] {
            assert!(
                painted.contains(&button.to_owned()),
                "no {button}: {painted:?}"
            );
        }

        // Library is the pane the window opens on — and only its words show.
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Media folder location")),
            "the Library pane is not the one open: {painted:?}"
        );
        for foreign in [
            "Keyboard shortcuts",
            "Check the library database",
            "Settings file",
        ] {
            assert!(
                !painted.iter().any(|text| text == foreign),
                "the {foreign} pane shows through the open one: {painted:?}"
            );
        }
    }

    /// A tab under the pointer opens its pane: a real press finds the tab's
    /// own words, and the sheet that comes back holds the pane they name and
    /// no longer the one it held.
    #[test]
    fn a_tab_under_the_pointer_opens_its_pane() {
        let mut app = app(None);
        app.open_settings();
        let ctx = egui::Context::default();
        warm(&mut app, &ctx);

        // Where does the Shortcuts tab's word sit?
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));
        let mut first = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| crate::ui::render(&mut app, ui),
        );
        let mut texts = Vec::new();
        for clipped in &first.shapes {
            collect(&clipped.shape, &mut texts);
        }
        first.textures_delta.clear();
        let tab = texts
            .iter()
            .find(|(text, _)| text == "Shortcuts")
            .map(|(_, rect)| *rect)
            .expect("the strip has a Shortcuts tab");

        click(&mut app, &ctx, tab.center());

        assert_eq!(app.settings_tab(), SettingsTab::Shortcuts);

        let painted: Vec<String> = sheet(&mut app)
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect();
        assert!(
            painted.iter().any(|text| text == "Keyboard shortcuts"),
            "the click did not open the keys: {painted:?}"
        );
        assert!(
            !painted
                .iter()
                .any(|text| text.contains("Media folder location")),
            "the Library pane refused to leave: {painted:?}"
        );
    }

    /// The mark on the dark strip is the window's way out: a press shuts it,
    /// the same as the ✕ on the measured dialog's title bar.
    /// The mark on the dark strip is the window's way out: a press shuts it,
    /// the same as the close mark on the measured dialog's title bar. The
    /// mark is strokes, not type, so the test aims where the bar's own
    /// measured geometry puts it — ten in from the sheet's right, centred
    /// on the bar's twenty-eight points.
    #[test]
    fn the_close_mark_shuts_the_window() {
        let mut app = app(None);
        app.open_settings();
        let ctx = egui::Context::default();
        warm(&mut app, &ctx);

        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 808.0));
        let mut first = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| crate::ui::render(&mut app, ui),
        );
        let face = stroked(&first.shapes, theme::SETTINGS_EDGE);
        first.textures_delta.clear();

        click(
            &mut app,
            &ctx,
            egui::pos2(
                face.right() - CLOSE_RIGHT - CLOSE_SIZE / 2.0,
                face.top() + BAR_HEIGHT / 2.0,
            ),
        );

        assert!(!app.settings_open(), "the mark did not shut the window");
    }

    /// Cancel says what it does: the choices the window owns come back to
    /// where they stood when it opened, and the window is shut.
    #[test]
    fn cancel_puts_back_the_choices_the_window_owns() {
        let mut app = app(None);
        let volume = app.volume();
        let crossfade = app.crossfade_seconds();

        app.open_settings();
        app.set_volume(0.25);
        app.set_crossfade_seconds(3.0);

        app.cancel_settings();

        assert!(!app.settings_open(), "Cancel shut the window");
        assert_eq!(app.volume(), volume, "Cancel did not undo the volume");
        assert_eq!(
            app.crossfade_seconds(),
            crossfade,
            "Cancel did not undo the crossfade"
        );
    }

    /// OK is honest the other way: what the window applied live stays applied,
    /// because OK only closes and never claimed to commit anything.
    #[test]
    fn closing_by_ok_keeps_what_the_window_applied() {
        let mut app = app(None);
        app.open_settings();
        app.set_volume(0.25);

        app.close_settings();

        assert!(!app.settings_open());
        assert_eq!(app.volume(), 0.25, "OK undid what it never promised to");
    }

    /// And the promise is only ever about this trip through the window: a
    /// change OKed is a change kept, and the next Cancel puts back to it, not
    /// past it to some earlier visit.
    #[test]
    fn a_reopened_window_cancels_only_its_own_changes() {
        let mut app = app(None);

        app.open_settings();
        app.set_volume(0.25);
        app.close_settings(); // OK.

        app.open_settings();
        app.set_crossfade_seconds(4.0);
        app.cancel_settings();

        assert_eq!(
            app.volume(),
            0.25,
            "the second Cancel undid the first trip's kept change"
        );
        assert_eq!(
            app.crossfade_seconds(),
            0.0,
            "its own change was not undone"
        );
    }

    /// The Help menu's questions open on the pane that answers them, not on
    /// wherever the gear last left the strip.
    #[test]
    fn the_help_panes_open_where_their_questions_were_asked() {
        let mut app = app(None);
        app.set_settings_tab(SettingsTab::Storage);

        app.open_settings_at(SettingsTab::About);

        assert!(app.settings_open());
        assert_eq!(
            app.settings_tab(),
            SettingsTab::About,
            "the window opened on someone else's question"
        );
    }

    /// Every line of text one Settings pane paints.
    ///
    /// The sheet is fixed at the reference dialog's 573×648, and the part of a
    /// pane below its fold never reaches the painter, so the pane is drawn here
    /// at the sheet's own width but tall enough to hold it whole — a short
    /// screen would otherwise only test the lines that come first. The same
    /// scroll area the window wraps the pane in wraps it here.
    fn everything(app: &mut MusicCraftApp, tab: SettingsTab) -> Vec<String> {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_WIDTH, 2400.0));
        let ctx = egui::Context::default();
        let mut texts = Vec::new();

        for _ in 0..3 {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| show_tab(app, ui, tab));
                },
            );
            frame.textures_delta.clear();
            texts.clear();
            for clipped in &frame.shapes {
                collect(&clipped.shape, &mut texts);
            }
        }

        texts.into_iter().map(|(text, _)| text).collect()
    }

    /// An application with a player that has no output device.
    fn app(error: Option<AppError>) -> MusicCraftApp {
        MusicCraftApp::with_playback(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
        )
    }

    /// An application that has a library, so folders can be watched by one.
    ///
    /// The pane's own `app()` deliberately has none; anything that adds or
    /// forgets a watched folder needs the library version, or it would be
    /// asserting against a failure ("there is no library to import into")
    /// rather than against the folder list.
    fn watched() -> MusicCraftApp {
        let library = crate::library::Library::in_memory(
            std::env::temp_dir().join("musiccraft-settings-artwork"),
        )
        .expect("an in-memory library");

        MusicCraftApp::with_library(
            Startup {
                config: AppConfig::default(),
                config_path: None,
                error: None,
            },
            Playback::with_engine(AudioEngine::without_device(), 1.0),
            library,
        )
    }
}

#[cfg(test)]
mod reference_settings_tests {
    #[test]
    fn preferences_window_matches_reference() {
        assert_eq!(super::WINDOW_WIDTH, 573.0);
        assert_eq!(super::WINDOW_FALLBACK_HEIGHT, 648.0);
    }
}
