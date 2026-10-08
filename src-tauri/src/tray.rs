use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Runtime};

use crate::cli::Target;
use crate::ipc::now_ms;
use crate::windows;

pub fn build<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
    let menu = Menu::with_items(
        app,
        &[
            &item("notes", "Quick note")?,
            &item("clipboard", "Clipboard")?,
            &item("ports", "Ports")?,
            &PredefinedMenuItem::separator(app)?,
            &item("main", "Open QuickDesk")?,
            &item("quit", "Quit")?,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("quickdesk")
        .tooltip("QuickDesk")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let target = match event.id().as_ref() {
                "notes" => Target::Notes,
                "clipboard" => Target::Clipboard,
                "ports" => Target::Ports,
                "main" => Target::Main,
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
