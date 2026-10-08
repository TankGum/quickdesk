use qd_clipboard::ClipEntry;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{CmdError, CmdResult};
use crate::clipboard::ClipStatus;
use crate::ipc::now_ms;
use crate::state::AppState;

const DEFAULT_LIMIT: u32 = 200;

#[tauri::command]
pub fn clip_list(state: State<'_, AppState>, limit: Option<u32>) -> CmdResult<Vec<ClipEntry>> {
    Ok(qd_clipboard::list(&*state.db.conn()?, limit.unwrap_or(DEFAULT_LIMIT))?)
}

#[tauri::command]
pub fn clip_search(state: State<'_, AppState>, query: String, limit: Option<u32>) -> CmdResult<Vec<ClipEntry>> {
    Ok(qd_clipboard::search(&*state.db.conn()?, &query, limit.unwrap_or(DEFAULT_LIMIT))?)
}

/// Put an entry back on the system clipboard and move it to the top.
#[tauri::command]
pub fn clip_copy(app: AppHandle, state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    let text = {
        let conn = state.db.conn()?;
        // Touch first so the watcher sees our own write as a repeat, not a new copy.
        qd_clipboard::touch(&conn, id, now_ms() as i64)?;
        qd_clipboard::content(&conn, id)?
    };
    app.clipboard().write_text(text).map_err(|e| CmdError::new("internal", e.to_string()))?;
    let _ = app.emit("clipboard://changed", ());
    Ok(())
}

#[tauri::command]
pub fn clip_pin(app: AppHandle, state: State<'_, AppState>, id: i64, pinned: bool) -> CmdResult<ClipEntry> {
    let entry = qd_clipboard::set_pinned(&*state.db.conn()?, id, pinned)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(entry)
}

#[tauri::command]
pub fn clip_delete(app: AppHandle, state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    qd_clipboard::delete(&*state.db.conn()?, id)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(())
}

#[tauri::command]
pub fn clip_clear(app: AppHandle, state: State<'_, AppState>, keep_pinned: bool) -> CmdResult<usize> {
    let n = qd_clipboard::clear(&*state.db.conn()?, keep_pinned)?;
    let _ = app.emit("clipboard://changed", ());
    Ok(n)
}

#[tauri::command]
pub fn clip_status(state: State<'_, AppState>) -> ClipStatus {
    state.clipboard.status()
}

#[tauri::command]
pub fn clip_set_paused(app: AppHandle, paused: bool) -> CmdResult<()> {
    Ok(crate::clipboard::set_paused(&app, paused)?)
}
