use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};

use crate::cli::Target;
use crate::ipc::now_ms;
use crate::state::AppState;
use crate::windows;

const PAUSE_CLIP: &str = "pause_clipboard";

pub fn build(app: &App, clip_paused: bool) -> tauri::Result<()> {
    let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
    let pause = CheckMenuItem::with_id(app, PAUSE_CLIP, "Pause clipboard history", true, clip_paused, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &item("quick-note", "Quick note")?,
            &item("notes", "Notes")?,
            &item("clipboard", "Clipboard")?,
            &item("ports", "Ports")?,
            &PredefinedMenuItem::separator(app)?,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &item("main", "Open QuickDesk")?,
            &item("quit", "Quit")?,
        ],
    )?;
    *app.state::<AppState>().clipboard.tray_item.lock().unwrap_or_else(|e| e.into_inner()) = Some(pause);

    let mut tray = TrayIconBuilder::with_id("quickdesk")
        .tooltip("QuickDesk")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let target = match event.id().as_ref() {
                "quick-note" => Target::QuickNote,
                "notes" => Target::Notes,
                "clipboard" => Target::Clipboard,
                "ports" => Target::Ports,
                "main" => Target::Main,
                PAUSE_CLIP => {
                    let paused = !app.state::<AppState>().clipboard.is_paused();
                    if let Err(e) = crate::clipboard::set_paused(app, paused) {
                        tracing::error!(error = %e, "failed to toggle clipboard pause");
                    }
                    return;
                }
                "quit" => return crate::quit(app),
                _ => return,
            };
            windows::show(app, target, now_ms());
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
