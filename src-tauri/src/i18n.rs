//! UI language for the parts Rust renders itself (the tray menus). The web UI
//! has its own dictionaries (src/shared/i18n.ts, src/shared/locales/) and
//! follows `ui://language`. The languages are the website's.

use serde::{Deserialize, Serialize};

pub const SETTINGS_KEY: &str = "ui.language";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LangPref {
    #[default]
    Auto,
    En,
    Vi,
    Zh,
    Ja,
    Hi,
    Es,
    Fr,
    Ar,
    Bn,
    Ru,
    Pt,
    Id,
}

/// In the order of the columns of `STRINGS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Vi,
    Zh,
    Ja,
    Hi,
    Es,
    Fr,
    Ar,
    Bn,
    Ru,
    Pt,
    Id,
}

impl Lang {
    #[cfg(test)]
    const ALL: [Lang; 12] = [
        Lang::En,
        Lang::Vi,
        Lang::Zh,
        Lang::Ja,
        Lang::Hi,
        Lang::Es,
        Lang::Fr,
        Lang::Ar,
        Lang::Bn,
        Lang::Ru,
        Lang::Pt,
        Lang::Id,
    ];

    /// "vi_VN.UTF-8", "pt-BR", "zh-Hans-CN" → that language, if QuickDesk has it.
    fn from_tag(tag: &str) -> Option<Lang> {
        let code = tag.split(['-', '_', '.', '@']).next()?.to_ascii_lowercase();
        Some(match code.as_str() {
            "en" => Lang::En,
            "vi" => Lang::Vi,
            "zh" => Lang::Zh,
            "ja" => Lang::Ja,
            "hi" => Lang::Hi,
            "es" => Lang::Es,
            "fr" => Lang::Fr,
            "ar" => Lang::Ar,
            "bn" => Lang::Bn,
            "ru" => Lang::Ru,
            "pt" => Lang::Pt,
            "id" => Lang::Id,
            _ => return None,
        })
    }
}

impl LangPref {
    pub fn resolve(self) -> Lang {
        match self {
            LangPref::Auto => system_lang(),
            LangPref::En => Lang::En,
            LangPref::Vi => Lang::Vi,
            LangPref::Zh => Lang::Zh,
            LangPref::Ja => Lang::Ja,
            LangPref::Hi => Lang::Hi,
            LangPref::Es => Lang::Es,
            LangPref::Fr => Lang::Fr,
            LangPref::Ar => Lang::Ar,
            LangPref::Bn => Lang::Bn,
            LangPref::Ru => Lang::Ru,
            LangPref::Pt => Lang::Pt,
            LangPref::Id => Lang::Id,
        }
    }
}

/// The first of the user's languages that QuickDesk has, else English. The
/// locale variables come first (LANGUAGE may list several: "vi:en"); macOS
/// and Windows keep the user's languages outside the environment.
fn system_lang() -> Lang {
    let vars = ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"];
    let env = vars.iter().filter_map(|v| std::env::var(v).ok()).find(|v| !v.is_empty()).unwrap_or_default();
    env.split(':')
        .map(str::to_owned)
        .chain(sys_locale::get_locales())
        .find_map(|tag| Lang::from_tag(&tag))
        .unwrap_or(Lang::En)
}

/// Menu strings, one column per language (`Lang` order). GNOME draws tray
/// menus itself, so they stay plain text, macOS style: short title-case
/// labels, a check mark for the one toggle.
#[rustfmt::skip]
const STRINGS: &[(&str, [&str; 12])] = &[
    // Main tray menu
    ("quick-note", ["New Quick Note", "Ghi chú nhanh mới", "新建快速笔记", "新しいクイックメモ", "नया क्विक नोट", "Nueva nota rápida", "Nouvelle note rapide", "ملاحظة سريعة جديدة", "নতুন কুইক নোট", "Новая быстрая заметка", "Nova nota rápida", "Catatan cepat baru"]),
    ("notes", ["Notes", "Ghi chú", "笔记", "メモ", "नोट्स", "Notas", "Notes", "الملاحظات", "নোট", "Заметки", "Notas", "Catatan"]),
    ("clipboard", ["Clipboard History", "Lịch sử clipboard", "剪贴板历史", "クリップボード履歴", "क्लिपबोर्ड इतिहास", "Historial del portapapeles", "Historique du presse-papiers", "سجل الحافظة", "ক্লিপবোর্ড ইতিহাস", "История буфера обмена", "Histórico da área de transferência", "Riwayat clipboard"]),
    ("ports", ["Ports", "Cổng", "端口", "ポート", "पोर्ट", "Puertos", "Ports", "المنافذ", "পোর্ট", "Порты", "Portas", "Port"]),
    ("main", ["Open QuickDesk", "Mở QuickDesk", "打开 QuickDesk", "QuickDesk を開く", "QuickDesk खोलें", "Abrir QuickDesk", "Ouvrir QuickDesk", "فتح QuickDesk", "QuickDesk খুলুন", "Открыть QuickDesk", "Abrir o QuickDesk", "Buka QuickDesk"]),
    ("quit", ["Quit QuickDesk", "Thoát QuickDesk", "退出 QuickDesk", "QuickDesk を終了", "QuickDesk बंद करें", "Salir de QuickDesk", "Quitter QuickDesk", "إنهاء QuickDesk", "QuickDesk বন্ধ করুন", "Выйти из QuickDesk", "Sair do QuickDesk", "Keluar dari QuickDesk"]),
    ("save-clipboard", ["Save Clipboard History", "Lưu lịch sử clipboard", "保存剪贴板历史", "クリップボード履歴を保存", "क्लिपबोर्ड इतिहास सहेजें", "Guardar historial del portapapeles", "Enregistrer l’historique du presse-papiers", "حفظ سجل الحافظة", "ক্লিপবোর্ড ইতিহাস সংরক্ষণ", "Сохранять историю буфера обмена", "Salvar histórico da área de transferência", "Simpan riwayat clipboard"]),
    ("update", ["Update to {version}…", "Cập nhật lên {version}…", "更新到 {version}…", "{version} にアップデート…", "{version} पर अपडेट करें…", "Actualizar a {version}…", "Mettre à jour vers {version}…", "التحديث إلى {version}…", "{version}-এ আপডেট করুন…", "Обновить до {version}…", "Atualizar para {version}…", "Perbarui ke {version}…"]),
    // AI usage tray: limit names (the ids of qd-ai-usage's windows)
    ("five_hour", ["5-hour", "5 giờ", "5 小时", "5 時間", "5 घंटे", "5 horas", "5 heures", "5 ساعات", "5 ঘণ্টা", "5 часов", "5 horas", "5 jam"]),
    ("weekly", ["Weekly", "Tuần", "每周", "週間", "साप्ताहिक", "Semanal", "Hebdomadaire", "أسبوعي", "সাপ্তাহিক", "Неделя", "Semanal", "Mingguan"]),
    ("weekly_opus", ["Weekly (Opus)", "Tuần (Opus)", "每周 (Opus)", "週間 (Opus)", "साप्ताहिक (Opus)", "Semanal (Opus)", "Hebdomadaire (Opus)", "أسبوعي (Opus)", "সাপ্তাহিক (Opus)", "Неделя (Opus)", "Semanal (Opus)", "Mingguan (Opus)"]),
    ("weekly_sonnet", ["Weekly (Sonnet)", "Tuần (Sonnet)", "每周 (Sonnet)", "週間 (Sonnet)", "साप्ताहिक (Sonnet)", "Semanal (Sonnet)", "Hebdomadaire (Sonnet)", "أسبوعي (Sonnet)", "সাপ্তাহিক (Sonnet)", "Неделя (Sonnet)", "Semanal (Sonnet)", "Mingguan (Sonnet)"]),
    ("monthly", ["Monthly", "Tháng", "每月", "月間", "मासिक", "Mensual", "Mensuel", "شهري", "মাসিক", "Месяц", "Mensal", "Bulanan"]),
    ("extra", ["Extra usage", "Extra usage", "额外用量", "追加利用量", "अतिरिक्त उपयोग", "Uso adicional", "Utilisation supplémentaire", "استخدام إضافي", "অতিরিক্ত ব্যবহার", "Дополнительное использование", "Uso extra", "Penggunaan tambahan"]),
    ("extra_off", ["Extra usage: off", "Extra usage: tắt", "额外用量：关闭", "追加利用量：オフ", "अतिरिक्त उपयोग: बंद", "Uso adicional: desactivado", "Utilisation supplémentaire : désactivée", "الاستخدام الإضافي: متوقف", "অতিরিক্ত ব্যবহার: বন্ধ", "Дополнительное использование: выкл.", "Uso extra: desativado", "Penggunaan tambahan: mati"]),
    ("quota_hit", ["Last ran out of quota", "Hết quota lần cuối", "上次用完配额", "前回クォータを使い切った時刻", "पिछली बार कोटा खत्म हुआ", "Última vez sin cuota", "Dernier quota épuisé", "آخر نفاد للحصة", "শেষবার কোটা শেষ হয়েছে", "Последний раз лимит исчерпан", "Última vez sem cota", "Terakhir kehabisan kuota"]),
    // AI usage tray: the rest
    ("tooltip", ["AI usage", "AI usage", "AI 用量", "AI 利用量", "AI उपयोग", "Uso de IA", "Utilisation de l’IA", "استخدام الذكاء الاصطناعي", "AI ব্যবহার", "Использование ИИ", "Uso de IA", "Penggunaan AI"]),
    ("refresh", ["Refresh Now", "Làm mới ngay", "立即刷新", "今すぐ更新", "अभी ताज़ा करें", "Actualizar ahora", "Actualiser maintenant", "حدّث الآن", "এখনই রিফ্রেশ", "Обновить сейчас", "Atualizar agora", "Segarkan sekarang"]),
    ("open", ["Open AI Usage…", "Mở trang AI…", "打开 AI 用量…", "AI 利用量を開く…", "AI उपयोग खोलें…", "Abrir uso de IA…", "Ouvrir l’utilisation de l’IA…", "فتح استخدام الذكاء الاصطناعي…", "AI ব্যবহার খুলুন…", "Открыть использование ИИ…", "Abrir uso de IA…", "Buka penggunaan AI…"]),
    ("no_percent", ["Does not report a quota percentage", "Không cung cấp % hạn mức", "不提供配额百分比", "クォータの割合を報告しません", "कोटा का प्रतिशत नहीं बताता", "No informa el porcentaje de cuota", "N’indique pas le pourcentage de quota", "لا يعرض نسبة الحصة", "কোটার শতাংশ জানায় না", "Не сообщает процент лимита", "Não informa a porcentagem da cota", "Tidak melaporkan persentase kuota"]),
    ("resets_in", ["resets in {left}", "đặt lại sau {left}", "{left}后重置", "{left}後にリセット", "{left} में रीसेट", "se reinicia en {left}", "réinitialisation dans {left}", "يُعاد ضبطه بعد {left}", "{left} পরে রিসেট", "сброс через {left}", "reinicia em {left}", "diatur ulang dalam {left}"]),
    ("ring", ["Track", "Theo dõi", "跟踪", "追跡", "ट्रैक", "Seguir", "Suivre", "تتبُّع", "ট্র্যাক", "Отслеживать", "Acompanhar", "Pantau"]),
    ("ring_auto", ["Automatic (highest 5-hour limit)", "Tự động (limit 5 giờ cao nhất)", "自动（最高的 5 小时限额）", "自動（最も高い5時間制限）", "स्वचालित (सबसे ऊँची 5 घंटे की सीमा)", "Automático (límite de 5 horas más alto)", "Automatique (limite de 5 heures la plus haute)", "تلقائي (أعلى حد لـ 5 ساعات)", "স্বয়ংক্রিয় (সর্বোচ্চ ৫ ঘণ্টার সীমা)", "Автоматически (самый высокий 5-часовой лимит)", "Automático (maior limite de 5 horas)", "Otomatis (batas 5 jam tertinggi)"]),
    // Time left, compact: minutes, hours and minutes, days and hours
    ("dur_m", ["{m}m", "{m} phút", "{m} 分", "{m} 分", "{m} मि", "{m} min", "{m} min", "{m} دقيقة", "{m} মি", "{m} мин", "{m} min", "{m} m"]),
    ("dur_hm", ["{h}h {m}m", "{h} giờ {m} phút", "{h} 小时 {m} 分", "{h} 時間 {m} 分", "{h} घं {m} मि", "{h} h {m} min", "{h} h {m}", "{h} ساعة و{m} دقيقة", "{h} ঘ {m} মি", "{h} ч {m} мин", "{h} h {m} min", "{h} j {m} m"]),
    ("dur_dh", ["{d}d {h}h", "{d} ngày {h} giờ", "{d} 天 {h} 小时", "{d} 日 {h} 時間", "{d} दिन {h} घं", "{d} d {h} h", "{d} j {h} h", "{d} يوم و{h} ساعة", "{d} দিন {h} ঘ", "{d} д {h} ч", "{d} d {h} h", "{d} h {h} j"]),
];

/// The menu string `key` in `lang`; "" for an unknown key.
pub fn tr(lang: Lang, key: &str) -> &'static str {
    STRINGS.iter().find(|(k, _)| *k == key).map_or("", |(_, row)| row[lang as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_preference_wins_and_every_string_is_translated() {
        assert_eq!(LangPref::Vi.resolve(), Lang::Vi);
        assert_eq!(LangPref::En.resolve(), Lang::En);
        assert_eq!(LangPref::Ar.resolve(), Lang::Ar);
        for lang in Lang::ALL {
            for (key, _) in STRINGS {
                assert!(!tr(lang, key).is_empty(), "{lang:?} {key}");
            }
        }
        assert_eq!(tr(Lang::Id, "nope"), "");
    }

    #[test]
    fn language_tags() {
        assert_eq!(Lang::from_tag("vi_VN.UTF-8"), Some(Lang::Vi));
        assert_eq!(Lang::from_tag("pt-BR"), Some(Lang::Pt));
        assert_eq!(Lang::from_tag("zh-Hans-CN"), Some(Lang::Zh));
        assert_eq!(Lang::from_tag("C.UTF-8"), None);
        assert_eq!(Lang::from_tag("de_DE"), None);
    }

    #[test]
    fn serialized_names_match_the_web_ui() {
        assert_eq!(serde_json::to_string(&Lang::Pt).unwrap(), "\"pt\"");
        assert_eq!(serde_json::from_str::<LangPref>("\"id\"").unwrap(), LangPref::Id);
    }
}
