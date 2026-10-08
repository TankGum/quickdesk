use std::path::PathBuf;
use std::sync::Mutex;

use qd_core::Db;
use qd_platform::{HotkeyStrategy, Session};

use crate::commands::FocusReport;
use crate::hotkeys::HotkeyConfig;

pub struct AppState {
    #[allow(dead_code)] // used by modules from M2 on
    pub db: Db,
    pub data_dir: PathBuf,
    pub device_id: String,
    pub session: Session,
    pub strategy: HotkeyStrategy,
    pub hotkeys: HotkeyConfig,
    pub focus_reports: Mutex<Vec<FocusReport>>,
}
