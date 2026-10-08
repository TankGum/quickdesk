use qd_ports::PortEntry;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use super::{blocking, CmdError, CmdResult};

#[tauri::command]
pub async fn ports_scan() -> CmdResult<Vec<PortEntry>> {
    blocking(|| Ok(qd_ports::scan()?)).await
}

#[tauri::command]
pub fn ports_kill(pid: u32, force: bool) -> CmdResult<()> {
    tracing::info!(pid, force, "kill requested");
    Ok(qd_ports::kill(pid, force)?)
}

#[tauri::command]
pub fn ports_is_alive(pid: u32) -> bool {
    qd_ports::is_alive(pid)
}

#[tauri::command]
pub async fn ports_stop_container(id: String) -> CmdResult<()> {
    tracing::info!(%id, "container stop requested");
    #[cfg(unix)]
    return blocking(move || Ok(qd_ports::docker::stop(&id)?)).await;
    #[cfg(not(unix))]
    Err(CmdError::new("unsupported", format!("cannot stop container {id} on this platform")))
}

#[tauri::command]
pub fn ports_open(app: AppHandle, port: u16) -> CmdResult<()> {
    app.opener()
        .open_url(format!("http://localhost:{port}"), None::<&str>)
        .map_err(|e| CmdError::new("internal", e.to_string()))
}
