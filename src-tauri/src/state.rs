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
    pub hotkeys: Mutex<HotkeyConfig>,
    pub focus_reports: Mutex<Vec<FocusReport>>,
    pub clipboard: ClipboardService,
    pub sync: SyncService,
    pub lang_pref: Mutex<crate::i18n::LangPref>,
    pub ai: crate::ai_usage::AiUsageService,
}

impl AppState {
    pub fn lang(&self) -> crate::i18n::Lang {
        self.lang_pref.lock().unwrap_or_else(|e| e.into_inner()).resolve()
    }

    /// A local note changed: schedule a (debounced) push.
    pub fn on_notes_changed(&self) {
        self.sync.trigger(Trigger::LocalChange);
    }
}
