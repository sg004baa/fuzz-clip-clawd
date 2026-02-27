use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rdev::{listen, Event, EventType, Key};

/// Start the global hotkey listener in a background thread.
/// Detects Ctrl+Ctrl double-tap (two Ctrl presses within 300ms).
pub fn start_listener(visible: Arc<Mutex<bool>>, ctx: eframe::egui::Context) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // last_ctrl_press: timestamp of the previous genuine Ctrl tap.
        let last_ctrl_press: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        // ctrl_is_down: true while any Ctrl key is physically held.
        // Used to ignore OS key-repeat events (KeyPress fires repeatedly while
        // held, which would otherwise trigger a false double-tap after ~530 ms).
        let ctrl_is_down: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let last_ctrl = Arc::clone(&last_ctrl_press);
        let is_down = Arc::clone(&ctrl_is_down);
        let vis = Arc::clone(&visible);

        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(Key::ControlLeft)
                | EventType::KeyPress(Key::ControlRight) => {
                    // Ignore key-repeat events produced by holding the key.
                    let mut down = is_down.lock().unwrap();
                    if *down {
                        return;
                    }
                    *down = true;
                    drop(down);

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

                            if is_now_visible {
                                crate::platform::show_window_native();
                            } else {
                                crate::platform::hide_window_native();
                            }

                            ctx.request_repaint();
                            *last = None; // Reset to avoid triple-tap
                            return;
                        }
                    }

                    *last = Some(now);
                }
                EventType::KeyRelease(Key::ControlLeft)
                | EventType::KeyRelease(Key::ControlRight) => {
                    *is_down.lock().unwrap() = false;
                }
                _ => {}
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("Failed to start hotkey listener: {:?}", e);
        }
    })
}
