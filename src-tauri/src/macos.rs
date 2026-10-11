//! macOS window behaviour: QuickDesk is a menu bar app, and its windows come
//! to the Space you are on instead of switching you to theirs.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSWindow, NSWindowCollectionBehavior};
use tauri::{AppHandle, Runtime, WebviewWindow};

/// Popups show on every Space, also over an app in full screen; the main
/// window moves to the current Space when it is shown.
pub fn place_on_spaces<R: Runtime>(window: &WebviewWindow<R>, popup: bool) {
    let Ok(ptr) = window.ns_window() else { return };
    // SAFETY: tauri hands out this window's live NSWindow; this runs in
    // setup, on the main thread, and only changes how the window joins Spaces.
    let ns: &NSWindow = unsafe { &*ptr.cast::<NSWindow>() };
    let extra = if popup {
        NSWindowCollectionBehavior::CanJoinAllSpaces | NSWindowCollectionBehavior::FullScreenAuxiliary
    } else {
        NSWindowCollectionBehavior::MoveToActiveSpace
    };
    ns.setCollectionBehavior(ns.collectionBehavior() | extra);
}

/// Step back so macOS makes the app that was in front before the popup active
/// again, with its window and text field focused as they were. Asking for that
/// app to be activated instead is only a request since macOS 14 and may leave
/// the field without focus; stepping back is what the system honours.
pub fn give_back_focus<R: Runtime>(app: &AppHandle<R>) {
    let done = app.run_on_main_thread(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            NSApplication::sharedApplication(mtm).deactivate();
        }
    });
    if let Err(e) = done {
        tracing::warn!(error = %e, "could not step back from the front");
    }
}
