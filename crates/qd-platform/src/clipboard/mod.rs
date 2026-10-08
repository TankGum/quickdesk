//! Background clipboard watchers. Each backend runs on its own thread and
//! reports every new text selection through a callback.

#[cfg(not(target_os = "linux"))]
mod poll;
#[cfg(target_os = "linux")]
mod x11;

use std::sync::{Arc, Mutex};

/// Largest selection we read; bigger ones are skipped.
pub const MAX_BYTES: usize = 1 << 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipEvent {
    pub text: String,
    /// Application that owns the selection, when the platform tells us.
    pub source_app: Option<String>,
}

/// Human-readable watcher state for the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherState {
    Starting,
    Running,
    /// Temporarily disconnected; the watcher retries on its own.
    Retrying(String),
    /// No way to watch the clipboard in this session.
    Unavailable(String),
}

#[derive(Clone)]
pub struct WatcherHandle {
    pub backend: &'static str,
    state: Arc<Mutex<WatcherState>>,
}

impl WatcherHandle {
    pub fn state(&self) -> WatcherState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

pub(crate) fn set_state(state: &Arc<Mutex<WatcherState>>, s: WatcherState) {
    *state.lock().unwrap_or_else(|e| e.into_inner()) = s;
}

/// Start watching. Password-manager selections are never read or reported.
pub fn spawn_watcher(on_event: impl Fn(ClipEvent) + Send + 'static) -> WatcherHandle {
    let state = Arc::new(Mutex::new(WatcherState::Starting));
    #[cfg(target_os = "linux")]
    let backend = x11::spawn(on_event, state.clone());
    #[cfg(not(target_os = "linux"))]
    let backend = poll::spawn(on_event, state.clone());
    WatcherHandle { backend, state }
}
