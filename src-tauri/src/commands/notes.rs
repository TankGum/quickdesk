use qd_notes::{repo, Note};
use tauri::{AppHandle, Emitter, State};

use super::CmdResult;
use crate::state::AppState;

const LIST_LIMIT: u32 = 500;

/// Tell every window to refresh, and the sync worker that there is work.
fn changed(app: &AppHandle, state: &AppState) {
    let _ = app.emit("notes://changed", ());
    state.on_notes_changed();
}

#[tauri::command]
pub fn notes_create(app: AppHandle, state: State<'_, AppState>, body: String) -> CmdResult<Note> {
    let note = repo::create(&*state.db.conn()?, &state.clock, &body)?;
    changed(&app, &state);
    Ok(note)
}

#[tauri::command]
pub fn notes_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    body: Option<String>,
    pinned: Option<bool>,
) -> CmdResult<Note> {
    let note = repo::update(&*state.db.conn()?, &state.clock, &id, body.as_deref(), pinned)?;
    changed(&app, &state);
    Ok(note)
}

#[tauri::command]
pub fn notes_delete(app: AppHandle, state: State<'_, AppState>, id: String) -> CmdResult<()> {
    repo::delete(&*state.db.conn()?, &state.clock, &id)?;
    changed(&app, &state);
    Ok(())
}

#[tauri::command]
pub fn notes_restore(app: AppHandle, state: State<'_, AppState>, id: String) -> CmdResult<Note> {
    let note = repo::restore(&*state.db.conn()?, &state.clock, &id)?;
    changed(&app, &state);
    Ok(note)
}

#[tauri::command]
pub fn notes_list(state: State<'_, AppState>) -> CmdResult<Vec<Note>> {
    Ok(repo::list(&*state.db.conn()?, LIST_LIMIT)?)
}

#[tauri::command]
pub fn notes_search(state: State<'_, AppState>, query: String) -> CmdResult<Vec<Note>> {
    Ok(repo::search(&*state.db.conn()?, &query, LIST_LIMIT)?)
}
