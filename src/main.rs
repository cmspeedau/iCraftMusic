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

/// Open the window and run until the user closes it.
fn run() -> anyhow::Result<()> {
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
