# Spike: Wayland clipboard & hotkey (2026-10-07)

Environment: Ubuntu 24.04.4, GNOME Shell 46.0, Wayland session, Xwayland on `DISPLAY=:0`.
Scripts: `clip_watch.py`, `popup_toggle.py` (GTK3 via PyGObject — same toolkit Tauri uses on Linux).

## Results

| Question | Result |
|---|---|
| `org.freedesktop.portal.GlobalShortcuts` available? | **No** (arrives in GNOME 48) |
| Hidden **Wayland-native** process sees clipboard changes? | **No** — 0 events, 0 successful polls |
| Hidden **XWayland (X11)** process sees clipboard changes? | **Yes** — captured copies from terminal, browser, GNOME app, VS Code within <1s |
| X11 `owner-change` noise | One copy can fire a burst of ~13 events → must debounce + dedup by content hash |
| GNOME custom shortcut → command → IPC → show pre-created window | Works; show cost is negligible (logged ~300ms = the 300ms report timer) |
| Popup gets keyboard focus — **Wayland** window | **2/2 yes**, even with no activation token |
| Popup gets keyboard focus — **X11/XWayland** window | **1/2** — 2nd time focus-stealing prevention denied it |
| Activation token passed to shortcut command? | **No** — `XDG_ACTIVATION_TOKEN` and `DESKTOP_STARTUP_ID` both unset |

## Decisions for Linux (GNOME Wayland)

1. **UI runs Wayland-native** (default GDK backend). Do *not* force `GDK_BACKEND=x11` — popup focus becomes unreliable.
2. **Clipboard watcher uses an X11 connection to Xwayland** from a background thread in the same Rust process
   (`x11rb` + XFixes `SelectSelectionInput` on `CLIPBOARD`). A Wayland-native GTK process may hold an
   independent X11 connection; Mutter bridges the Wayland clipboard to X selections.
   Writing back to the clipboard happens while our popup is focused, via the normal Wayland path.
3. **Watcher must debounce (~150ms) and dedup by hash** of the latest entry.
4. **Global hotkey on GNOME < 48**: register a GNOME custom keybinding (gsettings) that runs
   `quickdesk toggle <module>`; `tauri-plugin-single-instance` forwards argv to the running app.
   Use `tauri-plugin-global-shortcut` on Windows, macOS and X11 sessions; use the portal on GNOME ≥ 48 / KDE.
5. **Auto-paste into the previous app is out of scope** on Wayland: select → write clipboard → hide; user pastes.

## Still unverified (check in Milestone 1 with real Tauri)

- WebKitGTK webview inside the Tauri window keeps the same focus behaviour as a plain GTK window.
- Window placement: Wayland ignores client positioning; Mutter decides where the popup appears.
- Xwayland may be started on demand; confirm the watcher reconnects if Xwayland restarts.
