//! Pre-created windows that hotkeys only show/hide, never create.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, Window, WindowEvent};

use crate::cli::Target;
use crate::ipc::now_ms;

pub const MAIN: &str = "main";
pub const NOTE_POPUP: &str = "note-popup";
pub const CLIP_POPUP: &str = "clip-popup";

/// Sent to a window each time it is brought up, so the UI can focus its input
/// and report back whether it actually received keyboard focus.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub sent_at_ms: u64,
    pub shown_at_ms: u64,
    /// For `main`: which tab to open.
    pub tab: Option<&'static str>,
}

fn label_and_tab(target: Target) -> (&'static str, Option<&'static str>) {
    match target {
        Target::Notes => (NOTE_POPUP, None),
        Target::Clipboard => (CLIP_POPUP, None),
        Target::Ports => (MAIN, Some("ports")),
        Target::Main => (MAIN, None),
    }
}

pub fn show<R: Runtime>(app: &AppHandle<R>, target: Target, sent_at_ms: u64) {
    let (label, tab) = label_and_tab(target);
    let Some(w) = app.get_webview_window(label) else {
        tracing::error!(label, "window missing");
        return;
    };
    if let Err(e) = w.show().and_then(|_| w.unminimize()).and_then(|_| w.set_focus()) {
        tracing::warn!(label, error = %e, "failed to show window");
    }
    let shown = Shown { sent_at_ms, shown_at_ms: now_ms(), tab };
    if let Err(e) = app.emit_to(label, "window://shown", shown) {
        tracing::warn!(label, error = %e, "failed to emit shown");
    }
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>, target: Target, sent_at_ms: u64) {
    let (label, tab) = label_and_tab(target);
    let Some(w) = app.get_webview_window(label) else { return };
    let visible = w.is_visible().unwrap_or(false);
    let focused = w.is_focused().unwrap_or(false);
    tracing::debug!(label, visible, focused, "toggle");
    let up = visible && focused;
    // Ports lives in a tab of main: if main is up on another tab, switch instead of hiding.
    if up && tab.is_none() {
        let _ = w.hide();
    } else {
        show(app, target, sent_at_ms);
    }
}

pub fn is_popup(label: &str) -> bool {
    label == NOTE_POPUP || label == CLIP_POPUP
}

pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    match event {
        // Tray app: closing any window only hides it.
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        // Popups behave like a launcher: clicking elsewhere dismisses them.
        WindowEvent::Focused(false) if is_popup(window.label()) => {
            tracing::debug!(label = window.label(), "popup lost focus, hiding");
            let _ = window.hide();
        }
        _ => {}
    }
}
