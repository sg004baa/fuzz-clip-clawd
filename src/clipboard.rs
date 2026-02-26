use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;

use crate::history::History;
use crate::storage;

/// Start clipboard monitoring in a background thread.
/// Polls the clipboard at the given interval and pushes new text to history.
/// Calls `request_repaint` on the egui context when history changes.
pub fn start_monitor(
    history: Arc<Mutex<History>>,
    poll_interval: Duration,
    ctx: eframe::egui::Context,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to initialize clipboard: {e}");
                return;
            }
        };

        let mut last_text = clipboard.get_text().unwrap_or_default();

        loop {
            thread::sleep(poll_interval);

            let current_text = match clipboard.get_text() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if current_text != last_text && !current_text.is_empty() {
                last_text = current_text.clone();

                let mut hist = history.lock().unwrap();
                if hist.push(current_text) {
                    // Save on every change
                    if let Err(e) = storage::save(&hist) {
                        eprintln!("Failed to save history: {e}");
                    }
                    ctx.request_repaint();
                }
            }
        }
    })
}
