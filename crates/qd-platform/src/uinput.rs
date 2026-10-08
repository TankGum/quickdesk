//! Kernel-level virtual keyboard via `/dev/uinput`.
//!
//! The compositor sees an ordinary keyboard, so there is no consent dialog
//! and no "remote control" indicator. It needs write access to
//! `/dev/uinput`, granted to the logged-in user by a one-time udev rule
//! (the same one `steam-devices` ships), see [`SETUP_COMMAND`].

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::time::Duration;

const DEVICE: &str = "/dev/uinput";

pub const UDEV_RULE: &str = r#"KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput""#;
pub const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/70-quickdesk-uinput.rules";

/// One-time setup, shown to the user to run in a terminal.
pub const SETUP_COMMAND: &str = concat!(
    "echo 'KERNEL==\"uinput\", SUBSYSTEM==\"misc\", TAG+=\"uaccess\", OPTIONS+=\"static_node=uinput\"' ",
    "| sudo tee /etc/udev/rules.d/70-quickdesk-uinput.rules && ",
    "sudo udevadm control --reload-rules && sudo udevadm trigger --action=change --sysname-match=uinput && sudo udevadm settle"
);

// linux/input-event-codes.h
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const SYN_REPORT: u16 = 0;
pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_V: u16 = 47;
pub const KEY_INSERT: u16 = 110;
const KEYS: [u16; 4] = [KEY_LEFTCTRL, KEY_LEFTSHIFT, KEY_V, KEY_INSERT];

// linux/uinput.h ioctls (_IOW('U', n, T) / _IO('U', n)).
const UI_SET_EVBIT: libc::c_ulong = 0x4004_5564;
const UI_SET_KEYBIT: libc::c_ulong = 0x4004_5565;
const UI_DEV_SETUP: libc::c_ulong = 0x405c_5503;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;
const BUS_VIRTUAL: u16 = 0x06;

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[repr(C)]
struct UinputSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputEvent {
    time: libc::timeval,
    kind: u16,
    code: u16,
    value: i32,
}

/// Whether we may create a virtual keyboard right now.
pub fn available() -> bool {
    OpenOptions::new().write(true).open(DEVICE).is_ok()
}

pub struct VirtualKeyboard {
    file: File,
}

fn ioctl(fd: i32, req: libc::c_ulong, arg: libc::c_ulong) -> std::io::Result<()> {
    // SAFETY: plain uinput ioctls on an fd we own; pointer args point to live structs.
    if unsafe { libc::ioctl(fd, req as _, arg) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

impl VirtualKeyboard {
    pub const NAME: &'static str = "QuickDesk virtual keyboard";

    pub fn create() -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(DEVICE)
            .map_err(|e| format!("cannot open {DEVICE}: {e} (one-time setup needed)"))?;
        let fd = file.as_raw_fd();
        let setup_err = |what: &str, e: std::io::Error| format!("uinput {what}: {e}");
        ioctl(fd, UI_SET_EVBIT, EV_KEY as _).map_err(|e| setup_err("set evbit", e))?;
        for key in KEYS {
            ioctl(fd, UI_SET_KEYBIT, key as _).map_err(|e| setup_err("set keybit", e))?;
        }
        let mut setup = UinputSetup {
            id: InputId { bustype: BUS_VIRTUAL, vendor: 0x5144, product: 0x0001, version: 1 },
            name: [0; 80],
            ff_effects_max: 0,
        };
        setup.name[..Self::NAME.len()].copy_from_slice(Self::NAME.as_bytes());
        ioctl(fd, UI_DEV_SETUP, &setup as *const _ as _).map_err(|e| setup_err("setup", e))?;
        ioctl(fd, UI_DEV_CREATE, 0).map_err(|e| setup_err("create", e))?;
        // The compositor needs a moment to pick up a new input device.
        std::thread::sleep(Duration::from_millis(200));
        Ok(VirtualKeyboard { file })
    }

    fn emit(&mut self, kind: u16, code: u16, value: i32) -> std::io::Result<()> {
        let ev = InputEvent { time: libc::timeval { tv_sec: 0, tv_usec: 0 }, kind, code, value };
        // SAFETY: InputEvent is repr(C) plain data; we view it as bytes for write(2).
        let bytes = unsafe {
            std::slice::from_raw_parts(&ev as *const InputEvent as *const u8, std::mem::size_of::<InputEvent>())
        };
        self.file.write_all(bytes)
    }

    fn key(&mut self, code: u16, down: bool) -> std::io::Result<()> {
        self.emit(EV_KEY, code, down as i32)?;
        self.emit(EV_SYN, SYN_REPORT, 0)?;
        std::thread::sleep(Duration::from_millis(8));
        Ok(())
    }

    /// Press `keys` in order, then release them in reverse.
    pub fn chord(&mut self, keys: &[u16]) -> Result<(), String> {
        let run = |kb: &mut Self| -> std::io::Result<()> {
            for &k in keys {
                kb.key(k, true)?;
            }
            for &k in keys.iter().rev() {
                kb.key(k, false)?;
            }
            Ok(())
        };
        run(self).map_err(|e| format!("uinput write: {e}"))
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        let _ = ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_layout_matches_kernel_abi() {
        assert_eq!(std::mem::size_of::<UinputSetup>(), 92, "UI_DEV_SETUP encodes size 0x5c");
        assert_eq!(std::mem::size_of::<InputEvent>(), 24);
        assert!(SETUP_COMMAND.contains(UDEV_RULE_PATH));
        assert!(SETUP_COMMAND.contains(UDEV_RULE), "command must install exactly UDEV_RULE");
    }

    #[test]
    #[ignore = "needs write access to /dev/uinput"]
    fn creates_a_visible_input_device() {
        let kb = VirtualKeyboard::create().unwrap();
        let devices = std::fs::read_to_string("/proc/bus/input/devices").unwrap();
        assert!(devices.contains(VirtualKeyboard::NAME), "device not registered");
        drop(kb);
    }
}
