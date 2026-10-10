//! Runtimes tab: the last scan of this computer, the cached version lists, and
//! the one install/uninstall/default change that may run at a time. Jobs run
//! on their own thread and report through `runtimes://job`; when one ends the
//! computer is scanned again and `runtimes://changed` fires.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use qd_runtimes::{Action, Available, Runtime, System, SystemRunner};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::ipc::now_ms;
use crate::state::AppState;

/// Version lists change rarely; ask the manager at most this often.
const AVAILABLE_TTL: Duration = Duration::from_secs(3600);
/// A scan this recent is as good as a new one.
const FRESH: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobState {
    pub lang: String,
    pub action: &'static str,
    pub version: String,
    pub running: bool,
    /// When the tool prints one.
    pub percent: Option<f32>,
    /// Latest output line.
    pub line: String,
    pub error: Option<String>,
    pub cancelled: bool,
    pub started_at: i64,
    /// `upgrade`: the version being replaced (it stays installed).
    pub from: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableList {
    pub versions: Vec<Available>,
    pub fetched_at: i64,
    /// Could not refresh; these are from `fetched_at`.
    pub stale: bool,
}

#[derive(Default)]
pub struct RuntimesService {
    system: Mutex<Option<(Instant, Arc<System>)>>,
    available: Mutex<HashMap<String, (Instant, AvailableList)>>,
    job: Mutex<Option<JobState>>,
    runner: Arc<SystemRunner>,
}

impl RuntimesService {
    /// The cached scan, or a fresh one. A forced scan right after another
    /// (the tab shows the cache, then rescans) reuses it.
    pub fn system(&self, force: bool) -> Arc<System> {
        {
            let cached = self.system.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((at, s)) = cached.as_ref() {
                if !force || at.elapsed() < FRESH {
                    return s.clone();
                }
            }
        }
        let fresh = Arc::new(System::detect());
        *self.system.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), fresh.clone()));
        fresh
    }

    /// After a change: always scan again.
    pub fn rescan(&self) -> Arc<System> {
        let fresh = Arc::new(System::detect());
        *self.system.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), fresh.clone()));
        fresh
    }

    pub fn runtimes(&self, force: bool) -> Vec<Runtime> {
        self.system(force).runtimes()
    }

    pub fn available(&self, lang: &str) -> qd_runtimes::Result<AvailableList> {
        let cached = self.available.lock().unwrap_or_else(|e| e.into_inner()).get(lang).cloned();
        if let Some((at, list)) = &cached {
            if at.elapsed() < AVAILABLE_TTL {
                return Ok(list.clone());
            }
        }
        let runner = SystemRunner::with_timeout(Duration::from_secs(60));
        match self.system(false).available(lang, &runner) {
            Ok(versions) => {
                let list = AvailableList { versions, fetched_at: now_ms() as i64, stale: false };
                self.available
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(lang.to_owned(), (Instant::now(), list.clone()));
                Ok(list)
            }
            // Offline: the older list is better than nothing.
            Err(e) => match cached {
                Some((_, list)) => Ok(AvailableList { stale: true, ..list }),
                None => Err(e),
            },
        }
    }

    pub fn job(&self) -> Option<JobState> {
        self.job.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn cancel(&self) {
        self.runner.cancel();
    }
}

fn action_id(a: Action) -> &'static str {
    match a {
        Action::Install => "install",
        Action::Uninstall => "uninstall",
        Action::SetDefault => "set_default",
        Action::InstallManager => "install_manager",
    }
}

fn is_progress_bar(line: &str) -> bool {
    line.chars().all(|c| matches!(c, '#' | '=' | '-' | '>' | ' ' | '.' | '%') || c.is_ascii_digit())
}

fn emit_job(app: &AppHandle, job: &JobState) {
    let _ = app.emit("runtimes://job", job);
}

/// Start a job. Fails right away when one is running or the request is bad.
pub fn start(app: &AppHandle, lang: String, action: Action, version: String) -> qd_runtimes::Result<()> {
    let cmd = app.state::<AppState>().runtimes.system(false).command(&lang, action, &version)?;
    run_job(app, lang, action_id(action), version, None, vec![cmd])
}

/// Move to a newer release of the same line (see `System::upgrade_commands`).
pub fn upgrade(app: &AppHandle, lang: String, from: String, to: String) -> qd_runtimes::Result<()> {
    let cmds = app.state::<AppState>().runtimes.system(false).upgrade_commands(&lang, &from, &to)?;
    run_job(app, lang, "upgrade", to, Some(from), cmds)
}

fn run_job(
    app: &AppHandle,
    lang: String,
    action: &'static str,
    version: String,
    from: Option<String>,
    cmds: Vec<qd_runtimes::Cmd>,
) -> qd_runtimes::Result<()> {
    let state = app.state::<AppState>();
    let svc = &state.runtimes;
    {
        let mut job = svc.job.lock().unwrap_or_else(|e| e.into_inner());
        if job.as_ref().is_some_and(|j| j.running) {
            return Err(qd_runtimes::Error::Invalid("another version change is still running".into()));
        }
        *job = Some(JobState {
            lang: lang.clone(),
            action,
            version: version.clone(),
            running: true,
            percent: None,
            line: String::new(),
            error: None,
            cancelled: false,
            started_at: now_ms() as i64,
            from,
        });
    }
    if let Some(j) = svc.job() {
        emit_job(app, &j);
    }
    tracing::info!(%lang, action, %version, "version change started");

    let app = app.clone();
    let runner = svc.runner.clone();
    std::thread::Builder::new()
        .name("qd-runtimes".into())
        .spawn(move || {
            let mut last_emit = Instant::now();
            let result = {
                let app = app.clone();
                let mut on_line = move |line: &str| {
                    let state = app.state::<AppState>();
                    let snapshot = {
                        let mut job = state.runtimes.job.lock().unwrap_or_else(|e| e.into_inner());
                        let Some(j) = job.as_mut() else { return };
                        if let Some(p) = qd_runtimes::run::percent(line) {
                            j.percent = Some(p);
                        }
                        // A bare progress bar (`#####   8.7%`) is already the bar in the UI.
                        if !is_progress_bar(line) {
                            j.line = line.chars().take(200).collect();
                        }
                        j.clone()
                    };
                    // Progress bars print many updates per second.
                    if last_emit.elapsed() > Duration::from_millis(120) {
                        last_emit = Instant::now();
                        emit_job(&app, &snapshot);
                    }
                };
                // Steps run in order; the first failure ends the job.
                cmds.iter().try_for_each(|cmd| qd_runtimes::Runner::run(runner.as_ref(), cmd, &mut on_line).map(drop))
            };
            let state = app.state::<AppState>();
            // Open terminals with the prompt hook pick the change up at their
            // next prompt. Without the hook nothing reads the mark.
            let sys = state.runtimes.system(false);
            if result.is_ok() && sys.auto_apply() {
                if let Err(e) = qd_runtimes::hook::mark_changed(&sys.probe, &now_ms().to_string()) {
                    tracing::warn!(error = %e, "could not mark the version change for open terminals");
                }
            }
            // What changed is visible right away in the tab.
            state.runtimes.rescan();
            state.runtimes.available.lock().unwrap_or_else(|e| e.into_inner()).clear();
            let snapshot = {
                let mut job = state.runtimes.job.lock().unwrap_or_else(|e| e.into_inner());
                let Some(j) = job.as_mut() else { return };
                j.running = false;
                match &result {
                    Ok(_) => j.percent = Some(100.0),
                    Err(qd_runtimes::Error::Cancelled) => j.cancelled = true,
                    Err(e) => j.error = Some(e.to_string()),
                }
                j.clone()
            };
            match &result {
                Ok(_) => tracing::info!(lang = %snapshot.lang, action = snapshot.action, "version change done"),
                Err(e) => {
                    tracing::warn!(lang = %snapshot.lang, action = snapshot.action, error = %e, "version change failed")
                }
            }
            emit_job(&app, &snapshot);
            let _ = app.emit("runtimes://changed", ());
        })
        .map_err(|e| qd_runtimes::Error::Io(e.to_string()))?;
    Ok(())
}
