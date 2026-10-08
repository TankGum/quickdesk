use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ipc::now_ms;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    os: String,
    wayland: bool,
    desktop: String,
    hotkey_strategy: String,
    hotkeys: crate::hotkeys::HotkeyConfig,
    device_id: String,
    data_dir: String,
}

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        os: state.session.os.into(),
        wayland: state.session.wayland,
        desktop: state.session.desktop.clone(),
        hotkey_strategy: format!("{:?}", state.strategy),
        hotkeys: state.hotkeys.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        device_id: state.device_id.clone(),
        data_dir: state.data_dir.display().to_string(),
    }
}

/// What a window observed right after `window://shown`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusReport {
    label: String,
    document_focused: bool,
    input_focused: bool,
    sent_at_ms: u64,
    shown_at_ms: u64,
    #[serde(default)]
    reported_at_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStats {
    total: usize,
    focused: usize,
    /// Hotkey press (client process start) → input focused, in ms.
    latency_p50: Option<u64>,
    latency_max: Option<u64>,
    recent: Vec<FocusReport>,
}

#[tauri::command]
pub fn diag_focus_report(state: State<'_, AppState>, mut report: FocusReport) {
    report.reported_at_ms = now_ms();
    tracing::info!(
        label = %report.label,
        document_focused = report.document_focused,
        input_focused = report.input_focused,
        latency_ms = report.reported_at_ms.saturating_sub(report.sent_at_ms),
        "focus report"
    );
    let mut reports = state.focus_reports.lock().unwrap_or_else(|e| e.into_inner());
    reports.push(report);
}

#[tauri::command]
pub fn diag_focus_stats(state: State<'_, AppState>) -> FocusStats {
    let reports = state.focus_reports.lock().unwrap_or_else(|e| e.into_inner());
    let popups: Vec<&FocusReport> = reports.iter().filter(|r| r.label != crate::windows::MAIN).collect();
    let mut latencies: Vec<u64> = popups.iter().map(|r| r.reported_at_ms.saturating_sub(r.sent_at_ms)).collect();
    latencies.sort_unstable();
    FocusStats {
        total: popups.len(),
        focused: popups.iter().filter(|r| r.document_focused && r.input_focused).count(),
        latency_p50: latencies.get(latencies.len() / 2).copied(),
        latency_max: latencies.last().copied(),
        recent: popups.iter().rev().take(10).map(|r| (*r).clone()).collect(),
    }
}

#[tauri::command]
pub fn app_quit(app: AppHandle) {
    crate::quit(&app);
}

#[tauri::command]
pub fn app_show(app: AppHandle, target: String) -> super::CmdResult<()> {
    let target = crate::cli::Target::parse(&target)
        .ok_or_else(|| super::CmdError::new("invalid", format!("unknown target {target:?}")))?;
    crate::windows::show(&app, target, now_ms());
    Ok(())
}

#[tauri::command]
pub fn clipboard_write(app: AppHandle, text: String) -> super::CmdResult<()> {
    let _ = app;
    qd_platform::paste::write_text(&text).map_err(|e| super::CmdError::new("internal", e))
}

const ONBOARDING_KEY: &str = "onboarding.done";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Onboarding {
    done: bool,
    /// Offer the one-time "instant auto-paste" opt-in.
    offer_uinput: bool,
}

#[tauri::command]
pub fn app_onboarding(state: State<'_, AppState>) -> super::CmdResult<Onboarding> {
    let done = qd_core::settings::get(&*state.db.conn()?, ONBOARDING_KEY)?.unwrap_or(false);
    #[cfg(target_os = "linux")]
    let offer_uinput = !qd_platform::uinput::available() && qd_platform::uinput::pkexec_available();
    #[cfg(not(target_os = "linux"))]
    let offer_uinput = false;
    Ok(Onboarding { done, offer_uinput })
}

#[tauri::command]
pub fn app_onboarding_finish(state: State<'_, AppState>) -> super::CmdResult<()> {
    Ok(qd_core::settings::set(&*state.db.conn()?, ONBOARDING_KEY, &true)?)
}
