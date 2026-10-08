//! OS / desktop-environment integration that does not depend on Tauri.

pub mod accel;
pub mod clipboard;
pub mod gnome;
pub mod paste;
pub mod session;
#[cfg(target_os = "linux")]
pub mod uinput;

pub use accel::Accelerator;
pub use session::{HotkeyStrategy, Session};
