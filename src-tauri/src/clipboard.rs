//! Clipboard history service: feeds watcher events into the database.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use qd_clipboard::{Ingested, Policy};
use qd_core::settings;
use qd_platform::clipboard::{spawn_watcher, WatcherHandle, WatcherState};
use serde::Serialize;
use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::ipc::now_ms;
use crate::state::AppState;

const PAUSED_KEY: &str = "clipboard.paused";
const PRUNE_EVERY: Duration = Duration::from_secs(3600);

pub struct ClipboardService {
    pub policy: Policy,
    paused: AtomicBool,
    watcher: Mutex<Option<WatcherHandle>>,
    /// Tray checkbox, kept in sync when pausing from the UI.
    pub tray_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    backend: &'static str,
    /// starting | running | retrying | unavailable
    state: &'static str,
    detail: Option<String>,
    paused: bool,
}

impl ClipboardService {
    pub fn new(paused: bool) -> Self {
        ClipboardService {
            policy: Policy::default(),
            paused: AtomicBool::new(paused),
            watcher: Mutex::new(None),
            tray_item: Mutex::new(None),
        }
    }

    pub fn load_paused(conn: &rusqlite::Connection) -> bool {
        settings::get(conn, PAUSED_KEY).ok().flatten().unwrap_or(false)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn status(&self) -> ClipStatus {
        let watcher = self.watcher.lock().unwrap_or_else(|e| e.into_inner());
        let (backend, state, detail) = match watcher.as_ref() {
            None => ("none", "starting", None),
            Some(w) => match w.state() {
                WatcherState::Starting => (w.backend, "starting", None),
                WatcherState::Running => (w.backend, "running", None),
                WatcherState::Retrying(e) => (w.backend, "retrying", Some(e)),
                WatcherState::Unavailable(e) => (w.backend, "unavailable", Some(e)),
            },
        };
        ClipStatus { backend, state, detail, paused: self.is_paused() }
    }
}

/// Pause or resume recording; persists, updates the tray and notifies windows.
pub fn set_paused(app: &AppHandle, paused: bool) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    state.clipboard.paused.store(paused, Ordering::Relaxed);
    settings::set(&*state.db.conn()?, PAUSED_KEY, &paused)?;
    if let Some(item) = state.clipboard.tray_item.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = item.set_checked(paused);
    }
    let _ = app.emit("clipboard://changed", ());
    tracing::info!(paused, "clipboard history");
    Ok(())
}

/// Start the watcher and the hourly retention job.
pub fn start(app: &AppHandle) {
    let state = app.state::<AppState>();
    let handle = app.clone();
    let watcher = spawn_watcher(move |ev| {
        let state = handle.state::<AppState>();
        if state.clipboard.is_paused() {
            return;
        }
        let result = state.db.conn().map_err(|e| e.to_string()).and_then(|conn| {
            qd_clipboard::ingest(&conn, &ev.text, ev.source_app.as_deref(), now_ms() as i64, &state.clipboard.policy)
                .map_err(|e| e.to_string())
        });
        match result {
            Ok(Ingested::Inserted(_) | Ingested::Bumped(_)) => {
                let _ = handle.emit("clipboard://changed", ());
            }
            Ok(Ingested::Skipped(reason)) => tracing::debug!(?reason, "clipboard event skipped"),
            Err(e) => tracing::error!(error = %e, "failed to store clipboard entry"),
        }
    });
    tracing::info!(backend = watcher.backend, "clipboard watcher started");
    *state.clipboard.watcher.lock().unwrap_or_else(|e| e.into_inner()) = Some(watcher);

    let handle = app.clone();
    let _ = std::thread::Builder::new().name("qd-clip-prune".into()).spawn(move || loop {
        let state = handle.state::<AppState>();
        match state.db.conn().map(|c| qd_clipboard::prune(&c, &state.clipboard.policy, now_ms() as i64)) {
            Ok(Ok(n)) if n > 0 => {
                tracing::info!(removed = n, "clipboard retention");
                let _ = handle.emit("clipboard://changed", ());
            }
            Ok(Err(e)) => tracing::error!(error = %e, "clipboard prune failed"),
            _ => {}
        }
        std::thread::sleep(PRUNE_EVERY);
    });
}
