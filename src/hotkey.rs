use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rdev::{listen, Event, EventType, Key};

/// Start the global hotkey listener in a background thread.
/// Detects Ctrl+Ctrl double-tap (two Ctrl presses within 300ms).
pub fn start_listener(visible: Arc<Mutex<bool>>, ctx: eframe::egui::Context) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let last_ctrl_press: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let last_ctrl = Arc::clone(&last_ctrl_press);
        let vis = Arc::clone(&visible);

        let callback = move |event: Event| {
            if let EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) =
                event.event_type
            {
                let mut last = last_ctrl.lock().unwrap();
                let now = Instant::now();

                if let Some(prev) = *last {
                    let elapsed = now.duration_since(prev);
                    if elapsed.as_millis() < 300 {
                        // Double-tap detected — toggle visibility
                        let mut v = vis.lock().unwrap();
                        *v = !*v;
                        let is_now_visible = *v;
                        drop(v);

                        // On Windows, WM_PAINT is not delivered to hidden windows,
                        // so request_repaint() alone cannot wake the egui event loop.
                        // Call ShowWindow directly so the loop resumes and processes
                        // the updated visibility flag.
                        if is_now_visible {
                            crate::platform::show_window_native();
                        }

                        ctx.request_repaint();
                        *last = None; // Reset to avoid triple-tap
                        return;
                    }
                }

                *last = Some(now);
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("Failed to start hotkey listener: {:?}", e);
        }
    })
}
