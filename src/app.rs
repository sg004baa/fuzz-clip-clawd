use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::clipboard;
use crate::config::Config;
use crate::fuzzy;
use crate::history::History;
use crate::hotkey;
use crate::tray;

pub struct ClipboardHistoryApp {
    history: Arc<Mutex<History>>,
    search_query: String,
    selected_index: usize,
    visible: Arc<Mutex<bool>>,
    config: Config,
    initialized: bool,
    was_visible: bool,
    _tray: Option<tray_icon::TrayIcon>,
    cursor_pos: Arc<Mutex<(f64, f64)>>,
}

impl ClipboardHistoryApp {
    pub fn new(
        history: Arc<Mutex<History>>,
        visible: Arc<Mutex<bool>>,
        config: Config,
    ) -> Self {
        Self {
            history,
            search_query: String::new(),
            selected_index: 0,
            visible,
            config,
            initialized: false,
            was_visible: false,
            _tray: None,
            cursor_pos: Arc::new(Mutex::new((0.0, 0.0))),
        }
    }
}

impl eframe::App for ClipboardHistoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start background threads and tray on first frame (now we have the real Context)
        if !self.initialized {
            self.initialized = true;

            // Start clipboard monitor
            clipboard::start_monitor(
                Arc::clone(&self.history),
                std::time::Duration::from_millis(self.config.poll_interval_ms),
                ctx.clone(),
            );

            // Start hotkey listener (also tracks global mouse cursor position)
            hotkey::start_listener(Arc::clone(&self.visible), ctx.clone(), Arc::clone(&self.cursor_pos));

            // Build system tray with the real egui Context
            self._tray = Some(tray::build_tray(Arc::clone(&self.visible), ctx.clone()));
        }

        // Poll periodically to check visibility flag changes from hotkey/tray threads
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Check visibility
        let is_visible = *self.visible.lock().unwrap();

        if is_visible && !self.was_visible {
            // Just became visible — show window, move to cursor, reset state
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

            // Move window near mouse cursor using globally tracked position.
            // If the window would extend below the screen, show it above the cursor instead.
            let (cx, cy) = *self.cursor_pos.lock().unwrap();
            let cx = cx as f32;
            let cy = cy as f32;
            let win_h = self.config.window_height;
            let screen_h = ctx
                .input(|i| i.viewport().monitor_size)
                .map(|s| s.y)
                .unwrap_or(1080.0);
            let y = if cy - 50.0 + win_h > screen_h {
                // Not enough space below — show window above the cursor
                (cy - win_h).max(0.0)
            } else {
                cy - 50.0
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                egui::pos2(cx - 200.0, y),
            ));

            self.search_query.clear();
            self.selected_index = 0;
        } else if !is_visible && self.was_visible {
            // Just became hidden — hide natively first to avoid a black flash
            // before egui presents the final frame.
            crate::platform::hide_window_native();
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        self.was_visible = is_visible;

        if !is_visible {
            // Window is hidden — don't render UI but keep the loop alive
            return;
        }

        // Handle Escape key to hide
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            *self.visible.lock().unwrap() = false;
            crate::platform::hide_window_native();
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.search_query.clear();
            self.selected_index = 0;
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Search bar
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search clipboard history...")
                    .desired_width(f32::INFINITY),
            );

            // Auto-focus the search bar
            if !search_response.has_focus() {
                search_response.request_focus();
            }

            ui.add_space(4.0);
            ui.separator();

            // Get filtered entries
            let history = self.history.lock().unwrap();
            let entries = history.entries();
            let results = fuzzy::search(&self.search_query, entries);

            // Handle keyboard navigation
            let up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
            let down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
            let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));

            if up && self.selected_index > 0 {
                self.selected_index -= 1;
            }
            if down && self.selected_index + 1 < results.len() {
                self.selected_index += 1;
            }

            // Clamp selected index
            if !results.is_empty() && self.selected_index >= results.len() {
                self.selected_index = results.len() - 1;
            }

            // Handle Enter key selection
            let mut selected_content: Option<String> = None;
            if enter && !results.is_empty() {
                selected_content = Some(results[self.selected_index].0.content.clone());
            }

            // Scrollable entry list
            if results.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label("No clipboard history yet. Copy some text!");
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, (entry, _score)) in results.iter().enumerate() {
                        let is_selected = i == self.selected_index;

                        // Truncate content for display (single line preview)
                        let preview: String = entry
                            .content
                            .chars()
                            .take(80)
                            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
                            .collect();

                        let label = egui::SelectableLabel::new(is_selected, &preview);
                        let response = ui.add(label);

                        if response.clicked() {
                            selected_content = Some(entry.content.clone());
                        }

                        // Auto-scroll to selected item
                        if is_selected {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                });
            }

            // Handle selection (set clipboard and hide)
            drop(history); // Release lock before clipboard operation
            if let Some(content) = selected_content {
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(&content);
                }
                *self.visible.lock().unwrap() = false;
                crate::platform::hide_window_native();
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.search_query.clear();
                self.selected_index = 0;
            }
        });
    }
}
