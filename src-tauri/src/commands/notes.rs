use std::path::PathBuf;
use std::time::Duration;

use qd_notes::{images, repo, Note};
use tauri::{AppHandle, Emitter, Manager, State};

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

/// Where the pictures of notes are kept.
pub fn image_dir(state: &AppState) -> PathBuf {
    state.data_dir.join("note-images")
}

/// A picture for a note, sent as base64 (read from a file, a paste or a drop):
/// it is shrunk and stored, and its id goes into the note as `qdimg:<id>`.
#[tauri::command]
pub async fn note_image_add(state: State<'_, AppState>, data: String) -> CmdResult<String> {
    let dir = image_dir(&state);
    let hash = tauri::async_runtime::spawn_blocking(move || images::add_base64(&dir, &data))
        .await
        .map_err(|e| super::CmdError::new("internal", e.to_string()))??;
    Ok(hash)
}

/// The picture on the system clipboard, stored like `note_image_add` does; `None` when it holds no picture.
/// The webview cannot see a pasted screenshot on every system, so the editor asks for it here.
#[tauri::command]
pub async fn note_image_paste(state: State<'_, AppState>) -> CmdResult<Option<String>> {
    let dir = image_dir(&state);
    let hash = tauri::async_runtime::spawn_blocking(move || -> CmdResult<Option<String>> {
        let Some(png) = qd_platform::paste::read_image().map_err(|e| super::CmdError::new("internal", e))? else {
            return Ok(None);
        };
        let pic = images::process(&png)?;
        images::store(&dir, &pic)?;
        Ok(Some(pic.hash))
    })
    .await
    .map_err(|e| super::CmdError::new("internal", e.to_string()))??;
    Ok(hash)
}

/// The picture as a data URL, or `None` while it is not on this computer.
#[tauri::command]
pub fn note_image(state: State<'_, AppState>, hash: String) -> CmdResult<Option<String>> {
    Ok(images::read_data_url(&image_dir(&state), &hash)?)
}

/// Pictures no note points to are deleted, a minute after start and then daily.
/// A picture younger than a day stays: it may belong to a note being written.
pub fn start_image_cleanup(app: &AppHandle) {
    let handle = app.clone();
    std::thread::Builder::new()
        .name("qd-note-images".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(60));
            loop {
                let state = handle.state::<AppState>();
                let result = state
                    .db
                    .conn()
                    .map_err(qd_notes::Error::from)
                    .and_then(|conn| images::clean(&conn, &image_dir(&state), Duration::from_secs(24 * 60 * 60)));
                match result {
                    Ok(0) => {}
                    Ok(n) => tracing::info!(removed = n, "unused note pictures deleted"),
                    Err(e) => tracing::warn!(error = %e, "cleaning note pictures failed"),
                }
                std::thread::sleep(Duration::from_secs(24 * 60 * 60));
            }
        })
        .ok();
}
