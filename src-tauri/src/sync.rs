//! Background sync worker for Quick Notes.
//!
//! Runs on its own thread so network I/O never blocks the UI. Triggers:
//! a local note change (debounced), a window being shown, a manual "sync
//! now", and a periodic timer. Failures back off exponentially.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use qd_core::settings;
use qd_sync::crypto::Dek;
use qd_sync::{EngineConfig, Layout, S3Config, S3Transport, SyncEngine, SyncReport};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::ipc::now_ms;
use crate::secrets;
use crate::state::AppState;

pub const CONFIG_KEY: &str = "sync.config";

const PERIOD: Duration = Duration::from_secs(60);
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
    pub config: Option<S3Config>,
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
                config: None,
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

/// Everything needed to talk to the bucket, or why we cannot.
pub enum Setup {
    Disabled,
    Locked { config: S3Config, transport: S3Transport },
    Unlocked { config: S3Config, transport: S3Transport, dek: Dek },
}

pub fn load(app: &AppHandle) -> Result<Setup, String> {
    let state = app.state::<AppState>();
    let config: Option<S3Config> =
        settings::get(&*state.db.conn().map_err(|e| e.to_string())?, CONFIG_KEY).map_err(|e| e.to_string())?;
    let Some(config) = config else { return Ok(Setup::Disabled) };
    let secret = secrets::get(secrets::S3_SECRET)?.ok_or("storage secret missing from OS keyring; reconnect sync")?;
    let transport = S3Transport::new(&config, &secret).map_err(|e| e.to_string())?;
    match secrets::get(secrets::SYNC_DEK)? {
        None => Ok(Setup::Locked { config, transport }),
        Some(b64) => Ok(Setup::Unlocked { dek: Dek::from_base64(&b64).map_err(|e| e.to_string())?, config, transport }),
    }
}

/// Refresh the status from configuration without syncing.
pub fn refresh_status(app: &AppHandle) {
    match load(app) {
        Ok(Setup::Disabled) => update(app, |s| {
            s.state = "disabled";
            s.config = None;
            s.last_error = None;
        }),
        Ok(Setup::Locked { config, .. }) => update(app, |s| {
            s.state = "locked";
            s.config = Some(config);
        }),
        Ok(Setup::Unlocked { config, .. }) => update(app, |s| {
            if matches!(s.state, "disabled" | "locked") {
                s.state = "idle";
            }
            s.config = Some(config);
        }),
        Err(e) => update(app, |s| {
            s.state = "error";
            s.last_error = Some(e);
        }),
    }
}

fn run_once(app: &AppHandle) -> Result<Option<SyncReport>, (bool, String)> {
    let Setup::Unlocked { config, transport, dek } = load(app).map_err(|e| (false, e))? else {
        refresh_status(app);
        return Ok(None);
    };
    update(app, |s| {
        s.state = "syncing";
        s.config = Some(config.clone());
    });
    let state = app.state::<AppState>();
    let name = device_name();
    let engine = SyncEngine {
        transport: &transport,
        dek: &dek,
        layout: Layout::new(&config.prefix),
        device_id: &state.device_id,
        device_name: &name,
        config: EngineConfig::default(),
    };
    engine.sync_once(&state.db, &state.clock).map(Some).map_err(|e| (e.is_transient(), e.to_string()))
}

pub fn start(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<Trigger>();
    *app.state::<AppState>().sync.tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
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
                    tracing::info!(?report, "sync ok");
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
                    next = Instant::now() + PERIOD;
                }
                Ok(None) => next = Instant::now() + PERIOD,
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
