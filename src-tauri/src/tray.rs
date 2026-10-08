use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager, Wry};

use crate::cli::Target;
use crate::i18n::{tr, Lang};
use crate::ipc::now_ms;
use crate::state::AppState;
use crate::windows;

const PAUSE_CLIP: &str = "pause_clipboard";

const TRAY_ID: &str = "quickdesk";

/// The tray menu in `lang`; also remembers the pause checkbox for later updates.
fn menu<M: Manager<Wry>>(m: &M, lang: Lang, clip_paused: bool) -> tauri::Result<Menu<Wry>> {
    let item = |id: &str| MenuItem::with_id(m, id, tr(lang, id), true, None::<&str>);
    let pause = CheckMenuItem::with_id(m, PAUSE_CLIP, pause_label(lang, clip_paused), true, clip_paused, None::<&str>)?;
    let menu = Menu::with_items(
        m,
        &[
            &item("quick-note")?,
            &item("notes")?,
            &item("clipboard")?,
            &item("ports")?,
            &PredefinedMenuItem::separator(m)?,
            &pause,
            &PredefinedMenuItem::separator(m)?,
            &item("main")?,
            &item("quit")?,
        ],
    )?;
    *m.state::<AppState>().clipboard.tray_item.lock().unwrap_or_else(|e| e.into_inner()) = Some(pause);
    Ok(menu)
}

/// Label that says what clicking the pause item will do.
pub fn pause_label(lang: Lang, paused: bool) -> &'static str {
    tr(lang, if paused { "paused" } else { "pause" })
}

/// Rebuild the menu after a language change.
pub fn relabel(app: &AppHandle) {
    crate::ai_usage::relabel(app);
    let state = app.state::<AppState>();
    let (lang, paused) = (state.lang(), state.clipboard.is_paused());
    match (app.tray_by_id(TRAY_ID), menu(app, lang, paused)) {
        (Some(tray), Ok(m)) => {
            let _ = tray.set_menu(Some(m));
        }
        (_, Err(e)) => tracing::warn!(error = %e, "failed to rebuild tray menu"),
        _ => {}
    }
}

pub fn build(app: &App, clip_paused: bool) -> tauri::Result<()> {
    let menu = menu(app, app.state::<AppState>().lang(), clip_paused)?;

    let mut tray =
        TrayIconBuilder::with_id(TRAY_ID).tooltip("QuickDesk").menu(&menu).show_menu_on_left_click(true).on_menu_event(
            |app, event| {
                let target = match event.id().as_ref() {
                    "quick-note" => Target::QuickNote,
                    "notes" => Target::Notes,
                    "clipboard" => Target::Clipboard,
                    "ports" => Target::Ports,
                    "main" => Target::Main,
                    PAUSE_CLIP => {
                        let paused = !app.state::<AppState>().clipboard.is_paused();
                        if let Err(e) = crate::clipboard::set_paused(app, paused, None) {
                            tracing::error!(error = %e, "failed to toggle clipboard pause");
                        }
                        return;
                    }
                    "quit" => return crate::quit(app),
                    _ => return,
                };
                windows::show(app, target, now_ms());
            },
        );
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
