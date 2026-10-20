//! iCraftMusic: a native desktop music player.
//!
//! The binary is intentionally thin. It initialises logging, reads the settings
//! and hands control to `eframe`; everything else lives in the library crate so
//! it can be tested without opening a window.

use std::process::ExitCode;

use musiccraft::{APP_NAME, app};

/// Native window width at first launch, in points.
const INITIAL_WIDTH: f32 = 1180.0;
/// Native window height at first launch, in points.
const INITIAL_HEIGHT: f32 = 780.0;
/// Smallest window the layout is designed to survive.
const MIN_WIDTH: f32 = 760.0;
/// Smallest window height the layout is designed to survive.
const MIN_HEIGHT: f32 = 520.0;

fn main() -> ExitCode {
    musiccraft::logging::init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // The subscriber is already installed, so this reaches the user's
            // terminal (or journal) with the full cause chain.
            tracing::error!("{error:#}");
            eprintln!("{APP_NAME} could not start: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Whether the command line asked for the X11 window.
fn use_x11(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--x11")
}

/// What `--help` says, which is also the honest answer to "why is there a
/// flag for a window at all": the flag is about what the desktop delivers,
/// not about how the window looks.
fn usage() -> String {
    format!(
        "{APP_NAME} {}\n\n\
         Drag music onto the window from an X11 session, or from a Wayland one\n\
         with --x11, which draws the window through XWayland so the desktop\n\
         can deliver the drop. The gear menu's Add entries work everywhere.\n",
        musiccraft::APP_VERSION
    )
}

/// Open the window and run until the user closes it.
fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{}", usage());
        return Ok(());
    }

    if use_x11(&args) {
        // Winit picks its backend from these two variables, and X11 is what
        // gives `--x11` its whole point: the desktop delivers file drops to
        // X11 windows, and on Wayland it delivers them to nobody.
        //
        // SAFETY: this runs before any thread exists — the first act of the
        // first and only thread — so nothing can be reading the environment
        // while it changes.
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::remove_var("WAYLAND_SOCKET");
        }
        tracing::info!("--x11: the window runs through XWayland, where file drops arrive");
    } else if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        // The one caveat on this kind of desktop, said once and quietly: the
        // windowing toolkit cannot receive drops on native Wayland, so the
        // pointer's way in is closed while the menu's two ways work.
        tracing::info!(
            "running on native Wayland: the desktop delivers no file drops here; \
             pass --x11 to run through XWayland if you want to drag music in"
        );
    }

    let startup = app::Startup::load();

    if let Some(error) = startup.error.as_ref() {
        tracing::warn!("settings were not loaded: {error}");
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_app_id("musiccraft")
            .with_inner_size([INITIAL_WIDTH, INITIAL_HEIGHT])
            .with_min_inner_size([MIN_WIDTH, MIN_HEIGHT])
            // Dropping files on the window is a way to add music, so the
            // window has to accept them: asking for this is what makes the
            // desktop tell egui about a drop at all.
            .with_drag_and_drop(true),
        ..Default::default()
    };

    tracing::info!("starting {APP_NAME} {}", musiccraft::APP_VERSION);

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |creation_context| {
            let mut app = app::MusicCraftApp::new(creation_context, startup);
            // Look for a newer build on its own thread; the window opens whether
            // or not the answer ever comes.
            app.check_for_release();
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flag is read as a word of its own: a stray argument that merely
    /// looks like it — a longer flag, a negation, a path — must not silently
    /// choose the window's backend.
    #[test]
    fn the_x11_flag_is_read_as_a_word_of_its_own() {
        assert!(use_x11(&["musiccraft".into(), "--x11".into()]));
        assert!(!use_x11(&["musiccraft".into()]));
        assert!(!use_x11(&["musiccraft".into(), "--x111".into()]));
        assert!(!use_x11(&["musiccraft".into(), "--no-x11".into()]));
    }

    /// The `--help` text is where someone who has never opened the README
    /// learns that a flag for the window exists at all, and what it is for:
    /// the drop, not the look.
    #[test]
    fn the_usage_names_the_flag_and_says_what_it_buys() {
        let text = usage();
        assert!(
            text.contains("--x11"),
            "the flag is not in the usage: {text}"
        );
        assert!(
            text.contains("drop") && text.contains("gear"),
            "the usage does not say what the flag buys: {text}"
        );
    }
}
