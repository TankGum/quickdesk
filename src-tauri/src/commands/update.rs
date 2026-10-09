use tauri::AppHandle;

use super::{CmdError, CmdResult};
use crate::updater::{self, UpdateInfo};

#[tauri::command]
pub fn update_status(app: AppHandle) -> UpdateInfo {
    updater::info(&app)
}

/// Check now and return the result.
#[tauri::command]
pub async fn update_check(app: AppHandle) -> UpdateInfo {
    updater::check(&app).await;
    updater::info(&app)
}

/// Install the pending update; on success the app restarts and never returns.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> CmdResult<()> {
    updater::install(&app).await.map_err(|e| CmdError::new("update_failed", e))
}

#[tauri::command]
pub fn update_set_auto(app: AppHandle, enabled: bool) -> CmdResult<UpdateInfo> {
    updater::set_auto_check(&app, enabled)?;
    Ok(updater::info(&app))
}
