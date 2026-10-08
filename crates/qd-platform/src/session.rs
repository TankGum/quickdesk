use std::env;

/// What we know about the desktop session we are running in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub os: &'static str,
    pub wayland: bool,
    /// Lower-cased `XDG_CURRENT_DESKTOP`, e.g. `ubuntu:gnome`.
    pub desktop: String,
}

/// How global hotkeys are delivered in this session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyStrategy {
    /// OS-level grab via `tauri-plugin-global-shortcut` (Windows, macOS, X11).
    Plugin,
    /// GNOME custom keybinding that runs `quickdesk toggle <target>`.
    GnomeKeybinding,
    /// No working mechanism; user must bind `quickdesk toggle ...` manually.
    Manual,
}

impl Session {
    pub fn detect() -> Self {
        Self::from_vars(
            env::consts::OS,
            env::var("XDG_SESSION_TYPE").ok().as_deref(),
            env::var("WAYLAND_DISPLAY").ok().as_deref(),
            env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
        )
    }

    pub fn from_vars(
        os: &'static str,
        session_type: Option<&str>,
        wayland_display: Option<&str>,
        current_desktop: Option<&str>,
    ) -> Self {
        let wayland = os == "linux"
            && (session_type == Some("wayland") || wayland_display.is_some_and(|d| !d.is_empty()));
        Session { os, wayland, desktop: current_desktop.unwrap_or_default().to_lowercase() }
    }

    pub fn is_gnome(&self) -> bool {
        self.desktop.split(':').any(|d| d == "gnome")
    }

    pub fn hotkey_strategy(&self) -> HotkeyStrategy {
        match (self.wayland, self.is_gnome()) {
            (false, _) => HotkeyStrategy::Plugin,
            (true, true) => HotkeyStrategy::GnomeKeybinding,
            // KDE / wlroots: GlobalShortcuts portal backend comes later.
            (true, false) => HotkeyStrategy::Manual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubuntu_gnome_wayland_uses_gnome_keybindings() {
        let s = Session::from_vars("linux", Some("wayland"), Some("wayland-0"), Some("ubuntu:GNOME"));
        assert!(s.wayland && s.is_gnome());
        assert_eq!(s.hotkey_strategy(), HotkeyStrategy::GnomeKeybinding);
    }

    #[test]
    fn x11_and_other_os_use_plugin() {
        let x11 = Session::from_vars("linux", Some("x11"), None, Some("ubuntu:GNOME"));
        assert_eq!(x11.hotkey_strategy(), HotkeyStrategy::Plugin);
        let win = Session::from_vars("windows", None, None, None);
        assert_eq!(win.hotkey_strategy(), HotkeyStrategy::Plugin);
    }

    #[test]
    fn kde_wayland_is_manual_for_now() {
        let s = Session::from_vars("linux", Some("wayland"), None, Some("KDE"));
        assert_eq!(s.hotkey_strategy(), HotkeyStrategy::Manual);
    }
}
