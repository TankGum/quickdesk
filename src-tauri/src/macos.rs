//! macOS window behaviour: QuickDesk is a menu bar app, and its windows come
//! to the Space you are on instead of switching you to theirs.

use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
use tauri::{Runtime, WebviewWindow};

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
