//! Clipboard history service: feeds watcher events into the database.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use qd_clipboard::{Ingested, Policy};
use qd_core::settings;
use qd_platform::clipboard::{spawn_watcher, WatcherHandle, WatcherState};
use qd_platform::paste::{AutoPaster, PasteMethod};
use serde::Serialize;
use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::ipc::now_ms;
use crate::state::AppState;

const PAUSED_KEY: &str = "clipboard.paused";
const AUTO_PASTE_KEY: &str = "clipboard.auto_paste";
const PASTE_TOKEN_KEY: &str = "clipboard.paste_token";
const PASTE_METHOD_KEY: &str = "clipboard.paste_method";
/// Close the RemoteDesktop session (and GNOME's indicator) after this long unused.
const PASTE_SESSION_IDLE: Duration = Duration::from_secs(45);
const PRUNE_EVERY: Duration = Duration::from_secs(3600);

pub struct ClipboardService {
    pub policy: Policy,
    paused: AtomicBool,
    watcher: Mutex<Option<WatcherHandle>>,
    /// Tray checkbox, kept in sync when pausing from the UI.
    pub tray_item: Mutex<Option<CheckMenuItem<Wry>>>,
    auto_paste: AtomicBool,
    pub paster: AutoPaster,
    paste_method: Mutex<PasteMethod>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    backend: &'static str,
    /// starting | running | retrying | unavailable
    state: &'static str,
    detail: Option<String>,
    paused: bool,
    /// Picking an entry types it into the previous app (else: copy only).
    auto_paste: bool,
}

impl ClipboardService {
    pub fn new(conn: &rusqlite::Connection) -> Self {
        ClipboardService {
            policy: Policy::default(),
            paused: AtomicBool::new(settings::get(conn, PAUSED_KEY).ok().flatten().unwrap_or(false)),
            watcher: Mutex::new(None),
            tray_item: Mutex::new(None),
            auto_paste: AtomicBool::new(settings::get(conn, AUTO_PASTE_KEY).ok().flatten().unwrap_or(true)),
            paster: AutoPaster::default(),
            paste_method: Mutex::new(settings::get(conn, PASTE_METHOD_KEY).ok().flatten().unwrap_or_default()),
        }
    }

    pub fn paste_method(&self) -> PasteMethod {
        *self.paste_method.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn auto_paste(&self) -> bool {
        self.auto_paste.load(Ordering::Relaxed)
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
        ClipStatus { backend, state, detail, paused: self.is_paused(), auto_paste: self.auto_paste() }
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

pub fn set_paste_method(app: &AppHandle, method: PasteMethod) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    *state.clipboard.paste_method.lock().unwrap_or_else(|e| e.into_inner()) = method;
    settings::set(&*state.db.conn()?, PASTE_METHOD_KEY, &method)?;
    Ok(())
}

pub fn set_auto_paste(app: &AppHandle, enabled: bool) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    state.clipboard.auto_paste.store(enabled, Ordering::Relaxed);
    settings::set(&*state.db.conn()?, AUTO_PASTE_KEY, &enabled)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(())
}

/// Type the clipboard into the focused app, remembering the portal grant.
pub async fn paste_into_focused(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let token: Option<String> =
        settings::get(&*state.db.conn().map_err(|e| e.to_string())?, PASTE_TOKEN_KEY).map_err(|e| e.to_string())?;
    let new_token = state.clipboard.paster.paste(state.clipboard.paste_method(), token.as_deref()).await?;
    if let Some(t) = new_token {
        // Tokens are single-use: always keep the latest one.
        settings::set(&*state.db.conn().map_err(|e| e.to_string())?, PASTE_TOKEN_KEY, &t).map_err(|e| e.to_string())?;
    }
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
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            handle.state::<AppState>().clipboard.paster.close_if_idle(PASTE_SESSION_IDLE).await;
        }
    });

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
