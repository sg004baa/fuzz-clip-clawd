mod app;
mod clipboard;
mod config;
mod fuzzy;
mod history;
mod hotkey;
mod storage;
mod tray;

use std::sync::{Arc, Mutex};

use eframe::egui;

fn main() -> eframe::Result<()> {
    let config = config::Config::default();

    // Load history from disk
    let history = storage::load(config.max_size);
    let history = Arc::new(Mutex::new(history));

    // Shared visibility flag (start visible so user sees the window on first launch)
    let visible = Arc::new(Mutex::new(true));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_decorations(false)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Clipboard History",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(app::ClipboardHistoryApp::new(
                Arc::clone(&history),
                Arc::clone(&visible),
                config.clone(),
            )))
        }),
    )
}
