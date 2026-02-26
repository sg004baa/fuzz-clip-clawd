use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::clipboard;
use crate::config::Config;
use crate::fuzzy;
use crate::history::History;
use crate::hotkey;

pub struct ClipboardHistoryApp {
    history: Arc<Mutex<History>>,
    search_query: String,
    selected_index: usize,
    visible: Arc<Mutex<bool>>,
    config: Config,
    initialized: bool,
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
        }
    }
}

impl eframe::App for ClipboardHistoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start background threads on first frame
        if !self.initialized {
            self.initialized = true;

            // Start clipboard monitor
            clipboard::start_monitor(
                Arc::clone(&self.history),
                std::time::Duration::from_millis(self.config.poll_interval_ms),
                ctx.clone(),
            );

            // Start hotkey listener
            hotkey::start_listener(Arc::clone(&self.visible), ctx.clone());
        }

        // Check visibility
        let is_visible = *self.visible.lock().unwrap();
        if !is_visible {
            // Hide the window by not rendering any UI
            // Request repaint to check visibility flag changes
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            // Minimize or set as not visible - we handle this through viewport commands
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        // Show the window
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        // Poll for repaint to keep checking visibility
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Handle Escape key to hide
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            *self.visible.lock().unwrap() = false;
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

            // Handle selection (set clipboard and hide)
            drop(history); // Release lock before clipboard operation
            if let Some(content) = selected_content {
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(&content);
                }
                *self.visible.lock().unwrap() = false;
                self.search_query.clear();
                self.selected_index = 0;
            }
        });
    }
}
