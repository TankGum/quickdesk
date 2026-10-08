use tauri::{AppHandle, Manager, State};

use super::{blocking, CmdResult};
use crate::ai_usage::{self, RingChoice, UsageSnapshot};
use crate::state::AppState;

#[tauri::command]
pub fn ai_usage_get(state: State<'_, AppState>) -> UsageSnapshot {
    state.ai.snapshot()
}

/// Refresh now and return the fresh numbers.
#[tauri::command]
pub async fn ai_usage_refresh(app: AppHandle) -> CmdResult<UsageSnapshot> {
    blocking(move || {
        ai_usage::refresh(&app);
        Ok(app.state::<AppState>().ai.snapshot())
    })
    .await
}

/// Show or hide the usage ring in the top bar.
#[tauri::command]
pub fn ai_set_tray(app: AppHandle, enabled: bool) -> CmdResult<()> {
    Ok(ai_usage::set_tray_enabled(&app, enabled)?)
}

/// Pick which limit the top-bar ring shows (`None` = automatic).
#[tauri::command]
pub fn ai_set_ring(app: AppHandle, choice: Option<RingChoice>) -> CmdResult<()> {
    Ok(ai_usage::set_ring(&app, choice)?)
}
