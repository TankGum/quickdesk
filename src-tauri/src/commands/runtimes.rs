use std::path::PathBuf;

use qd_runtimes::{shellrc, Action, Runtime};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use super::{blocking, CmdError, CmdResult};
use crate::runtimes::{AvailableList, JobState};
use crate::state::AppState;

#[tauri::command]
pub async fn runtimes_scan(app: AppHandle, force: bool) -> CmdResult<Vec<Runtime>> {
    blocking(move || Ok(app.state::<AppState>().runtimes.runtimes(force))).await
}

#[tauri::command]
pub async fn runtimes_available(app: AppHandle, lang: String) -> CmdResult<AvailableList> {
    blocking(move || Ok(app.state::<AppState>().runtimes.available(&lang)?)).await
}

#[tauri::command]
pub fn runtimes_run(app: AppHandle, lang: String, action: String, version: String) -> CmdResult<()> {
    let action = Action::parse(&action).ok_or_else(|| CmdError::new("invalid", format!("unknown action {action}")))?;
    Ok(crate::runtimes::start(&app, lang, action, version)?)
}

#[tauri::command]
pub fn runtimes_upgrade(app: AppHandle, lang: String, from: String, to: String) -> CmdResult<()> {
    Ok(crate::runtimes::upgrade(&app, lang, from, to)?)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangUpdate {
    lang: String,
    #[serde(flatten)]
    update: qd_runtimes::Update,
}

/// Newer patch releases for what is installed. Asks each manager for its
/// list (cached for an hour); a manager that cannot answer is skipped.
#[tauri::command]
pub async fn runtimes_updates(app: AppHandle) -> CmdResult<Vec<LangUpdate>> {
    blocking(move || {
        let svc = &app.state::<AppState>().runtimes;
        let sys = svc.system(false);
        let mut out = Vec::new();
        for rt in sys.runtimes() {
            if rt.manager.is_none() || rt.installed.is_empty() || rt.free_input {
                continue;
            }
            let Ok(list) = svc.available(&rt.id) else { continue };
            for update in sys.updates(&rt.id, &list.versions) {
                out.push(LangUpdate { lang: rt.id.clone(), update });
            }
        }
        Ok(out)
    })
    .await
}

#[tauri::command]
pub fn runtimes_cancel(state: State<'_, AppState>) {
    state.runtimes.cancel();
}

#[tauri::command]
pub fn runtimes_job(state: State<'_, AppState>) -> Option<JobState> {
    state.runtimes.job()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellFix {
    /// `~/.bashrc` as an absolute path.
    file: String,
    /// What QuickDesk's block will hold after the change.
    lines: Vec<String>,
    /// What it holds now (empty: no block yet).
    current: Vec<String>,
}

fn rc_path(app: &AppHandle) -> PathBuf {
    let sys = app.state::<AppState>().runtimes.system(false);
    shellrc::rc_file(&sys.probe.home, &sys.probe.shell)
}

/// What fixing `lang`'s PATH would write, for the user to read first.
#[tauri::command]
pub async fn runtimes_shell_fix(app: AppHandle, lang: String) -> CmdResult<ShellFix> {
    blocking(move || {
        let add = app.state::<AppState>().runtimes.system(false).shell_fix(&lang)?;
        let file = rc_path(&app);
        let content = std::fs::read_to_string(&file).unwrap_or_default();
        let current = shellrc::block_lines(&content);
        let lines = shellrc::block_lines(&shellrc::with_lines(&content, &add));
        Ok(ShellFix { file: file.display().to_string(), lines, current })
    })
    .await
}

fn stamp() -> String {
    // Seconds since the epoch: unique enough and sorts by time.
    format!("{}", crate::ipc::now_ms() / 1000)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellApplied {
    file: String,
    backup: Option<String>,
}

#[tauri::command]
pub async fn runtimes_shell_apply(app: AppHandle, lang: String) -> CmdResult<ShellApplied> {
    blocking(move || {
        let state = app.state::<AppState>();
        let add = state.runtimes.system(false).shell_fix(&lang)?;
        let file = rc_path(&app);
        let backup = shellrc::apply(&file, &add, &stamp())?;
        tracing::info!(file = %file.display(), %lang, "added QuickDesk's PATH block");
        state.runtimes.rescan();
        Ok(ShellApplied { file: file.display().to_string(), backup: backup.map(|b| b.display().to_string()) })
    })
    .await
}

#[tauri::command]
pub async fn runtimes_shell_undo(app: AppHandle) -> CmdResult<bool> {
    blocking(move || {
        let removed = shellrc::undo(&rc_path(&app), &stamp())?;
        app.state::<AppState>().runtimes.rescan();
        Ok(removed)
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoApply {
    on: bool,
    /// The rc file and the line turning it on adds, shown before writing.
    file: String,
    line: String,
    script: String,
}

#[tauri::command]
pub async fn runtimes_auto_apply(app: AppHandle) -> CmdResult<AutoApply> {
    blocking(move || {
        let sys = app.state::<AppState>().runtimes.system(false);
        Ok(AutoApply {
            on: sys.auto_apply(),
            file: sys.rc_file().display().to_string(),
            line: qd_runtimes::hook::line(&sys.probe),
            script: qd_runtimes::hook::script_path(&sys.probe).display().to_string(),
        })
    })
    .await
}

#[tauri::command]
pub async fn runtimes_set_auto_apply(app: AppHandle, on: bool) -> CmdResult<ShellApplied> {
    blocking(move || {
        let state = app.state::<AppState>();
        let sys = state.runtimes.system(false);
        let backup = sys.set_auto_apply(on, &stamp())?;
        tracing::info!(on, "prompt hook for open terminals");
        state.runtimes.rescan();
        Ok(ShellApplied { file: sys.rc_file().display().to_string(), backup: backup.map(|b| b.display().to_string()) })
    })
    .await
}

#[tauri::command]
pub async fn runtimes_pick_folder(app: AppHandle) -> CmdResult<Option<String>> {
    blocking(move || {
        Ok(app.dialog().file().blocking_pick_folder().and_then(|p| p.into_path().ok()).map(|p| p.display().to_string()))
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPin {
    file: String,
    /// What the file says now, if it exists.
    content: Option<String>,
}

#[tauri::command]
pub fn runtimes_project_get(state: State<'_, AppState>, dir: String, lang: String) -> CmdResult<ProjectPin> {
    let name = state.runtimes.system(false).project_file(&lang)?;
    let file = PathBuf::from(dir).join(name);
    let content = std::fs::read_to_string(&file).ok().map(|c| c.chars().take(2000).collect());
    Ok(ProjectPin { file: file.display().to_string(), content })
}

#[tauri::command]
pub fn runtimes_project_set(
    state: State<'_, AppState>,
    dir: String,
    lang: String,
    version: String,
) -> CmdResult<String> {
    let path = state.runtimes.system(false).set_project(&PathBuf::from(dir), &lang, &version)?;
    tracing::info!(file = %path.display(), %version, "pinned a project version");
    Ok(path.display().to_string())
}
