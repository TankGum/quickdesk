//! X11 / Xwayland watcher using XFixes selection notifications.
//!
//! On GNOME Wayland an unfocused Wayland client cannot read the clipboard, but
//! Mutter mirrors the Wayland clipboard into Xwayland's CLIPBOARD selection,
//! which any X11 client may observe (see spikes/wayland/FINDINGS.md). The UI
//! itself stays Wayland-native; only this thread talks X11.

use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::xfixes::{ConnectionExt as _, SelectionEventMask};
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, Property, Timestamp, Window, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use super::{set_state, ClipEvent, WatcherState, MAX_BYTES};

/// Coalesce bursts: one copy can produce a dozen owner-change events.
const DEBOUNCE: Duration = Duration::from_millis(120);
/// Give up on an owner that never answers a conversion request.
const CONVERT_TIMEOUT: Duration = Duration::from_secs(2);

type BoxError = Box<dyn std::error::Error + Send + Sync>;

struct Atoms {
    clipboard: Atom,
    targets: Atom,
    utf8: Atom,
    text_plain_utf8: Atom,
    incr: Atom,
    property: Atom,
    /// Set by KeePassXC, KDE and others for secrets.
    password_hint: Atom,
}

impl Atoms {
    fn intern(conn: &RustConnection) -> Result<Self, BoxError> {
        let get = |name: &str| -> Result<Atom, BoxError> { Ok(conn.intern_atom(false, name.as_bytes())?.reply()?.atom) };
        Ok(Atoms {
            clipboard: get("CLIPBOARD")?,
            targets: get("TARGETS")?,
            utf8: get("UTF8_STRING")?,
            text_plain_utf8: get("text/plain;charset=utf-8")?,
            incr: get("INCR")?,
            property: get("QUICKDESK_SELECTION")?,
            password_hint: get("x-kde-passwordManagerHint")?,
        })
    }
}

enum Phase {
    Idle,
    /// Owner changed; read it once things calm down.
    Pending { at: Instant, time: Timestamp },
    AwaitTargets { since: Instant, time: Timestamp },
    AwaitText { since: Instant },
    /// Large selection arriving in chunks (INCR protocol).
    Incremental { since: Instant, buf: Vec<u8> },
}

pub(super) fn spawn(on_event: impl Fn(ClipEvent) + Send + 'static, state: Arc<Mutex<WatcherState>>) -> &'static str {
    if std::env::var_os("DISPLAY").is_none() {
        set_state(&state, WatcherState::Unavailable("no X11/Xwayland display (DISPLAY unset)".into()));
        return "x11";
    }
    thread::Builder::new()
        .name("qd-clipboard".into())
        .spawn(move || {
            let mut backoff = Duration::from_secs(1);
            loop {
                let started = Instant::now();
                let err = run(&on_event, &state).err().map(|e| e.to_string()).unwrap_or_default();
                tracing::warn!(error = %err, "clipboard watcher stopped; retrying");
                set_state(&state, WatcherState::Retrying(err));
                if started.elapsed() > Duration::from_secs(60) {
                    backoff = Duration::from_secs(1);
                }
                thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }
        })
        .expect("spawn clipboard thread");
    "x11"
}

/// Next event, waiting at most `timeout` (`None` = forever).
fn next_event(conn: &RustConnection, timeout: Option<Duration>) -> Result<Option<Event>, BoxError> {
    if let Some(ev) = conn.poll_for_event()? {
        return Ok(Some(ev));
    }
    let mut pfd = libc::pollfd { fd: conn.stream().as_raw_fd(), events: libc::POLLIN, revents: 0 };
    let ms = timeout.map(|t| t.as_millis().min(i32::MAX as u128) as i32).unwrap_or(-1);
    // SAFETY: `pfd` is a valid pollfd for the duration of the call.
    let rc = unsafe { libc::poll(&mut pfd, 1, ms) };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::Interrupted {
            return Ok(None);
        }
        return Err(err.into());
    }
    Ok(conn.poll_for_event()?)
}

fn run(on_event: &impl Fn(ClipEvent), state: &Arc<Mutex<WatcherState>>) -> Result<(), BoxError> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    let win: Window = conn.generate_id()?;
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        win,
        root,
        0,
        0,
        1,
        1,
        0,
        WindowClass::INPUT_ONLY,
        x11rb::COPY_FROM_PARENT,
        &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )?;
    conn.xfixes_query_version(5, 0)?.reply()?;
    let atoms = Atoms::intern(&conn)?;
    conn.xfixes_select_selection_input(win, atoms.clipboard, SelectionEventMask::SET_SELECTION_OWNER)?;
    conn.flush()?;
    set_state(state, WatcherState::Running);
    tracing::info!("clipboard watcher running (X11 XFixes)");

    let mut phase = Phase::Idle;
    // An owner change that arrived while we were busy reading the previous one.
    let mut again: Option<Timestamp> = None;

    loop {
        let deadline = match &phase {
            Phase::Idle => None,
            Phase::Pending { at, .. } => Some(*at + DEBOUNCE),
            Phase::AwaitTargets { since, .. } | Phase::AwaitText { since } | Phase::Incremental { since, .. } => {
                Some(*since + CONVERT_TIMEOUT)
            }
        };
        let timeout = deadline.map(|d| d.saturating_duration_since(Instant::now()));
        let event = next_event(&conn, timeout)?;
        let now = Instant::now();

        match (event, &mut phase) {
            (Some(Event::XfixesSelectionNotify(ev)), Phase::Idle | Phase::Pending { .. }) => {
                phase = Phase::Pending { at: now, time: ev.selection_timestamp };
            }
            (Some(Event::XfixesSelectionNotify(ev)), _) => again = Some(ev.selection_timestamp),

            (Some(Event::SelectionNotify(ev)), Phase::AwaitTargets { time, .. }) => {
                let time = *time;
                phase = Phase::Idle;
                if ev.property == x11rb::NONE {
                    continue; // owner refused
                }
                let reply = conn.get_property(true, win, atoms.property, AtomEnum::ATOM, 0, 1024)?.reply()?;
                let targets: Vec<Atom> = reply.value32().map(|v| v.collect()).unwrap_or_default();
                if targets.contains(&atoms.password_hint) {
                    tracing::debug!("skipping selection marked as secret");
                    continue;
                }
                let target = [atoms.utf8, atoms.text_plain_utf8, AtomEnum::STRING.into()]
                    .into_iter()
                    .find(|t| targets.contains(t));
                if let Some(target) = target {
                    conn.convert_selection(win, atoms.clipboard, target, atoms.property, time)?;
                    conn.flush()?;
                    phase = Phase::AwaitText { since: now };
                }
            }
            (Some(Event::SelectionNotify(ev)), Phase::AwaitText { .. }) => {
                phase = Phase::Idle;
                if ev.property == x11rb::NONE {
                    continue;
                }
                let reply =
                    conn.get_property(true, win, atoms.property, AtomEnum::ANY, 0, (MAX_BYTES / 4) as u32 + 1)?.reply()?;
                conn.flush()?;
                if reply.type_ == atoms.incr {
                    // Deleting the property (done above) tells the owner to start sending chunks.
                    phase = Phase::Incremental { since: now, buf: Vec::new() };
                } else if reply.bytes_after == 0 {
                    emit(on_event, &reply.value);
                } else {
                    tracing::debug!("skipping selection larger than {MAX_BYTES} bytes");
                }
            }
            (Some(Event::PropertyNotify(ev)), Phase::Incremental { buf, since })
                if ev.atom == atoms.property && ev.state == Property::NEW_VALUE =>
            {
                let reply = conn.get_property(true, win, atoms.property, AtomEnum::ANY, 0, u32::MAX / 4)?.reply()?;
                conn.flush()?;
                *since = now;
                if reply.value.is_empty() {
                    let done = std::mem::take(buf);
                    phase = Phase::Idle;
                    emit(on_event, &done);
                } else if buf.len() + reply.value.len() > MAX_BYTES {
                    tracing::debug!("incremental selection exceeded {MAX_BYTES} bytes; dropped");
                    phase = Phase::Idle;
                } else {
                    buf.extend_from_slice(&reply.value);
                }
            }
            (Some(_), _) => {}

            // Timeouts.
            (None, Phase::Pending { at, time }) if now >= *at + DEBOUNCE => {
                conn.convert_selection(win, atoms.clipboard, atoms.targets, atoms.property, *time)?;
                conn.flush()?;
                phase = Phase::AwaitTargets { since: now, time: *time };
            }
            (None, Phase::AwaitTargets { since, .. } | Phase::AwaitText { since } | Phase::Incremental { since, .. })
                if now >= *since + CONVERT_TIMEOUT =>
            {
                tracing::debug!("selection owner did not answer in time");
                phase = Phase::Idle;
            }
            (None, _) => {}
        }

        if matches!(phase, Phase::Idle) {
            if let Some(time) = again.take() {
                phase = Phase::Pending { at: now, time };
            }
        }
    }
}

fn emit(on_event: &impl Fn(ClipEvent), bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes).into_owned();
    if !text.trim().is_empty() {
        on_event(ClipEvent { text, source_app: None });
    }
}
