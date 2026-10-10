//! Registers global hotkeys with whichever mechanism the session supports.

use qd_platform::gnome::{Binding, GnomeKeybindings};
use qd_platform::{Accelerator, HotkeyStrategy};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::cli::Target;

pub const SETTINGS_KEY: &str = "hotkeys";

/// Accelerators per target; an empty string means "no hotkey".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub notes: String,
    #[serde(default)]
    pub quick_note: String,
    pub clipboard: String,
    pub ports: String,
}

impl Default for HotkeyConfig {
    /// Super+Alt rather than Super+Shift: input methods like IBus Unikey
    /// swallow Super+Shift+<letter> while a text field has focus.
    fn default() -> Self {
        HotkeyConfig {
            notes: "Super+Alt+N".into(),
            quick_note: String::new(),
            clipboard: "Super+Alt+V".into(),
            ports: "Super+Alt+P".into(),
        }
    }
}

impl HotkeyConfig {
    /// Defaults of the first release, replaced automatically by the new ones.
    fn is_legacy_default(&self) -> bool {
        self.notes == "Super+Shift+N" && self.clipboard == "Super+Shift+V" && self.ports == "Super+Shift+P"
    }

    /// Load from settings, upgrading untouched legacy defaults.
    pub fn load(conn: &rusqlite::Connection) -> qd_core::Result<Self> {
        let cfg: HotkeyConfig = qd_core::settings::get_or_init(conn, SETTINGS_KEY, HotkeyConfig::default)?;
        if cfg.is_legacy_default() {
            let upgraded = HotkeyConfig { quick_note: cfg.quick_note, ..HotkeyConfig::default() };
            qd_core::settings::set(conn, SETTINGS_KEY, &upgraded)?;
            tracing::info!("hotkeys upgraded to Super+Alt defaults");
            return Ok(upgraded);
        }
        Ok(cfg)
    }

    pub fn get(&self, target: Target) -> &str {
        match target {
            Target::Notes => &self.notes,
            Target::QuickNote => &self.quick_note,
            Target::Clipboard => &self.clipboard,
            Target::Ports | Target::Ai | Target::Versions | Target::Main => &self.ports,
        }
    }

    fn slot(&mut self, target: Target) -> &mut String {
        match target {
            Target::Notes => &mut self.notes,
            Target::QuickNote => &mut self.quick_note,
            Target::Clipboard => &mut self.clipboard,
            Target::Ports | Target::Ai | Target::Versions | Target::Main => &mut self.ports,
        }
    }

    /// Return a copy with `target` bound to `accel` (or unbound when empty),
    /// validated and normalized.
    pub fn with(&self, target: Target, accel: &str) -> Result<Self, String> {
        let mut next = self.clone();
        if accel.trim().is_empty() {
            next.slot(target).clear();
            return Ok(next);
        }
        let parsed: Accelerator = accel.parse().map_err(|e| format!("{e}"))?;
        parsed.validate_global()?;
        let normalized = parsed.to_string();
        if let Some(other) = Target::HOTKEY_TARGETS.iter().find(|&&t| t != target && self.get(t) == normalized) {
            return Err(format!("{normalized} is already used for {}", other.id()));
        }
        *next.slot(target) = normalized;
        Ok(next)
    }

    fn parsed(&self) -> Result<Vec<(Target, Accelerator)>, String> {
        Target::HOTKEY_TARGETS
            .iter()
            .filter(|&&t| !self.get(t).is_empty())
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

/// Existing desktop shortcuts that already use `accel`.
pub fn conflicts(strategy: HotkeyStrategy, accel: &Accelerator) -> Vec<String> {
    match strategy {
        HotkeyStrategy::GnomeKeybinding => GnomeKeybindings::new().conflicts(accel),
        // The OS reports a clash when registration fails; nothing to pre-check.
        HotkeyStrategy::Plugin | HotkeyStrategy::Manual => Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_validates_normalizes_and_rejects_duplicates() {
        let cfg = HotkeyConfig::default();
        let next = cfg.with(Target::QuickNote, "alt + super + q").unwrap();
        assert_eq!(next.quick_note, "Super+Alt+Q");
        assert!(cfg.with(Target::Notes, "Shift+N").is_err(), "needs Ctrl/Alt/Super");
        assert!(cfg.with(Target::Notes, "Super+Alt+?").is_err());
        let dup = cfg.with(Target::Notes, "Super+Alt+V").unwrap_err();
        assert!(dup.contains("clipboard"), "{dup}");
        assert_eq!(cfg.with(Target::Notes, "Super+Alt+N").unwrap(), cfg, "rebinding to itself is fine");
        assert_eq!(cfg.with(Target::Clipboard, "").unwrap().clipboard, "", "empty unbinds");
    }

    #[test]
    fn unbound_targets_are_not_registered() {
        let ids: Vec<&str> = HotkeyConfig::default().parsed().unwrap().into_iter().map(|(t, _)| t.id()).collect();
        assert_eq!(ids, vec!["notes", "clipboard", "ports"]);
    }

    #[test]
    fn legacy_defaults_are_upgraded_but_custom_choices_kept() {
        let db = qd_core::Db::open_in_memory(&[]).unwrap();
        let conn = db.conn().unwrap();
        let legacy = HotkeyConfig {
            notes: "Super+Shift+N".into(),
            quick_note: String::new(),
            clipboard: "Super+Shift+V".into(),
            ports: "Super+Shift+P".into(),
        };
        qd_core::settings::set(&conn, SETTINGS_KEY, &legacy).unwrap();
        assert_eq!(HotkeyConfig::load(&conn).unwrap(), HotkeyConfig::default());

        let custom = HotkeyConfig { notes: "Ctrl+Alt+N".into(), ..legacy };
        qd_core::settings::set(&conn, SETTINGS_KEY, &custom).unwrap();
        assert_eq!(HotkeyConfig::load(&conn).unwrap(), custom);
    }
}
