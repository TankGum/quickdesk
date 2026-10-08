use qd_core::settings;
use qd_platform::Accelerator;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::{blocking, CmdError, CmdResult};
use crate::cli::Target;
use crate::hotkeys::{self, HotkeyConfig, SETTINGS_KEY};
use crate::state::AppState;

fn target(id: &str) -> CmdResult<Target> {
    Target::parse(id)
        .filter(|t| Target::HOTKEY_TARGETS.contains(t))
        .ok_or_else(|| CmdError::new("invalid", format!("unknown hotkey target {id:?}")))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyInfo {
    config: HotkeyConfig,
    strategy: String,
}

#[tauri::command]
pub fn hotkeys_get(state: State<'_, AppState>) -> HotkeyInfo {
    HotkeyInfo {
        config: state.hotkeys.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        strategy: format!("{:?}", state.strategy),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyCheck {
    /// Canonical spelling, e.g. `Super+Alt+N`.
    normalized: String,
    /// Validation error; the shortcut cannot be saved.
    error: Option<String>,
    /// Desktop shortcuts already using it (saving is allowed but it may not fire).
    conflicts: Vec<String>,
    /// Input methods (IBus Unikey/Bamboo) may swallow it inside text fields.
    input_method_warning: bool,
}

#[tauri::command]
pub async fn hotkeys_check(app: AppHandle, target: String, accel: String) -> CmdResult<HotkeyCheck> {
    blocking(move || check(&app.state::<AppState>(), &target, accel)).await
}

fn check(state: &AppState, target: &str, accel: String) -> CmdResult<HotkeyCheck> {
    let t = self::target(target)?;
    let parsed: Accelerator = match accel.parse() {
        Ok(a) => a,
        Err(e) => {
            return Ok(HotkeyCheck {
                normalized: accel,
                error: Some(e.to_string()),
                conflicts: vec![],
                input_method_warning: false,
            })
        }
    };
    let current = state.hotkeys.lock().unwrap_or_else(|e| e.into_inner()).clone();
    Ok(HotkeyCheck {
        normalized: parsed.to_string(),
        error: current.with(t, &accel).err(),
        conflicts: hotkeys::conflicts(state.strategy, &parsed),
        input_method_warning: parsed.may_be_eaten_by_input_method(),
    })
}

fn apply(app: &AppHandle, state: &AppState, next: HotkeyConfig) -> CmdResult<HotkeyConfig> {
    hotkeys::register(app, state.strategy, &next).map_err(|e| CmdError::new("hotkey", e))?;
    settings::set(&*state.db.conn()?, SETTINGS_KEY, &next)?;
    *state.hotkeys.lock().unwrap_or_else(|e| e.into_inner()) = next.clone();
    tracing::info!(?next, "hotkeys updated");
    Ok(next)
}

/// Bind `target` to `accel` (empty string unbinds) and re-register everything.
#[tauri::command]
pub fn hotkeys_set(
    app: AppHandle,
    state: State<'_, AppState>,
    target: String,
    accel: String,
) -> CmdResult<HotkeyConfig> {
    let t = self::target(&target)?;
    let next = state
        .hotkeys
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .with(t, &accel)
        .map_err(|e| CmdError::new("invalid", e))?;
    apply(&app, &state, next)
}

#[tauri::command]
pub fn hotkeys_reset(app: AppHandle, state: State<'_, AppState>) -> CmdResult<HotkeyConfig> {
    apply(&app, &state, HotkeyConfig::default())
}

/// Temporarily remove our global shortcuts so the settings screen can record
/// a key combination (including the one currently bound) without triggering it.
#[tauri::command]
pub async fn hotkeys_suspend(app: AppHandle) -> CmdResult<()> {
    blocking(move || {
        hotkeys::unregister(&app, app.state::<AppState>().strategy);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn hotkeys_resume(app: AppHandle) -> CmdResult<()> {
    blocking(move || {
        let state = app.state::<AppState>();
        let cfg = state.hotkeys.lock().unwrap_or_else(|e| e.into_inner()).clone();
        hotkeys::register(&app, state.strategy, &cfg).map_err(|e| CmdError::new("hotkey", e))
    })
    .await
}
