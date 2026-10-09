//! Background sync worker for Quick Notes, through QuickDesk Cloud
//! (`qd_sync::cloud`, server in sync-server/). The sync account's token lives
//! in the OS keyring; settings only hold the account id and endpoint.
//!
//! Runs on its own thread so network I/O never blocks the UI. Triggers:
//! a local note change (debounced), a window being shown, a manual "sync
//! now", and a periodic timer (slower while no window is open). Failures back
//! off exponentially. An idle round is one cheap call (`SyncEngine::sync_once`).

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use qd_core::settings;
use qd_sync::crypto::Dek;
use qd_sync::engine::reset_local_state;
use qd_sync::{Account, CloudTransport, EngineConfig, Layout, SyncEngine, SyncReport};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::ipc::now_ms;
use crate::secrets;
use crate::state::AppState;

/// Where this device syncs: `CloudConfig`.
pub const CONFIG_KEY: &str = "sync.cloud";
/// Up to 0.3.0: a bucket the user configured (S3Config). No longer supported.
const LEGACY_S3_KEY: &str = "sync.config";
/// Set once the old bucket setup was removed, so the UI can say why sync is off.
pub const NOTICE_KEY: &str = "sync.notice";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudConfig {
    pub account: String,
    pub endpoint: String,
}

/// Everything sync stores in QuickDesk Cloud lives at the root of the account.
pub fn layout() -> Layout {
    Layout::new("")
}

const PERIOD: Duration = Duration::from_secs(60);
/// While every window is hidden; showing one syncs right away.
const PERIOD_HIDDEN: Duration = Duration::from_secs(300);
const LOCAL_CHANGE_DEBOUNCE: Duration = Duration::from_secs(2);
const MIN_GAP_ON_SHOW: Duration = Duration::from_secs(10);
const BACKOFF_START: Duration = Duration::from_secs(5);
const BACKOFF_MAX: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    LocalChange,
    WindowShown,
    Now,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    /// disabled | locked | idle | syncing | offline | error
    pub state: &'static str,
    pub last_sync_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_report: Option<SyncReport>,
    /// The sync account this device uses, when sync is on.
    pub account: Option<String>,
    /// e.g. `bucket_removed` after the old own-bucket setup was dropped.
    pub notice: Option<String>,
}

pub struct SyncService {
    tx: Mutex<Option<Sender<Trigger>>>,
    status: Mutex<SyncStatus>,
}

impl SyncService {
    pub fn new() -> Self {
        SyncService {
            tx: Mutex::new(None),
            status: Mutex::new(SyncStatus {
                state: "disabled",
                last_sync_at: None,
                last_error: None,
                last_report: None,
                account: None,
                notice: None,
            }),
        }
    }

    pub fn trigger(&self, t: Trigger) {
        if let Some(tx) = self.tx.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = tx.send(t);
        }
    }

    pub fn status(&self) -> SyncStatus {
        self.status.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

fn update(app: &AppHandle, f: impl FnOnce(&mut SyncStatus)) {
    let state = app.state::<AppState>();
    let snapshot = {
        let mut s = state.sync.status.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut s);
        s.clone()
    };
    let _ = app.emit("sync://status", snapshot);
}

pub fn device_name() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "device".into())
}

/// Everything needed to reach the sync account, or why we cannot.
pub enum Setup {
    Disabled,
    /// Account known, encryption key not on this device (e.g. keyring cleared).
    Locked {
        config: CloudConfig,
        account: Account,
    },
    Unlocked {
        config: CloudConfig,
        account: Account,
        dek: Dek,
    },
}

impl Setup {
    pub fn transport(config: &CloudConfig, account: &Account) -> CloudTransport {
        CloudTransport::new(&config.endpoint, account)
    }
}

pub fn load(app: &AppHandle) -> Result<Setup, String> {
    let state = app.state::<AppState>();
    let config: Option<CloudConfig> =
        settings::get(&*state.db.conn().map_err(|e| e.to_string())?, CONFIG_KEY).map_err(|e| e.to_string())?;
    let Some(config) = config else { return Ok(Setup::Disabled) };
    let token =
        secrets::get(secrets::SYNC_TOKEN)?.ok_or("sync code missing from the OS keyring; turn sync on again")?;
    let account = Account { account: config.account.clone(), token };
    match secrets::get(secrets::SYNC_DEK)? {
        None => Ok(Setup::Locked { config, account }),
        Some(b64) => Ok(Setup::Unlocked { dek: Dek::from_base64(&b64).map_err(|e| e.to_string())?, config, account }),
    }
}

/// QuickDesk up to 0.3.0 synced through a bucket the user set up. That option
/// is gone: forget it (and its secrets) once, keep the notes, and remember to
/// tell the user why sync is off.
fn retire_own_bucket(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Ok(conn) = state.db.conn() else { return };
    let had: Option<serde_json::Value> = settings::get(&conn, LEGACY_S3_KEY).ok().flatten();
    if had.is_none() {
        return;
    }
    let _ = conn.execute("DELETE FROM settings WHERE key = ?1", [LEGACY_S3_KEY]);
    let _ = reset_local_state(&conn);
    let _ = settings::set(&conn, NOTICE_KEY, &"bucket_removed");
    drop(conn);
    for name in [secrets::S3_SECRET, secrets::SYNC_DEK] {
        let _ = secrets::delete(name);
    }
    tracing::info!("removed the old own-bucket sync setup; sync now uses QuickDesk Cloud");
}

fn notice(app: &AppHandle) -> Option<String> {
    let state = app.state::<AppState>();
    let conn = state.db.conn().ok()?;
    settings::get(&conn, NOTICE_KEY).ok().flatten()
}

/// Refresh the status from configuration without syncing.
pub fn refresh_status(app: &AppHandle) {
    match load(app) {
        Ok(Setup::Disabled) => {
            let notice = notice(app);
            update(app, |s| {
                s.state = "disabled";
                s.account = None;
                s.last_error = None;
                s.notice = notice;
            })
        }
        Ok(Setup::Locked { config, .. }) => update(app, |s| {
            s.state = "locked";
            s.account = Some(config.account);
            s.notice = None;
        }),
        Ok(Setup::Unlocked { config, .. }) => update(app, |s| {
            if matches!(s.state, "disabled" | "locked") {
                s.state = "idle";
            }
            s.account = Some(config.account);
            s.notice = None;
        }),
        Err(e) => update(app, |s| {
            s.state = "error";
            s.last_error = Some(e);
        }),
    }
}

fn period(app: &AppHandle) -> Duration {
    let visible = app.webview_windows().values().any(|w| w.is_visible().unwrap_or(false));
    if visible {
        PERIOD
    } else {
        PERIOD_HIDDEN
    }
}

fn run_once(app: &AppHandle) -> Result<Option<SyncReport>, (bool, String)> {
    let Setup::Unlocked { config, account, dek } = load(app).map_err(|e| (false, e))? else {
        refresh_status(app);
        return Ok(None);
    };
    let transport = Setup::transport(&config, &account);
    update(app, |s| s.state = "syncing");
    let state = app.state::<AppState>();
    let name = device_name();
    let engine = SyncEngine {
        transport: &transport,
        dek: &dek,
        layout: layout(),
        device_id: &state.device_id,
        device_name: &name,
        config: EngineConfig::default(),
    };
    engine.sync_once(&state.db, &state.clock).map(Some).map_err(|e| (e.is_transient(), e.to_string()))
}

pub fn start(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<Trigger>();
    *app.state::<AppState>().sync.tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    retire_own_bucket(app);
    refresh_status(app);

    let app = app.clone();
    let _ = std::thread::Builder::new().name("qd-sync".into()).spawn(move || {
        let mut next = Instant::now() + Duration::from_secs(3);
        let mut last_run: Option<Instant> = None;
        let mut backoff = BACKOFF_START;
        loop {
            let now = Instant::now();
            match rx.recv_timeout(next.saturating_duration_since(now)) {
                Ok(Trigger::LocalChange) => {
                    next = next.min(now + LOCAL_CHANGE_DEBOUNCE);
                    continue;
                }
                Ok(Trigger::WindowShown) => {
                    let earliest = last_run.map_or(now, |t| (t + MIN_GAP_ON_SHOW).max(now));
                    next = next.min(earliest);
                    continue;
                }
                Ok(Trigger::Now) => {}
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            last_run = Some(Instant::now());
            match run_once(&app) {
                Ok(Some(report)) => {
                    if report != SyncReport::default() {
                        tracing::info!(?report, "sync ok");
                    }
                    if report.pulled > 0 {
                        let _ = app.emit("notes://changed", ());
                    }
                    update(&app, |s| {
                        s.state = "idle";
                        s.last_sync_at = Some(now_ms() as i64);
                        s.last_error = None;
                        s.last_report = Some(report);
                    });
                    backoff = BACKOFF_START;
                    next = Instant::now() + period(&app);
                }
                Ok(None) => next = Instant::now() + period(&app),
                Err((transient, msg)) => {
                    tracing::warn!(transient, error = %msg, "sync failed");
                    update(&app, |s| {
                        s.state = if transient { "offline" } else { "error" };
                        s.last_error = Some(msg);
                    });
                    next = Instant::now() + if transient { backoff } else { BACKOFF_MAX };
                    backoff = (backoff * 2).min(BACKOFF_MAX);
                }
            }
        }
    });
}
