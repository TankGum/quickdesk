//! UI language for the parts Rust renders itself (the tray menu). The web UI
//! has its own dictionary (src/shared/i18n.ts) and follows `ui://language`.

use serde::{Deserialize, Serialize};

pub const SETTINGS_KEY: &str = "ui.language";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LangPref {
    #[default]
    Auto,
    En,
    Vi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Vi,
}

impl LangPref {
    pub fn resolve(self) -> Lang {
        match self {
            LangPref::En => Lang::En,
            LangPref::Vi => Lang::Vi,
            LangPref::Auto => system_lang(),
        }
    }
}

fn system_lang() -> Lang {
    let vars = ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"];
    let first = vars.iter().filter_map(|v| std::env::var(v).ok()).find(|v| !v.is_empty());
    match first {
        Some(v) if v.to_ascii_lowercase().starts_with("vi") => Lang::Vi,
        _ => Lang::En,
    }
}

/// Tray menu strings. GNOME draws tray menus itself, so they stay plain text,
/// macOS style: short title-case labels, a check mark for the one toggle.
pub fn tr(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (Lang::Vi, "quick-note") => "Ghi chú nhanh mới",
        (Lang::Vi, "notes") => "Ghi chú",
        (Lang::Vi, "clipboard") => "Lịch sử clipboard",
        (Lang::Vi, "ports") => "Cổng",
        (Lang::Vi, "main") => "Mở QuickDesk",
        (Lang::Vi, "quit") => "Thoát QuickDesk",
        (Lang::Vi, "save-clipboard") => "Lưu lịch sử clipboard",
        (Lang::Vi, "update") => "Cập nhật lên {version}…",
        (_, "quick-note") => "New Quick Note",
        (_, "notes") => "Notes",
        (_, "clipboard") => "Clipboard History",
        (_, "ports") => "Ports",
        (_, "main") => "Open QuickDesk",
        (_, "quit") => "Quit QuickDesk",
        (_, "save-clipboard") => "Save Clipboard History",
        (_, "update") => "Update to {version}…",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_preference_wins_and_all_tray_keys_exist() {
        assert_eq!(LangPref::Vi.resolve(), Lang::Vi);
        assert_eq!(LangPref::En.resolve(), Lang::En);
        for lang in [Lang::En, Lang::Vi] {
            for key in ["quick-note", "notes", "clipboard", "ports", "main", "quit", "save-clipboard", "update"] {
                assert!(!tr(lang, key).is_empty(), "{lang:?} {key}");
            }
        }
    }
}
