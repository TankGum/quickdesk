//! Registers global hotkeys with whichever mechanism the session supports.

use qd_platform::gnome::{Binding, GnomeKeybindings};
use qd_platform::{Accelerator, HotkeyStrategy};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::cli::Target;

pub const SETTINGS_KEY: &str = "hotkeys";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub notes: String,
    pub clipboard: String,
    pub ports: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        HotkeyConfig { notes: "Super+Shift+N".into(), clipboard: "Super+Shift+V".into(), ports: "Super+Shift+P".into() }
    }
}

impl HotkeyConfig {
    fn get(&self, target: Target) -> &str {
        match target {
            Target::Notes => &self.notes,
            Target::Clipboard => &self.clipboard,
            Target::Ports | Target::Main => &self.ports,
        }
    }

    fn parsed(&self) -> Result<Vec<(Target, Accelerator)>, String> {
        Target::HOTKEY_TARGETS
            .iter()
            .map(|&t| self.get(t).parse().map(|a| (t, a)).map_err(|e| format!("{e}")))
            .collect()
    }
}

pub fn register<R: Runtime>(app: &AppHandle<R>, strategy: HotkeyStrategy, cfg: &HotkeyConfig) -> Result<(), String> {
    let bindings = cfg.parsed()?;
    match strategy {
        HotkeyStrategy::GnomeKeybinding => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let bindings: Vec<Binding> = bindings
                .into_iter()
                .map(|(t, accel)| Binding {
                    id: t.id().into(),
                    name: format!("QuickDesk: {}", t.id()),
                    argv: vec![exe.to_string_lossy().into_owned(), "toggle".into(), t.id().into()],
                    accel,
                })
                .collect();
            GnomeKeybindings::new().apply(&bindings).map_err(|e| e.to_string())
        }
        HotkeyStrategy::Plugin => plugin::register(app, bindings),
        HotkeyStrategy::Manual => {
            tracing::warn!("no global hotkey mechanism for this session; bind `quickdesk toggle <target>` manually");
            Ok(())
        }
    }
}

pub fn unregister<R: Runtime>(app: &AppHandle<R>, strategy: HotkeyStrategy) {
    let result = match strategy {
        HotkeyStrategy::GnomeKeybinding => GnomeKeybindings::new().remove_all().map_err(|e| e.to_string()),
        HotkeyStrategy::Plugin => plugin::unregister(app),
        HotkeyStrategy::Manual => Ok(()),
    };
    if let Err(e) = result {
        tracing::warn!(error = %e, "failed to unregister hotkeys");
    }
}

mod plugin {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    use super::*;
    use crate::ipc::now_ms;
    use crate::windows;

    pub fn register<R: Runtime>(app: &AppHandle<R>, bindings: Vec<(Target, Accelerator)>) -> Result<(), String> {
        let gs = app.global_shortcut();
        gs.unregister_all().map_err(|e| e.to_string())?;
        for (target, accel) in bindings {
            gs.on_shortcut(accel.to_string().as_str(), move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    windows::toggle(app, target, now_ms());
                }
            })
            .map_err(|e| format!("{accel}: {e}"))?;
        }
        Ok(())
    }

    pub fn unregister<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        app.global_shortcut().unregister_all().map_err(|e| e.to_string())
    }
}
