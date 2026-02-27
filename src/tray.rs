use std::sync::{Arc, Mutex};

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Create a simple 16x16 blue icon for the system tray.
fn create_default_icon() -> Icon {
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..size * size {
        // Blue icon with full opacity
        rgba.push(60);  // R
        rgba.push(120); // G
        rgba.push(216); // B
        rgba.push(255); // A
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

/// Build and return the system tray icon with a simple menu.
pub fn build_tray(visible: Arc<Mutex<bool>>, ctx: eframe::egui::Context) -> TrayIcon {
    let menu = Menu::new();
    let show_item = MenuItem::new("Show/Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    menu.append(&show_item).unwrap();
    menu.append(&quit_item).unwrap();

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Clipboard History")
        .with_icon(create_default_icon())
        .build()
        .expect("Failed to build tray icon");

    // Handle menu events in a background thread
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                if event.id() == &show_id {
                    let mut v = visible.lock().unwrap();
                    *v = !*v;
                    let is_now_visible = *v;
                    drop(v);

                    // Same Windows fix as in hotkey.rs: directly show the window
                    // so the egui event loop wakes up when the window was hidden.
                    if is_now_visible {
                        crate::platform::show_window_native();
                    } else {
                        // Hide immediately so the OS removes the window before
                        // egui's next frame can flash a black clear-color.
                        crate::platform::hide_window_native();
                    }

                    ctx.request_repaint();
                } else if event.id() == &quit_id {
                    std::process::exit(0);
                }
            }
        }
    });

    tray
}
