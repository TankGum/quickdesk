//! Windows clipboard watcher: a message-only window registered with
//! `AddClipboardFormatListener` gets `WM_CLIPBOARDUPDATE` on every change.
//! Like the X11 watcher it prefers copied files, then text, then images, and
//! never reads what password managers mark as excluded from clipboard
//! history and monitoring.

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, GetClipboardData, GetClipboardOwner, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows_sys::Win32::System::Ole::{CF_DIB, CF_HDROP, CF_UNICODETEXT};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Shell::DragQueryFileW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowThreadProcessId, RegisterClassW,
    TranslateMessage, HWND_MESSAGE, MSG, WM_CLIPBOARDUPDATE, WNDCLASSW,
};

use super::dib::dib_to_bmp;
use super::{set_state, ClipContent, ClipEvent, WatcherState, MAX_IMAGE_BYTES, MAX_TEXT_BYTES};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Formats registered by name (the ids differ per session).
struct Formats {
    png: u32,
    /// Password managers and Windows itself use these to keep secrets out
    /// of clipboard history and away from clipboard monitors.
    exclude: u32,
    viewer_ignore: u32,
    history: u32,
}

impl Formats {
    fn register() -> Self {
        // SAFETY: plain calls with NUL-terminated UTF-16 names.
        let reg = |name: &str| unsafe { RegisterClipboardFormatW(wide(name).as_ptr()) };
        Formats {
            png: reg("PNG"),
            exclude: reg("ExcludeClipboardContentFromMonitorProcessing"),
            viewer_ignore: reg("Clipboard Viewer Ignore"),
            history: reg("CanIncludeInClipboardHistory"),
        }
    }
}

thread_local! {
    /// What `WM_CLIPBOARDUPDATE` runs; set on the watcher thread before the loop.
    static ON_UPDATE: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) };
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        ON_UPDATE.with(|f| {
            if let Some(f) = f.borrow_mut().as_mut() {
                f();
            }
        });
        return 0;
    }
    // SAFETY: forwarding the message we were given.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// The clipboard, opened for reading; closed on drop.
struct Open;

impl Open {
    /// Another program may hold the clipboard for a moment: retry briefly.
    fn new(hwnd: HWND) -> Option<Self> {
        for _ in 0..10 {
            // SAFETY: `hwnd` is our own window.
            if unsafe { OpenClipboard(hwnd) } != 0 {
                return Some(Open);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        None
    }

    fn has(&self, format: u32) -> bool {
        // SAFETY: the clipboard is open.
        format != 0 && unsafe { IsClipboardFormatAvailable(format) } != 0
    }

    /// The bytes of a global-memory format, up to `max`.
    fn bytes(&self, format: u32, max: usize) -> Option<Vec<u8>> {
        // SAFETY: the clipboard is open; the handle is only used while locked.
        unsafe {
            let h = GetClipboardData(format);
            if h.is_null() {
                return None;
            }
            let size = GlobalSize(h);
            if size == 0 || size > max {
                return None;
            }
            let p = GlobalLock(h) as *const u8;
            if p.is_null() {
                return None;
            }
            let out = std::slice::from_raw_parts(p, size).to_vec();
            GlobalUnlock(h);
            Some(out)
        }
    }

    fn text(&self) -> Option<String> {
        let bytes = self.bytes(CF_UNICODETEXT as u32, MAX_TEXT_BYTES * 2)?;
        let units: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
        let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
        Some(String::from_utf16_lossy(&units[..end]))
    }

    fn files(&self) -> Option<Vec<String>> {
        // SAFETY: the clipboard is open; HDROP is read through its own API.
        unsafe {
            let h = GetClipboardData(CF_HDROP as u32);
            if h.is_null() {
                return None;
            }
            let count = DragQueryFileW(h, u32::MAX, std::ptr::null_mut(), 0);
            let mut paths = Vec::new();
            for i in 0..count {
                let len = DragQueryFileW(h, i, std::ptr::null_mut(), 0);
                let mut buf = vec![0u16; len as usize + 1];
                let got = DragQueryFileW(h, i, buf.as_mut_ptr(), buf.len() as u32);
                if got > 0 {
                    paths.push(String::from_utf16_lossy(&buf[..got as usize]));
                }
            }
            (!paths.is_empty()).then_some(paths)
        }
    }

    fn dword(&self, format: u32) -> Option<u32> {
        let b = self.bytes(format, 64)?;
        (b.len() >= 4).then(|| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        // SAFETY: we opened it.
        unsafe { CloseClipboard() };
    }
}

fn image_png(open: &Open, formats: &Formats) -> Option<Vec<u8>> {
    if open.has(formats.png) {
        if let Some(png) = open.bytes(formats.png, MAX_IMAGE_BYTES) {
            return Some(png);
        }
    }
    if !open.has(CF_DIB as u32) {
        return None;
    }
    let dib = open.bytes(CF_DIB as u32, MAX_IMAGE_BYTES * 4)?;
    let img = image::load_from_memory_with_format(&dib_to_bmp(&dib)?, image::ImageFormat::Bmp).ok()?;
    let mut png = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()?;
    (png.len() <= MAX_IMAGE_BYTES).then_some(png)
}

/// The program that put this on the clipboard (`chrome`, `Code`…), if known.
fn owner_app() -> Option<String> {
    // SAFETY: plain queries on a window handle and a process we only read.
    unsafe {
        let hwnd = GetClipboardOwner();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        CloseHandle(process);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).map(str::to_owned)
    }
}

/// Read what is on the clipboard now. `None`: nothing we keep, or a secret.
fn read(hwnd: HWND, formats: &Formats) -> Option<ClipContent> {
    let open = Open::new(hwnd)?;
    if open.has(formats.exclude) || open.has(formats.viewer_ignore) {
        return None;
    }
    if open.has(formats.history) && open.dword(formats.history) == Some(0) {
        return None;
    }
    if open.has(CF_HDROP as u32) {
        if let Some(paths) = open.files() {
            return Some(ClipContent::Files(paths));
        }
    }
    if open.has(CF_UNICODETEXT as u32) {
        if let Some(text) = open.text() {
            if !text.trim().is_empty() && text.len() <= MAX_TEXT_BYTES {
                return Some(ClipContent::Text(text));
            }
        }
    }
    image_png(&open, formats).map(|bytes| ClipContent::Image { mime: "image/png".into(), bytes })
}

pub(super) fn spawn(on_event: impl Fn(ClipEvent) + Send + 'static, state: Arc<Mutex<WatcherState>>) -> &'static str {
    let started = std::thread::Builder::new().name("qd-clipboard".into()).spawn(move || {
        let class = wide("QuickDeskClipboardWatcher");
        // SAFETY: registering a window class and creating a message-only window
        // owned by this thread; both live until the process exits.
        let hwnd = unsafe {
            let instance = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wc);
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                std::ptr::null_mut(),
                instance,
                std::ptr::null::<c_void>(),
            )
        };
        // SAFETY: `hwnd` was just created on this thread.
        if hwnd.is_null() || unsafe { AddClipboardFormatListener(hwnd) } == 0 {
            return set_state(&state, WatcherState::Unavailable("could not listen to the clipboard".into()));
        }
        let formats = Formats::register();
        // Do not record whatever was on the clipboard before we started.
        // SAFETY: plain query.
        let mut last_seq = unsafe { GetClipboardSequenceNumber() };
        ON_UPDATE.with(|f| {
            *f.borrow_mut() = Some(Box::new(move || {
                // SAFETY: plain query.
                let seq = unsafe { GetClipboardSequenceNumber() };
                if seq == last_seq {
                    return;
                }
                last_seq = seq;
                if let Some(content) = read(hwnd, &formats) {
                    on_event(ClipEvent { content, source_app: owner_app() });
                }
            }));
        });
        set_state(&state, WatcherState::Running);
        // SAFETY: a standard message loop for this thread's window.
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
    if let Err(e) = started {
        tracing::error!(error = %e, "clipboard watcher thread");
    }
    "win32"
}
