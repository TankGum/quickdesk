//! Self-update through tauri-plugin-updater. A signed manifest on the download
//! site (`update.json`, written by scripts/publish_r2.py) says what the newest
//! version is; packages are verified against the public key in
//! tauri.conf.json before anything is installed. QuickDesk checks on its own,
//! but only installs when the user says so: a .deb or .rpm is installed with
//! pkexec, so the system asks for the administrator password, then the app
//! restarts into the new version.

use std::sync::Mutex;
use std::time::Duration;

use qd_core::settings;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::ipc::now_ms;
use crate::state::AppState;

const SETTING_AUTO: &str = "update.auto_check";
/// Let startup settle before the first check…
const FIRST_CHECK: Duration = Duration::from_secs(30);
/// …then look again a few times a day.
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(tag = "state", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    UpToDate {
        checked_at: i64,
    },
    /// `notes` / `notes_vi`: this version's CHANGELOG section (update.json).
    Available {
        version: String,
        notes: Option<String>,
        notes_vi: Option<String>,
    },
    Downloading {
        version: String,
        downloaded: u64,
        total: Option<u64>,
    },
    Installing {
        version: String,
    },
    /// `version` is set when an update is still waiting to be retried.
    Error {
        message: String,
        version: Option<String>,
    },
}

/// What the UI shows in Settings → Updates and the update banner.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub auto_check: bool,
    pub status: UpdateStatus,
}

pub struct UpdateService {
    status: Mutex<UpdateStatus>,
    /// The update found by the last check, kept until it is installed.
    pending: Mutex<Option<Update>>,
    auto_check: Mutex<bool>,
}

impl UpdateService {
    pub fn new(conn: &rusqlite::Connection) -> Self {
        let auto = settings::get::<bool>(conn, SETTING_AUTO).ok().flatten().unwrap_or(true);
        UpdateService {
            status: Mutex::new(UpdateStatus::Idle),
            pending: Mutex::new(None),
            auto_check: Mutex::new(auto),
        }
    }

    pub fn status(&self) -> UpdateStatus {
        self.status.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Version waiting to be installed, for the tray menu.
    pub fn available_version(&self) -> Option<String> {
        match self.status() {
            UpdateStatus::Available { version, .. } => Some(version),
            _ => None,
        }
    }

    fn busy(&self) -> bool {
        matches!(
            self.status(),
            UpdateStatus::Checking | UpdateStatus::Downloading { .. } | UpdateStatus::Installing { .. }
        )
    }
}

pub fn info(app: &AppHandle) -> UpdateInfo {
    let state = app.state::<AppState>();
    let auto_check = *state.updates.auto_check.lock().unwrap_or_else(|e| e.into_inner());
    UpdateInfo { current_version: app.package_info().version.to_string(), auto_check, status: state.updates.status() }
}

fn set(app: &AppHandle, status: UpdateStatus) {
    let state = app.state::<AppState>();
    let tray_changed = {
        let mut current = state.updates.status.lock().unwrap_or_else(|e| e.into_inner());
        let was_available = matches!(*current, UpdateStatus::Available { .. });
        let now_available = matches!(status, UpdateStatus::Available { .. });
        *current = status;
        was_available != now_available
    };
    // The tray menu offers the update while one is waiting.
    if tray_changed {
        crate::tray::relabel(app);
    }
    let _ = app.emit("update://status", info(app));
}

/// Ask the download site whether there is a newer version.
pub async fn check(app: &AppHandle) {
    if app.state::<AppState>().updates.busy() {
        return;
    }
    set(app, UpdateStatus::Checking);
    let result = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(e) => Err(e),
    };
    match result {
        Ok(Some(update)) => {
            tracing::info!(version = %update.version, "update available");
            // notes_vi is QuickDesk's own field in update.json, next to the standard notes.
            let notes_vi = update.raw_json.get("notes_vi").and_then(|v| v.as_str()).map(str::to_owned);
            let status =
                UpdateStatus::Available { version: update.version.clone(), notes: update.body.clone(), notes_vi };
            *app.state::<AppState>().updates.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(update);
            set(app, status);
        }
        Ok(None) => set(app, UpdateStatus::UpToDate { checked_at: now_ms() as i64 }),
        Err(e) => {
            tracing::warn!(error = %e, "update check failed");
            let version = app.state::<AppState>().updates.available_version();
            set(app, UpdateStatus::Error { message: e.to_string(), version });
        }
    }
}

/// Download, verify and install the pending update, then restart into it.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.updates.busy() {
        return Err("an update is already in progress".into());
    }
    let Some(update) = state.updates.pending.lock().unwrap_or_else(|e| e.into_inner()).clone() else {
        return Err("no update to install; check for updates first".into());
    };
    let version = update.version.clone();
    set(app, UpdateStatus::Downloading { version: version.clone(), downloaded: 0, total: None });

    let mut downloaded = 0u64;
    let mut last_percent = 0u64;
    let (on_chunk, on_done) = (app.clone(), app.clone());
    let (v_chunk, v_done) = (version.clone(), version.clone());
    let result = update
        .download_and_install(
            move |chunk, total| {
                downloaded += chunk as u64;
                // Report about every percent, not every network chunk.
                let percent = total.map_or(0, |t| downloaded * 100 / t.max(1));
                if percent != last_percent || total.is_none() {
                    last_percent = percent;
                    set(&on_chunk, UpdateStatus::Downloading { version: v_chunk.clone(), downloaded, total });
                }
            },
            move || set(&on_done, UpdateStatus::Installing { version: v_done.clone() }),
        )
        .await;

    match result {
        Ok(()) => {
            tracing::info!(%version, "update installed; restarting");
            // Free the single-instance socket first, so the new process does
            // not hand itself over to this exiting one.
            crate::ipc::cleanup();
            app.restart();
        }
        Err(e) => {
            // Cancelling the password prompt lands here too; the update stays
            // available so the user can try again.
            tracing::warn!(error = %e, "update install failed");
            set(app, UpdateStatus::Error { message: e.to_string(), version: Some(version) });
            Err(e.to_string())
        }
    }
}

pub fn set_auto_check(app: &AppHandle, enabled: bool) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    settings::set(&*state.db.conn()?, SETTING_AUTO, &enabled)?;
    *state.updates.auto_check.lock().unwrap_or_else(|e| e.into_inner()) = enabled;
    let _ = app.emit("update://status", info(app));
    Ok(())
}

/// Background checks while automatic checking is on. Debug builds only check
/// when asked, so development runs do not nag about the released version.
pub fn start(app: &AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    let _ = std::thread::Builder::new().name("qd-updater".into()).spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        loop {
            let auto = *app.state::<AppState>().updates.auto_check.lock().unwrap_or_else(|e| e.into_inner());
            if auto {
                tauri::async_runtime::block_on(check(&app));
            }
            std::thread::sleep(CHECK_EVERY);
        }
    });
}
