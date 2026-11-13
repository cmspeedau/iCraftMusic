mod app;
mod domain;
mod media;
mod storage;

use app::LibraryApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("iCraftMusic")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([560.0, 420.0]),
        ..Default::default()
    };

    eframe::run_native(
        "iCraftMusic",
        options,
        Box::new(|_creation_context| Ok(Box::new(LibraryApp::load()))),
    )
}
