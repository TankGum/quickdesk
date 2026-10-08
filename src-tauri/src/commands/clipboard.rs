use std::time::Duration;

use qd_clipboard::{ClipEntry, Payload};
use qd_platform::paste::Chord;

use tauri::{AppHandle, Emitter, Manager, State};

use super::{CmdError, CmdResult};
use crate::clipboard::ClipStatus;
use crate::ipc::now_ms;
use crate::state::AppState;

const DEFAULT_LIMIT: u32 = 200;

/// `kind`: `text`, `image`, `files`, or none for all.
#[tauri::command]
pub fn clip_list(state: State<'_, AppState>, limit: Option<u32>, kind: Option<String>) -> CmdResult<Vec<ClipEntry>> {
    Ok(qd_clipboard::list(&*state.db.conn()?, kind.as_deref(), limit.unwrap_or(DEFAULT_LIMIT))?)
}

#[tauri::command]
pub fn clip_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
    kind: Option<String>,
) -> CmdResult<Vec<ClipEntry>> {
    Ok(qd_clipboard::search(&*state.db.conn()?, &query, kind.as_deref(), limit.unwrap_or(DEFAULT_LIMIT))?)
}

/// Put an entry back on the clipboard; returns the keystroke that pastes it.
fn copy_entry(app: &AppHandle, state: &AppState, id: i64) -> CmdResult<Chord> {
    let payload = {
        let conn = state.db.conn()?;
        // Touch first so the watcher sees our own write as a repeat, not a new copy.
        qd_clipboard::touch(&conn, id, now_ms() as i64)?;
        qd_clipboard::payload(&conn, &state.clipboard.blobs, id)?
    };
    let fail = |e: String| CmdError::new("internal", e);
    state.clipboard.expect_own_write();
    let chord = match payload {
        Payload::Text(text) => {
            qd_platform::paste::write_text(&text).map_err(fail)?;
            Chord::ShiftInsert
        }
        Payload::Image { path, .. } => {
            let bytes =
                std::fs::read(&path).map_err(|e| CmdError::new("missing", format!("image file is gone: {e}")))?;
            qd_platform::paste::write_image(&bytes).map_err(fail)?;
            Chord::CtrlV
        }
        Payload::Files(paths) => {
            let existing: Vec<String> = paths.into_iter().filter(|p| std::path::Path::new(p).exists()).collect();
            if existing.is_empty() {
                return Err(CmdError::new("missing", "these files no longer exist"));
            }
            qd_platform::paste::write_files(&existing).map_err(fail)?;
            Chord::CtrlV
        }
    };
    let _ = app.emit("clipboard://changed", ());
    Ok(chord)
}

/// Put an entry back on the system clipboard and move it to the top.
#[tauri::command]
pub fn clip_copy(app: AppHandle, state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    copy_entry(&app, &state, id).map(drop)
}

/// Copy an entry, hide the popup and type it into the app that had focus before.
/// On failure the text is still on the clipboard.
#[tauri::command]
pub async fn clip_paste(app: AppHandle, id: i64) -> CmdResult<()> {
    let chord = copy_entry(&app, &app.state::<AppState>(), id)?;
    if let Some(w) = app.get_webview_window(crate::windows::CLIP_POPUP) {
        let _ = w.hide();
    }
    // Give the compositor time to hand focus back to the previous window.
    tokio::time::sleep(Duration::from_millis(120)).await;
    crate::clipboard::paste_into_focused(&app, chord).await.map_err(|e| {
        tracing::warn!(error = %e, "auto-paste failed");
        CmdError::new("paste_failed", e)
    })
}

#[tauri::command]
pub fn clip_set_auto_paste(app: AppHandle, enabled: bool) -> CmdResult<()> {
    Ok(crate::clipboard::set_auto_paste(&app, enabled)?)
}

#[tauri::command]
pub fn clip_pin(app: AppHandle, state: State<'_, AppState>, id: i64, pinned: bool) -> CmdResult<ClipEntry> {
    let entry = qd_clipboard::set_pinned(&*state.db.conn()?, id, pinned)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(entry)
}

#[tauri::command]
pub fn clip_delete(app: AppHandle, state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    qd_clipboard::delete(&*state.db.conn()?, &state.clipboard.blobs, id)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(())
}

#[tauri::command]
pub fn clip_clear(app: AppHandle, state: State<'_, AppState>, keep_pinned: bool) -> CmdResult<usize> {
    let n = qd_clipboard::clear(&*state.db.conn()?, &state.clipboard.blobs, keep_pinned)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(n)
}

#[tauri::command]
pub fn clip_status(state: State<'_, AppState>) -> ClipStatus {
    state.clipboard.status()
}

#[tauri::command]
pub fn clip_set_paused(app: AppHandle, paused: bool, minutes: Option<u32>) -> CmdResult<()> {
    Ok(crate::clipboard::set_paused(&app, paused, minutes)?)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteInfo {
    method: qd_platform::paste::PasteMethod,
    /// What `method` resolves to right now (auto → uinput or portal).
    effective: qd_platform::paste::PasteMethod,
    uinput_available: bool,
    /// Our udev rule is installed (so "Disable" makes sense).
    rule_installed: bool,
    /// The app can install the rule itself behind a password dialog.
    can_enable: bool,
    /// Manual fallback that grants /dev/uinput (Linux).
    setup_command: Option<&'static str>,
}

#[tauri::command]
pub fn clip_paste_info(state: State<'_, AppState>) -> PasteInfo {
    let method = state.clipboard.paste_method();
    #[cfg(target_os = "linux")]
    let (available, installed, can_enable, setup) = {
        use qd_platform::uinput;
        (uinput::available(), uinput::rule_installed(), uinput::pkexec_available(), Some(uinput::SETUP_COMMAND))
    };
    #[cfg(not(target_os = "linux"))]
    let (available, installed, can_enable, setup) = (false, false, false, None);
    PasteInfo {
        method,
        effective: qd_platform::paste::AutoPaster::resolve(method),
        uinput_available: available,
        rule_installed: installed,
        can_enable,
        setup_command: setup,
    }
}

/// Install the uinput rule behind the desktop's password dialog.
#[tauri::command]
pub async fn clip_uinput_enable() -> CmdResult<()> {
    #[cfg(target_os = "linux")]
    return super::blocking(|| qd_platform::uinput::enable().map_err(|e| CmdError::new("uinput", e))).await;
    #[cfg(not(target_os = "linux"))]
    Err(CmdError::new("unsupported", "only available on Linux"))
}

#[tauri::command]
pub async fn clip_uinput_disable() -> CmdResult<()> {
    #[cfg(target_os = "linux")]
    return super::blocking(|| qd_platform::uinput::disable().map_err(|e| CmdError::new("uinput", e))).await;
    #[cfg(not(target_os = "linux"))]
    Err(CmdError::new("unsupported", "only available on Linux"))
}

#[tauri::command]
pub fn clip_set_paste_method(app: AppHandle, method: qd_platform::paste::PasteMethod) -> CmdResult<()> {
    Ok(crate::clipboard::set_paste_method(&app, method)?)
}
