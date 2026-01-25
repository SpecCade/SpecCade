//! File watcher for external file changes.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

/// Event emitted when a watched file changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    /// The path that changed.
    pub path: String,
    /// The kind of change (created, modified, removed).
    pub kind: String,
}

/// State for the file watcher.
pub struct WatcherState {
    watcher: Option<RecommendedWatcher>,
    watched_path: Option<PathBuf>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            watcher: None,
            watched_path: None,
        }
    }
}

/// Start watching a file for changes.
#[tauri::command]
pub fn watch_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    state: tauri::State<'_, std::sync::Mutex<WatcherState>>,
) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Stop existing watcher if any
    state.watcher = None;
    state.watched_path = None;

    // Create channel for debouncing
    let (tx, rx) = mpsc::channel();

    // Create watcher with debounce
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )
    .map_err(|e| e.to_string())?;

    // Watch the file's parent directory (notify requires directory watching on some platforms)
    let watch_path = path.parent().unwrap_or(&path);
    watcher
        .watch(watch_path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    // Spawn thread to handle events and emit to frontend
    let watched_file = path.clone();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        while let Ok(event) = rx.recv() {
            // Only emit for our specific file
            let is_our_file = event.paths.iter().any(|p| p == &watched_file);
            if !is_our_file {
                continue;
            }

            // Debounce: don't emit more than once per 100ms
            if last_emit.elapsed() < Duration::from_millis(100) {
                continue;
            }
            last_emit = std::time::Instant::now();

            let kind = match event.kind {
                notify::EventKind::Create(_) => "created",
                notify::EventKind::Modify(_) => "modified",
                notify::EventKind::Remove(_) => "removed",
                _ => continue,
            };

            let change_event = FileChangeEvent {
                path: watched_file.to_string_lossy().to_string(),
                kind: kind.to_string(),
            };

            let _ = app.emit("file-changed", change_event);
        }
    });

    state.watcher = Some(watcher);
    state.watched_path = Some(path);

    Ok(())
}

/// Stop watching the current file.
#[tauri::command]
pub fn unwatch_file(state: tauri::State<'_, std::sync::Mutex<WatcherState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.watcher = None;
    state.watched_path = None;
    Ok(())
}
