//! Writing the clipboard and pasting into the previously focused app.
//!
//! Wayland apps may not synthesize input. On Linux there are two ways:
//!
//! * **uinput** virtual keyboard: invisible to the user, needs a one-time udev
//!   rule granting `/dev/uinput` (see [`crate::uinput`]).
//! * xdg-desktop-portal **RemoteDesktop** (keyboard only): works out of the box,
//!   but the desktop asks once and shows a "remote control" indicator while
//!   the session is open.
//!
//! Either way we type Shift+Insert, which pastes in browsers, editors,
//! GTK/Qt/Electron apps *and* terminals (where Ctrl+V does not); the text is
//! put on both CLIPBOARD and PRIMARY so either convention works.

use serde::{Deserialize, Serialize};

/// How to inject the paste keystroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PasteMethod {
    /// uinput when accessible, otherwise the portal.
    #[default]
    Auto,
    Uinput,
    Portal,
}

use std::sync::{Mutex, OnceLock};

/// Run `f` with the process-wide clipboard handle. It is kept alive for the
/// whole process: on X11 the owner must keep serving the data it set.
fn with_clipboard<T>(f: impl FnOnce(&mut arboard::Clipboard) -> Result<T, String>) -> Result<T, String> {
    static CLIPBOARD: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();
    let mut guard = CLIPBOARD.get_or_init(|| Mutex::new(None)).lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(arboard::Clipboard::new().map_err(|e| format!("clipboard unavailable: {e}"))?);
    }
    f(guard.as_mut().expect("initialized above"))
}

/// Put an encoded image (PNG/JPEG) on the clipboard; apps receive PNG.
pub fn write_image(encoded: &[u8]) -> Result<(), String> {
    let rgba = image::load_from_memory(encoded).map_err(|e| format!("cannot decode image: {e}"))?.to_rgba8();
    let (width, height) = rgba.dimensions();
    let data = arboard::ImageData { width: width as usize, height: height as usize, bytes: rgba.into_raw().into() };
    with_clipboard(|cb| cb.set_image(data).map_err(|e| e.to_string()))
}

/// Put a list of files on the clipboard, as a file manager's "Copy" does.
pub fn write_files(paths: &[String]) -> Result<(), String> {
    with_clipboard(|cb| cb.set().file_list(paths).map_err(|e| e.to_string()))
}

/// Which keystroke pastes this kind of content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chord {
    /// Text: also works in terminals, where Ctrl+V does not.
    ShiftInsert,
    /// Images and files: what file managers, chat apps and browsers expect.
    CtrlV,
}

/// Put `text` on the system clipboard (and the PRIMARY selection on Linux).
pub fn write_text(text: &str) -> Result<(), String> {
    with_clipboard(|cb| {
        #[cfg(target_os = "linux")]
        {
            use arboard::{LinuxClipboardKind, SetExtLinux};
            cb.set().clipboard(LinuxClipboardKind::Primary).text(text.to_owned()).map_err(|e| e.to_string())?;
        }
        cb.set_text(text.to_owned()).map_err(|e| e.to_string())
    })
}

#[cfg(target_os = "linux")]
mod imp {
    use std::time::{Duration, Instant};

    use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions};
    use ashpd::desktop::{PersistMode, Session};
    use tokio::sync::Mutex;

    use super::{Chord, PasteMethod};
    use crate::uinput::{self, VirtualKeyboard, KEY_INSERT, KEY_LEFTCTRL, KEY_LEFTSHIFT, KEY_V};

    // X keysyms for the portal.
    const SHIFT_L: i32 = 0xffe1;
    const INSERT: i32 = 0xff63;
    const CONTROL_L: i32 = 0xffe3;
    const V: i32 = 0x0076;

    impl Chord {
        fn keycodes(self) -> [u16; 2] {
            match self {
                Chord::ShiftInsert => [KEY_LEFTSHIFT, KEY_INSERT],
                Chord::CtrlV => [KEY_LEFTCTRL, KEY_V],
            }
        }
        fn keysyms(self) -> [i32; 2] {
            match self {
                Chord::ShiftInsert => [SHIFT_L, INSERT],
                Chord::CtrlV => [CONTROL_L, V],
            }
        }
    }

    struct Active {
        proxy: RemoteDesktop,
        session: Session<RemoteDesktop>,
        last_used: Instant,
    }

    /// A lazily created uinput keyboard, and/or one RemoteDesktop session opened
    /// on demand and closed when idle (GNOME shows an indicator while it is open).
    #[derive(Default)]
    pub struct AutoPaster {
        active: Mutex<Option<Active>>,
        keyboard: std::sync::Mutex<Option<VirtualKeyboard>>,
    }

    impl AutoPaster {
        /// Which backend `method` resolves to right now.
        pub fn resolve(method: PasteMethod) -> PasteMethod {
            match method {
                PasteMethod::Auto if uinput::available() => PasteMethod::Uinput,
                PasteMethod::Auto => PasteMethod::Portal,
                m => m,
            }
        }

        /// Type the paste `chord` into the focused window. For the portal,
        /// `restore_token` comes from a previous call and the returned token must
        /// be stored for the next one.
        pub async fn paste(
            &self,
            method: PasteMethod,
            restore_token: Option<&str>,
            chord: Chord,
        ) -> Result<Option<String>, String> {
            match Self::resolve(method) {
                PasteMethod::Uinput => {
                    self.close_portal().await;
                    self.paste_uinput(chord).map(|_| None)
                }
                _ => self.paste_portal(restore_token, chord).await,
            }
        }

        fn paste_uinput(&self, chord: Chord) -> Result<(), String> {
            let mut kb = self.keyboard.lock().unwrap_or_else(|e| e.into_inner());
            if kb.is_none() {
                *kb = Some(VirtualKeyboard::create()?);
            }
            let result = kb.as_mut().expect("created above").chord(&chord.keycodes());
            if result.is_err() {
                *kb = None; // recreate next time
            }
            result
        }

        async fn close_portal(&self) {
            if let Some(a) = self.active.lock().await.take() {
                let _ = a.session.close().await;
            }
        }

        async fn paste_portal(&self, restore_token: Option<&str>, chord: Chord) -> Result<Option<String>, String> {
            let mut active = self.active.lock().await;
            let mut new_token = None;
            if active.is_none() {
                let (a, token) = Self::open(restore_token).await?;
                *active = Some(a);
                new_token = token;
                // Let focus settle in case a permission dialog was shown.
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
            let a = active.as_mut().expect("opened above");
            let result = Self::press(a, chord).await;
            if result.is_err() {
                // Session may have been revoked; reopen next time.
                if let Some(a) = active.take() {
                    let _ = a.session.close().await;
                }
            } else if let Some(a) = active.as_mut() {
                a.last_used = Instant::now();
            }
            result.map(|_| new_token)
        }

        async fn open(restore_token: Option<&str>) -> Result<(Active, Option<String>), String> {
            let err = |what: &str, e: ashpd::Error| format!("{what}: {e}");
            let proxy = RemoteDesktop::new().await.map_err(|e| err("remote desktop portal unavailable", e))?;
            let session = proxy.create_session(Default::default()).await.map_err(|e| err("create session", e))?;
            proxy
                .select_devices(
                    &session,
                    SelectDevicesOptions::default()
                        .set_devices(ashpd::enumflags2::BitFlags::from(DeviceType::Keyboard))
                        .set_persist_mode(PersistMode::ExplicitlyRevoked)
                        .set_restore_token(restore_token),
                )
                .await
                .map_err(|e| err("select devices", e))?;
            let selected = proxy
                .start(&session, None, Default::default())
                .await
                .map_err(|e| err("start", e))?
                .response()
                .map_err(|e| match e {
                    ashpd::Error::Response(_) => "keyboard access was not allowed".to_owned(),
                    e => err("start", e),
                })?;
            if !selected.devices().contains(DeviceType::Keyboard) {
                let _ = session.close().await;
                return Err("keyboard access was not granted".into());
            }
            let token = selected.restore_token().map(str::to_owned);
            Ok((Active { proxy, session, last_used: Instant::now() }, token))
        }

        async fn press(a: &Active, chord: Chord) -> Result<(), String> {
            let key =
                |sym: i32, state: KeyState| a.proxy.notify_keyboard_keysym(&a.session, sym, state, Default::default());
            let step = Duration::from_millis(8);
            let [modifier, k] = chord.keysyms();
            key(modifier, KeyState::Pressed).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(k, KeyState::Pressed).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(k, KeyState::Released).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(modifier, KeyState::Released).await.map_err(|e| e.to_string())
        }

        /// Close the session if unused for `idle`.
        pub async fn close_if_idle(&self, idle: Duration) {
            let mut active = self.active.lock().await;
            if active.as_ref().is_some_and(|a| a.last_used.elapsed() >= idle) {
                if let Some(a) = active.take() {
                    let _ = a.session.close().await;
                }
            }
        }
    }
}

/// Windows: the window to paste into is the one in front before our popup
/// opens; we bring it back and type the chord with `SendInput`.
#[cfg(windows)]
mod imp {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::time::{Duration, Instant};

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_CONTROL,
        VK_INSERT, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow, SetForegroundWindow};

    use super::{Chord, PasteMethod};

    /// The window that was in front when the popup opened (an HWND).
    static TARGET: AtomicIsize = AtomicIsize::new(0);
    const VK_V: u16 = 0x56;

    /// Call right before showing the popup.
    pub fn remember_target() {
        // SAFETY: plain query.
        let hwnd = unsafe { GetForegroundWindow() };
        TARGET.store(hwnd as isize, Ordering::SeqCst);
    }

    fn key(vk: u16, up: bool) -> INPUT {
        // Insert is an extended key; without the flag it can arrive as numpad 0.
        let extended = if vk == VK_INSERT { KEYEVENTF_EXTENDEDKEY } else { 0 };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: extended | if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    /// Fields kept private so every platform builds it the same way: `default()`.
    #[derive(Default)]
    pub struct AutoPaster {
        _private: (),
    }

    impl AutoPaster {
        pub fn resolve(_method: PasteMethod) -> PasteMethod {
            PasteMethod::Auto
        }

        pub async fn paste(
            &self,
            _method: PasteMethod,
            _restore_token: Option<&str>,
            chord: Chord,
        ) -> Result<Option<String>, String> {
            // Kept as a number: a raw HWND held across `.await` would make
            // this future unusable from Tauri's (multi-threaded) commands.
            let target = TARGET.load(Ordering::SeqCst);
            // SAFETY (all blocks below): plain calls on a window handle that
            // may have closed meanwhile; Windows checks it.
            let front = || unsafe { GetForegroundWindow() } as isize;
            if target == 0 || unsafe { IsWindow(target as _) } == 0 {
                return Err("the window to paste into is gone".into());
            }
            // We just handled the user's key press, so Windows lets us hand
            // the foreground back.
            unsafe { SetForegroundWindow(target as _) };
            let started = Instant::now();
            while front() != target && started.elapsed() < Duration::from_millis(400) {
                tokio_sleep(15).await;
            }
            if front() != target {
                return Err("could not bring the window back to the front".into());
            }
            tokio_sleep(40).await;
            let [modifier, k] = match chord {
                Chord::ShiftInsert => [VK_SHIFT, VK_INSERT],
                Chord::CtrlV => [VK_CONTROL, VK_V],
            };
            let inputs = [key(modifier, false), key(k, false), key(k, true), key(modifier, true)];
            // SAFETY: a valid array of keyboard inputs.
            let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
            if sent as usize != inputs.len() {
                return Err("Windows refused the key presses".into());
            }
            Ok(None)
        }

        pub async fn close_if_idle(&self, _idle: Duration) {}
    }

    async fn tokio_sleep(ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    use std::time::Duration;

    use super::{Chord, PasteMethod};

    /// Fields kept private so every platform builds it the same way: `default()`.
    #[derive(Default)]
    pub struct AutoPaster {
        _private: (),
    }

    impl AutoPaster {
        pub fn resolve(method: PasteMethod) -> PasteMethod {
            method
        }
        pub async fn paste(
            &self,
            _method: PasteMethod,
            _restore_token: Option<&str>,
            _chord: Chord,
        ) -> Result<Option<String>, String> {
            Err("auto-paste is not supported on this platform yet".into())
        }
        pub async fn close_if_idle(&self, _idle: Duration) {}
    }
}

pub use imp::AutoPaster;

/// Remember the window to paste into; call right before showing the popup.
/// Only Windows needs it: elsewhere hiding the popup refocuses that window.
pub fn remember_target() {
    #[cfg(windows)]
    imp::remember_target();
}
