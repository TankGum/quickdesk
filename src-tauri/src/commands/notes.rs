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
pub fn notes_create(
    app: AppHandle,
    state: State<'_, AppState>,
    title: Option<String>,
    body: String,
) -> CmdResult<Note> {
    let note = repo::create_note(&*state.db.conn()?, &state.clock, title.as_deref().unwrap_or(""), &body)?;
    changed(&app, &state);
    Ok(note)
}

#[tauri::command]
pub fn notes_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    body: Option<String>,
    pinned: Option<bool>,
) -> CmdResult<Note> {
    let patch = repo::NotePatch { title: title.as_deref(), body: body.as_deref(), pinned };
    let note = repo::update_note(&*state.db.conn()?, &state.clock, &id, &patch)?;
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
