use std::path::PathBuf;
use std::sync::Mutex;

use qd_core::{Clock, Db};
use qd_platform::{HotkeyStrategy, Session};

use crate::clipboard::ClipboardService;
use crate::commands::app::FocusReport;
use crate::hotkeys::HotkeyConfig;
use crate::sync::{SyncService, Trigger};

pub struct AppState {
    pub db: Db,
    pub clock: Clock,
    pub data_dir: PathBuf,
    pub device_id: String,
    pub session: Session,
    pub strategy: HotkeyStrategy,
    pub hotkeys: HotkeyConfig,
    pub focus_reports: Mutex<Vec<FocusReport>>,
    pub clipboard: ClipboardService,
    pub sync: SyncService,
}

impl AppState {
    /// A local note changed: schedule a (debounced) push.
    pub fn on_notes_changed(&self) {
        self.sync.trigger(Trigger::LocalChange);
    }
}
