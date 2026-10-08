//! Portable fallback for Windows / macOS: poll the clipboard text.
//! TODO: native listeners (AddClipboardFormatListener, NSPasteboard
//! changeCount) and their "concealed" markers for password managers.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::{set_state, ClipEvent, WatcherState, MAX_BYTES};

const INTERVAL: Duration = Duration::from_millis(500);

pub(super) fn spawn(on_event: impl Fn(ClipEvent) + Send + 'static, state: Arc<Mutex<WatcherState>>) -> &'static str {
    thread::Builder::new()
        .name("qd-clipboard".into())
        .spawn(move || {
            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => return set_state(&state, WatcherState::Unavailable(e.to_string())),
            };
            set_state(&state, WatcherState::Running);
            // Do not record whatever was on the clipboard before we started.
            let mut last = clipboard.get_text().ok();
            loop {
                thread::sleep(INTERVAL);
                let Ok(text) = clipboard.get_text() else { continue };
                if last.as_deref() != Some(text.as_str()) {
                    last = Some(text.clone());
                    if text.len() <= MAX_BYTES && !text.trim().is_empty() {
                        on_event(ClipEvent { text, source_app: None });
                    }
                }
            }
        })
        .expect("spawn clipboard thread");
    "poll"
}
