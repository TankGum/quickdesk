//! Background clipboard watchers. Each backend runs on its own thread and
//! reports every new selection (text, image or copied files) through a callback.

// Windows images arrive as DIBs; the conversion is plain code, tested everywhere.
#[cfg_attr(not(windows), allow(dead_code))]
mod dib;
#[cfg(not(any(target_os = "linux", windows)))]
mod poll;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;

use std::sync::{Arc, Mutex};

/// Largest text selection we read; bigger ones are skipped.
pub const MAX_TEXT_BYTES: usize = 1 << 20;
/// Largest image we read.
pub const MAX_IMAGE_BYTES: usize = 10 << 20;
/// Back-compat name for the text limit.
pub const MAX_BYTES: usize = MAX_TEXT_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipContent {
    Text(String),
    /// Encoded image as offered by the owner (`image/png`, `image/jpeg`, …).
    Image {
        mime: String,
        bytes: Vec<u8>,
    },
    /// Copied files (e.g. in a file manager), as absolute paths.
    Files(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipEvent {
    pub content: ClipContent,
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
    #[cfg(windows)]
    let backend = windows::spawn(on_event, state.clone());
    #[cfg(not(any(target_os = "linux", windows)))]
    let backend = poll::spawn(on_event, state.clone());
    WatcherHandle { backend, state }
}

/// `file:///home/me/a%20b.txt` → `/home/me/a b.txt`; `None` for non-file URIs.
pub fn file_uri_to_path(uri: &str) -> Option<String> {
    let rest = uri.trim().strip_prefix("file://")?;
    // Optional host part: file://localhost/path
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    if !path.starts_with('/') {
        return None;
    }
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

/// Parse `text/uri-list` or `x-special/gnome-copied-files` into paths.
/// Returns `None` unless every entry is a local file (a link list is text).
pub fn parse_file_list(data: &str) -> Option<Vec<String>> {
    let paths: Vec<Option<String>> = data
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && *l != "copy" && *l != "cut")
        .map(file_uri_to_path)
        .collect();
    if paths.is_empty() || paths.iter().any(Option::is_none) {
        return None;
    }
    Some(paths.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nautilus_and_uri_lists() {
        assert_eq!(
            parse_file_list("copy\nfile:///home/me/a%20b.txt\nfile:///tmp/x.png"),
            Some(vec!["/home/me/a b.txt".into(), "/tmp/x.png".into()])
        );
        assert_eq!(parse_file_list("# comment\r\nfile:///etc/hosts\r\n"), Some(vec!["/etc/hosts".into()]));
        assert_eq!(parse_file_list("cut\nfile://localhost/srv/f"), Some(vec!["/srv/f".into()]));
        assert_eq!(parse_file_list("https://example.com/x"), None, "links are not files");
        assert_eq!(parse_file_list("file:///a\nhttps://b"), None);
        assert_eq!(parse_file_list(""), None);
        assert_eq!(file_uri_to_path("file:///t%C3%A0i%20li%E1%BB%87u"), Some("/tài liệu".into()));
    }
}
