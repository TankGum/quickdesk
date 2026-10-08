//! OS / desktop-environment integration that does not depend on Tauri.

pub mod accel;
pub mod clipboard;
pub mod gnome;
pub mod session;

pub use accel::Accelerator;
pub use session::{HotkeyStrategy, Session};
