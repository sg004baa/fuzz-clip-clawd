# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`clipboard-history` is a Rust-based desktop GUI application — a system tray clipboard history manager with fuzzy search. It monitors the clipboard, stores history to JSON, and provides a hotkey-toggled window (Ctrl+Ctrl double-tap) for fuzzy searching and restoring prior clipboard entries.

## Common Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run in dev mode
cargo test                   # Run all tests
cargo test --lib             # Library/unit tests only
cargo clippy                 # Lint
cargo fmt                    # Format code
```

To run a single test:
```bash
cargo test <test_name>       # e.g., cargo test test_push_dedup
```

## Architecture

### Thread Model
The app uses a multi-thread architecture with shared state via `Arc<Mutex<T>>`:

- **Main thread**: egui GUI event loop (`app.rs`)
- **Clipboard monitor thread** (`clipboard.rs`): polls every 500ms for clipboard changes, auto-saves on change
- **Hotkey listener thread** (`hotkey.rs`): global keyboard listener detecting Ctrl+Ctrl double-tap (300ms window)
- **Tray thread** (`tray.rs`): system tray icon and Show/Hide/Quit menu

Background threads are lazily started on the **first GUI frame** (when egui Context is available), not in `main()`.

### Key Shared State
- `Arc<Mutex<History>>` — clipboard entry list
- `Arc<Mutex<bool>>` — window visibility flag, toggled by hotkey, tray menu, Escape key

### Module Responsibilities

| Module | Role |
|--------|------|
| `main.rs` | Initialization, window setup (400×500, borderless, always-on-top), thread spawning |
| `app.rs` | `ClipboardHistoryApp` — UI rendering, keyboard nav, selection/copy logic |
| `history.rs` | `History`/`ClipboardEntry` — FIFO with dedup (duplicates move to front with updated timestamp) |
| `clipboard.rs` | Background monitor, triggers save and GUI repaint on new content |
| `fuzzy.rs` | `SkimMatcherV2`-based fuzzy search returning score-ranked results |
| `config.rs` | `Config` struct (defaults: `max_size=100`, `poll_interval_ms=500`) |
| `storage.rs` | JSON persistence via `dirs::config_dir()` (e.g., `~/.config/clipboard-history/history.json`) |
| `hotkey.rs` | `rdev` global listener, Ctrl+Ctrl double-tap detection |
| `tray.rs` | `tray-icon` system tray with blue 16×16 icon |
| `platform.rs` | Windows-only Win32 calls (`ShowWindow`, `SetForegroundWindow`, `FindWindowW`) for native window control |

### Platform Notes
- Windows requires direct Win32 API calls in `platform.rs` to properly show/hide the window outside the egui event loop; non-Windows uses egui's repaint mechanism.
- `#![cfg_attr(windows, windows_subsystem = "windows")]` suppresses the console window on Windows.
- `windows-sys` is a Windows-only dependency in `Cargo.toml`.

### User Interactions
- **Type**: fuzzy filters history
- **Arrow keys**: navigate results
- **Enter**: copy selected entry to clipboard, hide window
- **Escape**: hide window
- **Ctrl+Ctrl** (global): toggle window visibility
- **Tray menu**: Show/Hide or Quit

### Tests
Unit tests live in the same files as the modules they test:
- `history.rs` — push/dedup/max-size enforcement
- `fuzzy.rs` — matching, scoring, filtering
- `storage.rs` — save/load roundtrip, error handling
