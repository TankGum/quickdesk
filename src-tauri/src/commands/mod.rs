//! Tauri command handlers: thin wrappers that call module crates and emit events.

pub mod app;
pub mod clipboard;
pub mod notes;
pub mod ports;

use serde::Serialize;

/// Error shape every command returns to the frontend.
#[derive(Debug, Serialize)]
pub struct CmdError {
    pub code: &'static str,
    pub message: String,
}

pub type CmdResult<T> = Result<T, CmdError>;

impl CmdError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        CmdError { code, message: message.into() }
    }
}

impl From<qd_core::Error> for CmdError {
    fn from(e: qd_core::Error) -> Self {
        CmdError::new("internal", e.to_string())
    }
}

impl From<qd_notes::Error> for CmdError {
    fn from(e: qd_notes::Error) -> Self {
        let code = match e {
            qd_notes::Error::NotFound(_) => "not_found",
            qd_notes::Error::EmptyBody => "invalid",
            _ => "internal",
        };
        CmdError::new(code, e.to_string())
    }
}

impl From<qd_ports::Error> for CmdError {
    fn from(e: qd_ports::Error) -> Self {
        let code = match e {
            qd_ports::Error::PermissionDenied(_) => "permission_denied",
            qd_ports::Error::NoSuchProcess(_) => "not_found",
            qd_ports::Error::Protected(_) => "invalid",
            _ => "internal",
        };
        CmdError::new(code, e.to_string())
    }
}

/// Run blocking work off the main thread.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> CmdResult<T> + Send + 'static) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| CmdError::new("internal", e.to_string()))?
}

impl From<qd_clipboard::Error> for CmdError {
    fn from(e: qd_clipboard::Error) -> Self {
        let code = match e {
            qd_clipboard::Error::NotFound(_) => "not_found",
            _ => "internal",
        };
        CmdError::new(code, e.to_string())
    }
}
