//! macOS window behaviour: QuickDesk is a menu bar app, its windows come to
//! the Space you are on instead of switching you to theirs, and its popups
//! take the keyboard without taking the app you were typing in out of focus.

use std::sync::OnceLock;

use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, NSObjectProtocol, Sel};
use objc2::{msg_send, sel, ClassType, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSPanel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
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

/// A borderless NSPanel that can take the keyboard. The `focusable` ivar is
/// the one tao's own window class has (it reads it by name); NSPanel adds no
/// fields to NSWindow, so a tao window can take this class as it is.
fn panel_class() -> &'static AnyClass {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();
    CLASS.get_or_init(|| {
        extern "C-unwind" fn yes(_: &AnyObject, _: Sel) -> Bool {
            Bool::YES
        }
        extern "C-unwind" fn no(_: &AnyObject, _: Sel) -> Bool {
            Bool::NO
        }
        let mut builder =
            ClassBuilder::new(c"QuickDeskPanel", NSPanel::class()).expect("QuickDeskPanel is defined only once");
        // SAFETY: both selectors take no arguments and return BOOL.
        unsafe {
            builder.add_method(sel!(canBecomeKeyWindow), yes as extern "C-unwind" fn(_, _) -> _);
            builder.add_method(sel!(canBecomeMainWindow), no as extern "C-unwind" fn(_, _) -> _);
        }
        builder.add_ivar::<Bool>(c"focusable");
        builder.register()
    })
}

/// Make a popup a non-activating panel: it takes the keyboard while the app
/// you were typing in stays the active one, with its text field focused, so
/// ⌘V after the popup closes lands there. A plain window would make QuickDesk
/// the active app and that field would lose the focus.
pub fn make_panel<R: Runtime>(window: &WebviewWindow<R>) {
    let Ok(ptr) = window.ns_window() else { return };
    // SAFETY: tauri hands out this window's live NSWindow, an instance of tao's
    // NSWindow subclass; this runs in setup, on the main thread. The panel
    // class has the same instance layout (NSWindow's fields plus `focusable`).
    let ns: &NSWindow = unsafe { &*ptr.cast::<NSWindow>() };
    unsafe { AnyObject::set_class(ns, panel_class()) };
    ns.setStyleMask(ns.styleMask() | NSWindowStyleMask::NonactivatingPanel);
    // AppKit only tells the window server about a non-activating panel when it
    // is created as one: set later, a click on it still activates QuickDesk,
    // the panel loses the keyboard and hides before the click picks an entry.
    // This private setter is what the style mask would have set.
    let prevents = sel!(_setPreventsActivation:);
    // SAFETY: called only where NSWindow has it; takes one BOOL, returns void.
    if ns.respondsToSelector(prevents) {
        let () = unsafe { msg_send![ns, _setPreventsActivation: Bool::YES] };
        // SAFETY: the matching getter, no arguments, returns BOOL.
        let now: Bool = unsafe { msg_send![ns, _preventsActivation] };
        tracing::debug!(prevents_activation = now.as_bool(), "popup is a non-activating panel");
    } else {
        tracing::warn!("no _setPreventsActivation:, clicks on popups may activate QuickDesk");
    }
    // SAFETY: the window is an NSPanel now.
    let panel: &NSPanel = unsafe { &*ptr.cast::<NSPanel>() };
    panel.setBecomesKeyOnlyIfNeeded(false);
    // Panels hide when their app is deactivated; QuickDesk is never active.
    ns.setHidesOnDeactivate(false);
}

/// Bring a popup up with the keyboard in its page, without activating QuickDesk
/// (`set_focus` would: it asks macOS to make QuickDesk the active app).
pub fn show_panel<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.show()?;
    window.as_ref().set_focus()
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
