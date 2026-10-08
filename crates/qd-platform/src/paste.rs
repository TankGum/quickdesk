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

/// Put `text` on the system clipboard (and the PRIMARY selection on Linux).
pub fn write_text(text: &str) -> Result<(), String> {
    static CLIPBOARD: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();
    // Kept alive for the whole process: on X11 the owner must keep serving the data.
    let mut guard = CLIPBOARD.get_or_init(|| Mutex::new(None)).lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(arboard::Clipboard::new().map_err(|e| format!("clipboard unavailable: {e}"))?);
    }
    let cb = guard.as_mut().expect("initialized above");
    #[cfg(target_os = "linux")]
    {
        use arboard::{LinuxClipboardKind, SetExtLinux};
        cb.set().clipboard(LinuxClipboardKind::Primary).text(text.to_owned()).map_err(|e| e.to_string())?;
    }
    cb.set_text(text.to_owned()).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
mod imp {
    use std::time::{Duration, Instant};

    use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions};
    use ashpd::desktop::{PersistMode, Session};
    use tokio::sync::Mutex;

    use super::PasteMethod;
    use crate::uinput::{self, VirtualKeyboard, KEY_INSERT, KEY_LEFTSHIFT};

    const SHIFT_L: i32 = 0xffe1;
    const INSERT: i32 = 0xff63;

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

        /// Type Shift+Insert into the focused window. For the portal,
        /// `restore_token` comes from a previous call and the returned token must
        /// be stored for the next one.
        pub async fn paste(&self, method: PasteMethod, restore_token: Option<&str>) -> Result<Option<String>, String> {
            match Self::resolve(method) {
                PasteMethod::Uinput => {
                    self.close_portal().await;
                    self.paste_uinput().map(|_| None)
                }
                _ => self.paste_portal(restore_token).await,
            }
        }

        fn paste_uinput(&self) -> Result<(), String> {
            let mut kb = self.keyboard.lock().unwrap_or_else(|e| e.into_inner());
            if kb.is_none() {
                *kb = Some(VirtualKeyboard::create()?);
            }
            let result = kb.as_mut().expect("created above").chord(&[KEY_LEFTSHIFT, KEY_INSERT]);
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

        async fn paste_portal(&self, restore_token: Option<&str>) -> Result<Option<String>, String> {
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
            let result = Self::shift_insert(a).await;
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

        async fn shift_insert(a: &Active) -> Result<(), String> {
            let key =
                |sym: i32, state: KeyState| a.proxy.notify_keyboard_keysym(&a.session, sym, state, Default::default());
            let step = Duration::from_millis(8);
            key(SHIFT_L, KeyState::Pressed).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(INSERT, KeyState::Pressed).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(INSERT, KeyState::Released).await.map_err(|e| e.to_string())?;
            tokio::time::sleep(step).await;
            key(SHIFT_L, KeyState::Released).await.map_err(|e| e.to_string())
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

#[cfg(not(target_os = "linux"))]
mod imp {
    use std::time::Duration;

    use super::PasteMethod;

    #[derive(Default)]
    pub struct AutoPaster;

    impl AutoPaster {
        pub fn resolve(method: PasteMethod) -> PasteMethod {
            method
        }
        pub async fn paste(
            &self,
            _method: PasteMethod,
            _restore_token: Option<&str>,
        ) -> Result<Option<String>, String> {
            Err("auto-paste is not supported on this platform yet".into())
        }
        pub async fn close_if_idle(&self, _idle: Duration) {}
    }
}

pub use imp::AutoPaster;
