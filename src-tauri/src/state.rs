use std::path::PathBuf;
use std::sync::Mutex;

use qd_core::{Clock, Db};
use qd_platform::{HotkeyStrategy, Session};

use crate::commands::app::FocusReport;
use crate::hotkeys::HotkeyConfig;

pub struct AppState {
    pub db: Db,
    pub clock: Clock,
    pub data_dir: PathBuf,
    pub device_id: String,
    pub session: Session,
    pub strategy: HotkeyStrategy,
    pub hotkeys: HotkeyConfig,
    pub focus_reports: Mutex<Vec<FocusReport>>,
}

impl AppState {
    /// Hook for the sync worker (M5); no-op until sync is configured.
    pub fn on_notes_changed(&self) {}
}
