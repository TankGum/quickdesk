//! AI usage limits: background refresh, top-bar ring icon and its menu.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;

use qd_ai_usage::{ProviderUsage, UsageWindow};
use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::cli::Target;
use crate::i18n::Lang;
use crate::ipc::now_ms;
use crate::state::AppState;

const TRAY_ID: &str = "ai-usage";
const SETTING_TRAY: &str = "ai.tray";
const SETTING_RING: &str = "ai.ring";

/// Which limit the top-bar ring shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RingChoice {
    pub provider: String,
    pub window: String,
}

/// The ring's value: the chosen limit when it has a percentage, otherwise
/// the highest 5-hour usage across tools ("automatic").
pub fn pick(providers: &[ProviderUsage], choice: Option<&RingChoice>) -> (Option<f64>, Option<RingChoice>) {
    let percent_of = |c: &RingChoice| {
        providers
            .iter()
            .find(|p| p.provider == c.provider)
            .and_then(|p| p.windows.iter().find(|w| w.id == c.window))
            .and_then(|w| w.used_percent)
    };
    if let Some(c) = choice {
        if let Some(p) = percent_of(c) {
            return (Some(p), Some(c.clone()));
        }
    }
    providers
        .iter()
        .flat_map(|p| p.windows.iter().filter(|w| w.is_five_hour()).map(move |w| (p, w)))
        .filter_map(|(p, w)| {
            w.used_percent.map(|pct| (pct, RingChoice { provider: p.provider.clone(), window: w.id.clone() }))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map_or((None, None), |(pct, c)| (Some(pct), Some(c)))
}
const REFRESH_EVERY: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub providers: Vec<ProviderUsage>,
    pub updated_at: Option<i64>,
    /// What the ring shows (percent) and which limit that is.
    pub headline: Option<f64>,
    pub ring_shows: Option<RingChoice>,
    /// The user's choice; `None` = automatic.
    pub ring: Option<RingChoice>,
    pub tray_enabled: bool,
}

pub struct AiUsageService {
    snapshot: Mutex<UsageSnapshot>,
    tx: Mutex<Option<Sender<()>>>,
}

impl AiUsageService {
    pub fn new(conn: &rusqlite::Connection) -> Self {
        let tray_enabled = qd_core::settings::get(conn, SETTING_TRAY).ok().flatten().unwrap_or(true);
        let ring = qd_core::settings::get(conn, SETTING_RING).ok().flatten();
        AiUsageService {
            snapshot: Mutex::new(UsageSnapshot { tray_enabled, ring, ..Default::default() }),
            tx: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> UsageSnapshot {
        self.snapshot.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn refresh_soon(&self) {
        if let Some(tx) = self.tx.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = tx.send(());
        }
    }
}

/// Collect now (blocking) and publish.
pub fn refresh(app: &AppHandle) {
    let providers = qd_ai_usage::collect();
    let state = app.state::<AppState>();
    let snap = {
        let mut s = state.ai.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        (s.headline, s.ring_shows) = pick(&providers, s.ring.as_ref());
        s.providers = providers;
        s.updated_at = Some(now_ms() as i64);
        s.clone()
    };
    tracing::debug!(headline = ?snap.headline, tools = snap.providers.len(), "ai usage refreshed");
    update_tray(app, &snap);
    let _ = app.emit("ai://usage", snap);
}

pub fn start(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<()>();
    *app.state::<AppState>().ai.tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    let app = app.clone();
    let _ = std::thread::Builder::new().name("qd-ai-usage".into()).spawn(move || loop {
        refresh(&app);
        match rx.recv_timeout(REFRESH_EVERY) {
            Ok(()) | Err(RecvTimeoutError::Timeout) => {
                // Coalesce a burst of manual refreshes.
                while rx.try_recv().is_ok() {}
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    });
}

pub fn set_tray_enabled(app: &AppHandle, enabled: bool) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    qd_core::settings::set(&*state.db.conn()?, SETTING_TRAY, &enabled)?;
    let snap = {
        let mut s = state.ai.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        s.tray_enabled = enabled;
        s.clone()
    };
    update_tray(app, &snap);
    let _ = app.emit("ai://usage", snap);
    Ok(())
}

/// Choose what the ring shows (`None` = automatic) and redraw it.
pub fn set_ring(app: &AppHandle, choice: Option<RingChoice>) -> qd_core::Result<()> {
    let state = app.state::<AppState>();
    qd_core::settings::set(&*state.db.conn()?, SETTING_RING, &choice)?;
    let snap = {
        let mut s = state.ai.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        (s.headline, s.ring_shows) = pick(&s.providers, choice.as_ref());
        s.ring = choice;
        s.clone()
    };
    update_tray(app, &snap);
    let _ = app.emit("ai://usage", snap);
    Ok(())
}

/// Re-render after a language change.
pub fn relabel(app: &AppHandle) {
    let snap = app.state::<AppState>().ai.snapshot();
    update_tray(app, &snap);
}

fn update_tray(app: &AppHandle, snap: &UsageSnapshot) {
    if !snap.tray_enabled || snap.providers.is_empty() {
        let _ = app.remove_tray_by_id(TRAY_ID);
        return;
    }
    let lang = app.state::<AppState>().lang();
    let icon = Image::new_owned(crate::ring::render(snap.headline), crate::ring::SIZE, crate::ring::SIZE);
    let menu = match build_menu(app, lang, snap) {
        Ok(m) => m,
        Err(e) => return tracing::warn!(error = %e, "ai usage menu"),
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_menu(Some(menu));
        return;
    }
    let built = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("AI usage")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "ai-refresh" => app.state::<AppState>().ai.refresh_soon(),
            "ai-ring-auto" => {
                let _ = set_ring(app, None);
            }
            id if id.starts_with("ai-ring-") => {
                let snap = app.state::<AppState>().ai.snapshot();
                if let Some(choice) = ring_choice_for_id(&snap, id) {
                    let _ = set_ring(app, Some(choice));
                }
            }
            _ => crate::windows::show(app, Target::Ai, now_ms()),
        })
        .build(app);
    if let Err(e) = built {
        tracing::warn!(error = %e, "failed to create AI usage tray icon");
    }
}

fn build_menu(app: &AppHandle, lang: Lang, snap: &UsageSnapshot) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    let line = |id: String, text: String| MenuItem::with_id(app, id, text, true, None::<&str>);
    for (i, p) in snap.providers.iter().enumerate() {
        if i > 0 {
            menu.append(&PredefinedMenuItem::separator(app)?)?;
        }
        menu.append(&line(format!("ai-p{i}"), provider_heading(lang, p))?)?;
        if let Some(err) = &p.error {
            menu.append(&line(format!("ai-p{i}-err"), format!("    {}", error_text(lang, err)))?)?;
        }
        for (j, w) in p.windows.iter().enumerate() {
            menu.append(&line(format!("ai-p{i}-w{j}"), format!("    {}", window_text(lang, w, p.as_of)))?)?;
        }
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&ring_submenu(app, lang, snap)?)?;
    if let Some(at) = snap.updated_at {
        menu.append(&line("ai-updated".into(), tx(lang, "updated").replace("{time}", &fmt_time(at)))?)?;
    }
    menu.append(&line("ai-refresh".into(), tx(lang, "refresh").into())?)?;
    menu.append(&line("ai-open".into(), tx(lang, "open").into())?)?;
    Ok(menu)
}

/// Selectable limits: every window that reports a percentage.
fn ring_options(snap: &UsageSnapshot) -> impl Iterator<Item = (usize, usize, &ProviderUsage, &UsageWindow)> {
    snap.providers
        .iter()
        .enumerate()
        .flat_map(|(i, p)| p.windows.iter().enumerate().map(move |(j, w)| (i, j, p, w)))
        .filter(|(_, _, _, w)| w.used_percent.is_some())
}

fn ring_choice_for_id(snap: &UsageSnapshot, id: &str) -> Option<RingChoice> {
    ring_options(snap)
        .find(|(i, j, _, _)| id == format!("ai-ring-{i}-{j}"))
        .map(|(_, _, p, w)| RingChoice { provider: p.provider.clone(), window: w.id.clone() })
}

fn ring_submenu(app: &AppHandle, lang: Lang, snap: &UsageSnapshot) -> tauri::Result<Submenu<Wry>> {
    let sub = Submenu::with_id(app, "ai-ring", tx(lang, "ring"), true)?;
    sub.append(&CheckMenuItem::with_id(
        app,
        "ai-ring-auto",
        tx(lang, "ring_auto"),
        true,
        snap.ring.is_none(),
        None::<&str>,
    )?)?;
    for (i, j, p, w) in ring_options(snap) {
        let label = match tx(lang, &w.id) {
            "" => w.label.clone(),
            l => l.to_owned(),
        };
        let checked = snap.ring.as_ref().is_some_and(|c| c.provider == p.provider && c.window == w.id);
        sub.append(&CheckMenuItem::with_id(
            app,
            format!("ai-ring-{i}-{j}"),
            format!("{} · {label}", p.name),
            true,
            checked,
            None::<&str>,
        )?)?;
    }
    Ok(sub)
}

fn tx(lang: Lang, key: &str) -> &'static str {
    let vi = lang == Lang::Vi;
    match key {
        "five_hour" => {
            if vi {
                "5 giờ"
            } else {
                "5-hour"
            }
        }
        "weekly" => {
            if vi {
                "Tuần"
            } else {
                "Weekly"
            }
        }
        "weekly_opus" => {
            if vi {
                "Tuần (Opus)"
            } else {
                "Weekly (Opus)"
            }
        }
        "weekly_sonnet" => {
            if vi {
                "Tuần (Sonnet)"
            } else {
                "Weekly (Sonnet)"
            }
        }
        "monthly" => {
            if vi {
                "Tháng"
            } else {
                "Monthly"
            }
        }
        "extra" => "Extra usage",
        "extra_off" => {
            if vi {
                "Extra usage: tắt"
            } else {
                "Extra usage: off"
            }
        }
        "quota_hit" => {
            if vi {
                "Hết quota lần cuối"
            } else {
                "Last ran out of quota"
            }
        }
        "resets" => {
            if vi {
                "đặt lại {time} (còn {left})"
            } else {
                "resets {time} (in {left})"
            }
        }
        "as_of" => {
            if vi {
                "số liệu lúc {time}"
            } else {
                "as of {time}"
            }
        }
        "updated" => {
            if vi {
                "Cập nhật lúc {time}"
            } else {
                "Updated {time}"
            }
        }
        "refresh" => {
            if vi {
                "Làm mới"
            } else {
                "Refresh"
            }
        }
        "open" => {
            if vi {
                "Xem chi tiết…"
            } else {
                "Show details…"
            }
        }
        "login_expired" => {
            if vi {
                "Đăng nhập đã hết hạn: mở Claude Code để làm mới"
            } else {
                "Login expired: open Claude Code to refresh it"
            }
        }
        "no_data" => {
            if vi {
                "Chưa có số liệu: dùng công cụ này một lần"
            } else {
                "No data yet: use this tool once"
            }
        }
        "no_percent" => {
            if vi {
                "Không cung cấp % hạn mức"
            } else {
                "Does not report a quota percentage"
            }
        }
        "ring" => {
            if vi {
                "Vòng tròn hiển thị"
            } else {
                "Ring shows"
            }
        }
        "ring_auto" => {
            if vi {
                "Tự động (limit 5 giờ cao nhất)"
            } else {
                "Automatic (highest 5-hour limit)"
            }
        }
        "error" => {
            if vi {
                "Lỗi: "
            } else {
                "Error: "
            }
        }
        _ => "",
    }
}

fn provider_heading(lang: Lang, p: &ProviderUsage) -> String {
    let mut s = p.name.clone();
    if let Some(plan) = &p.plan {
        s.push_str(&format!(" · {plan}"));
    }
    if p.source == "local" {
        if let Some(at) = p.as_of {
            s.push_str(&format!(" ({})", tx(lang, "as_of").replace("{time}", &fmt_time(at))));
        }
    }
    s
}

fn error_text(lang: Lang, err: &str) -> String {
    match err {
        "login_expired" | "no_data" | "no_percent" => tx(lang, err).into(),
        other => format!("{}{}", tx(lang, "error"), other.chars().take(80).collect::<String>()),
    }
}

fn window_text(lang: Lang, w: &UsageWindow, as_of: Option<i64>) -> String {
    let label = match tx(lang, &w.id) {
        "" => w.label.clone(),
        l => l.to_owned(),
    };
    if w.id == "quota_hit" {
        return format!("{label}: {}", as_of.map(fmt_time).unwrap_or_default());
    }
    let mut s = label;
    if let Some(p) = w.used_percent {
        s.push_str(&format!(": {p:.0}%"));
    }
    if let Some(d) = &w.detail {
        s.push_str(&format!(" · {d}"));
    }
    if let Some(r) = w.resets_at {
        let left = fmt_duration(lang, r - now_ms() as i64);
        s.push_str(&format!(" · {}", tx(lang, "resets").replace("{time}", &fmt_time(r)).replace("{left}", &left)));
    }
    s
}

/// Local wall-clock: "16:20" today, else "13/10 05:00".
pub fn fmt_time(ms: i64) -> String {
    let (day, hm) = local_parts(ms);
    let (today, _) = local_parts(now_ms() as i64);
    if day == today {
        hm
    } else {
        format!("{} {hm}", day)
    }
}

fn fmt_duration(lang: Lang, ms: i64) -> String {
    let min = (ms.max(0) + 59_999) / 60_000;
    let (d, h, m) = (min / 1440, (min % 1440) / 60, min % 60);
    let (dl, hl, ml) = if lang == Lang::Vi { (" ngày", " giờ", " phút") } else { ("d", "h", "m") };
    match (d, h) {
        (0, 0) => format!("{m}{ml}"),
        (0, _) => format!("{h}{hl} {m}{ml}"),
        _ => format!("{d}{dl} {h}{hl}"),
    }
}

/// ("dd/mm", "HH:MM") in local time.
fn local_parts(ms: i64) -> (String, String) {
    #[cfg(unix)]
    // SAFETY: localtime_r only writes into the struct we own.
    unsafe {
        let t: libc::time_t = (ms / 1000) as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if !libc::localtime_r(&t, &mut tm).is_null() {
            return (format!("{:02}/{:02}", tm.tm_mday, tm.tm_mon + 1), format!("{:02}:{:02}", tm.tm_hour, tm.tm_min));
        }
    }
    let secs = ms / 1000;
    (format!("day {}", secs / 86_400), format!("{:02}:{:02} UTC", (secs % 86_400) / 3600, (secs % 3600) / 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: &str, windows: &[(&str, Option<f64>)]) -> ProviderUsage {
        ProviderUsage {
            provider: id.into(),
            name: id.into(),
            plan: None,
            windows: windows
                .iter()
                .map(|(w, p)| UsageWindow {
                    id: (*w).into(),
                    label: (*w).into(),
                    used_percent: *p,
                    resets_at: None,
                    detail: None,
                })
                .collect(),
            source: "live",
            as_of: None,
            error: None,
        }
    }

    #[test]
    fn ring_follows_the_choice_and_falls_back_to_highest_five_hour() {
        let ps = [
            provider("claude", &[("five_hour", Some(64.0)), ("weekly", Some(51.0))]),
            provider("codex", &[("five_hour", Some(80.0)), ("monthly", Some(76.0))]),
        ];
        let choice = |p: &str, w: &str| RingChoice { provider: p.into(), window: w.into() };
        assert_eq!(pick(&ps, None), (Some(80.0), Some(choice("codex", "five_hour"))));
        assert_eq!(pick(&ps, Some(&choice("claude", "weekly"))), (Some(51.0), Some(choice("claude", "weekly"))));
        // A choice that has no number right now falls back to automatic.
        assert_eq!(pick(&ps, Some(&choice("gemini", "five_hour"))).0, Some(80.0));
        assert_eq!(pick(&[provider("codex", &[("monthly", Some(76.0))])], None), (None, None));
    }

    #[test]
    fn durations_read_naturally() {
        assert_eq!(fmt_duration(Lang::En, 5 * 60_000), "5m");
        assert_eq!(fmt_duration(Lang::Vi, (2 * 60 + 5) * 60_000), "2 giờ 5 phút");
        assert_eq!(fmt_duration(Lang::En, (3 * 1440 + 4 * 60) * 60_000), "3d 4h");
        assert_eq!(fmt_duration(Lang::En, -10), "0m");
    }

    #[test]
    fn window_lines_translate_known_ids_and_keep_unknown_labels() {
        let w = UsageWindow {
            id: "five_hour".into(),
            label: "session".into(),
            used_percent: Some(60.4),
            resets_at: None,
            detail: None,
        };
        assert_eq!(window_text(Lang::Vi, &w, None), "5 giờ: 60%");
        let other = UsageWindow {
            id: "weekly_cowork".into(),
            label: "weekly cowork".into(),
            used_percent: Some(5.0),
            resets_at: None,
            detail: None,
        };
        assert_eq!(window_text(Lang::En, &other, None), "weekly cowork: 5%");
        assert_eq!(error_text(Lang::En, "login_expired"), "Login expired: open Claude Code to refresh it");
    }
}
