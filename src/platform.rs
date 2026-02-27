/// On Windows, `ViewportCommand::Visible(true)` combined with
/// `ctx.request_repaint()` is not sufficient to un-hide a window that was
/// hidden via `ViewportCommand::Visible(false)`.  Win32 does not deliver
/// `WM_PAINT` to hidden windows, so the egui event loop never wakes up and
/// `update()` is never called — the visibility flag change is never acted on.
///
/// This function bypasses egui entirely: it calls `ShowWindow`/
/// `SetForegroundWindow` directly so that Windows delivers a `WM_PAINT`
/// message, waking the event loop and allowing `update()` to run normally.
///
/// On non-Windows platforms the egui repaint mechanism is sufficient, so this
/// is a no-op.
pub fn show_window_native() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, SetForegroundWindow, ShowWindow, SW_SHOW,
        };

        // Locate the main window by its title (set in eframe::run_native).
        let title: Vec<u16> = "Clipboard History\0".encode_utf16().collect();
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if hwnd != std::ptr::null_mut() {
            unsafe {
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            }
        }
    }
}

/// Hide the window immediately via Win32 `ShowWindow(SW_HIDE)`.
///
/// Called from the hotkey/tray threads before `request_repaint()` so the
/// window disappears instantly, preventing egui's black clear-color from
/// flashing on screen during the hide transition.
///
/// No-op on non-Windows platforms.
pub fn hide_window_native() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, ShowWindow, SW_HIDE,
        };

        let title: Vec<u16> = "Clipboard History\0".encode_utf16().collect();
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if hwnd != std::ptr::null_mut() {
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}
