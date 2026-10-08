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

/// Tray menu strings.
pub fn tr(lang: Lang, key: &str) -> &'static str {
    // Native tray menus cannot be styled (GNOME draws them), so each entry
    // gets a symbol; the red one marks Quit.
    match (lang, key) {
        (Lang::Vi, "quick-note") => "✏️  Ghi nhanh",
        (Lang::Vi, "notes") => "📝  Ghi chú",
        (Lang::Vi, "clipboard") => "📋  Clipboard",
        (Lang::Vi, "ports") => "🔌  Cổng",
        (Lang::Vi, "main") => "🏠  Mở QuickDesk",
        (Lang::Vi, "quit") => "⛔  Thoát QuickDesk",
        (Lang::Vi, "pause") => "⏸️  Tạm dừng lưu clipboard",
        (Lang::Vi, "paused") => "▶️  Đang tạm dừng lưu clipboard: bấm để bật lại",
        (_, "quick-note") => "✏️  Quick note",
        (_, "notes") => "📝  Notes",
        (_, "clipboard") => "📋  Clipboard",
        (_, "ports") => "🔌  Ports",
        (_, "main") => "🏠  Open QuickDesk",
        (_, "quit") => "⛔  Quit QuickDesk",
        (_, "pause") => "⏸️  Pause clipboard history",
        (_, "paused") => "▶️  Clipboard history paused: click to resume",
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
            for key in ["quick-note", "notes", "clipboard", "ports", "main", "quit", "pause", "paused"] {
                assert!(!tr(lang, key).is_empty(), "{lang:?} {key}");
            }
        }
    }
}
