//! AI usage limits: background refresh, top-bar ring icon and its menu.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
/// Local sources (log files) are cheap: re-read every minute.
const TICK: Duration = Duration::from_secs(60);
/// Vendor endpoints rate-limit; ask at most this often on our own…
const LIVE_EVERY: Duration = Duration::from_secs(5 * 60);
/// …and at most this often when the user presses Refresh.
const LIVE_MIN_GAP: Duration = Duration::from_secs(60);
const BACKOFF_START: Duration = Duration::from_secs(10 * 60);
const BACKOFF_MAX: Duration = Duration::from_secs(30 * 60);

/// When we last asked live endpoints, and how long to stay away after a 429.
#[derive(Default)]
struct LiveSchedule {
    last: Option<Instant>,
    blocked_until: Option<Instant>,
    backoff: Option<Duration>,
}

impl LiveSchedule {
    fn due(&self, manual: bool, now: Instant) -> bool {
        if self.blocked_until.is_some_and(|t| now < t) {
            return false;
        }
        let gap = if manual { LIVE_MIN_GAP } else { LIVE_EVERY };
        self.last.is_none_or(|t| now.duration_since(t) >= gap)
    }

    fn record(&mut self, rate_limited: bool, now: Instant) {
        self.last = Some(now);
        if rate_limited {
            let next = self.backoff.map_or(BACKOFF_START, |b| (b * 2).min(BACKOFF_MAX));
            self.backoff = Some(next);
            self.blocked_until = Some(now + next);
        } else {
            self.backoff = None;
            self.blocked_until = None;
        }
    }
}

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
    live: Mutex<LiveSchedule>,
}

impl AiUsageService {
    pub fn new(conn: &rusqlite::Connection) -> Self {
        let tray_enabled = qd_core::settings::get(conn, SETTING_TRAY).ok().flatten().unwrap_or(true);
        let ring = qd_core::settings::get(conn, SETTING_RING).ok().flatten();
        AiUsageService {
            snapshot: Mutex::new(UsageSnapshot { tray_enabled, ring, ..Default::default() }),
            tx: Mutex::new(None),
            live: Mutex::new(LiveSchedule::default()),
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

/// Collect (blocking) and publish. `manual` = the user asked for it.
pub fn refresh(app: &AppHandle, manual: bool) {
    let state = app.state::<AppState>();
    let now = Instant::now();
    let live = state.ai.live.lock().unwrap_or_else(|e| e.into_inner()).due(manual, now);
    let fresh = qd_ai_usage::collect(live);
    if live {
        let limited = fresh.iter().any(|p| p.error.as_deref() == Some("rate_limited"));
        state.ai.live.lock().unwrap_or_else(|e| e.into_inner()).record(limited, now);
        if limited {
            tracing::warn!("usage endpoint rate limited; backing off");
        }
    }
    let skipped = if live { Vec::new() } else { qd_ai_usage::live_providers() };
    let snap = {
        let mut s = state.ai.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        let providers = qd_ai_usage::merge(&s.providers, fresh, &skipped);
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
    let _ = std::thread::Builder::new().name("qd-ai-usage".into()).spawn(move || {
        let mut manual = false;
        loop {
            refresh(&app, manual);
            manual = match rx.recv_timeout(TICK) {
                Ok(()) => {
                    // Coalesce a burst of manual refreshes.
                    while rx.try_recv().is_ok() {}
                    true
                }
                Err(RecvTimeoutError::Timeout) => false,
                Err(RecvTimeoutError::Disconnected) => return,
            };
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
            // Any usage line opens the AI tab. Menu events are app-wide in
            // Tauri, so ignore entries that belong to the other tray menu.
            id if id.starts_with("ai-") => crate::windows::show(app, Target::Ai, now_ms()),
            _ => {}
        })
        .build(app);
    if let Err(e) = built {
        tracing::warn!(error = %e, "failed to create AI usage tray icon");
    }
}

/// macOS-style menu: each tool is a section under a greyed title ("Claude
/// Pro"), one line per limit, then the actions. Clicking a limit opens the
/// AI tab. GNOME draws this menu, so it stays plain text.
fn build_menu(app: &AppHandle, lang: Lang, snap: &UsageSnapshot) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    let mut any = false;
    for (i, p) in snap.providers.iter().enumerate() {
        let lines: Vec<_> = p.windows.iter().enumerate().filter(|(_, w)| w.used_percent.is_some()).collect();
        if lines.is_empty() {
            continue;
        }
        if any {
            menu.append(&PredefinedMenuItem::separator(app)?)?;
        }
        any = true;
        let title = match &p.plan {
            Some(plan) => format!("{} {plan}", p.name),
            None => p.name.clone(),
        };
        menu.append(&MenuItem::with_id(app, format!("ai-h{i}"), title, false, None::<&str>)?)?;
        for (j, w) in lines {
            let label = match tx(lang, &w.id) {
                "" => w.label.clone(),
                l => l.to_owned(),
            };
            let mut text = format!("{label}: {:.0}%", w.used_percent.unwrap_or(0.0));
            if let Some(r) = w.resets_at {
                text.push_str(" · ");
                text.push_str(&tx(lang, "resets_in").replace("{left}", &fmt_duration(lang, r - now_ms() as i64)));
            }
            menu.append(&MenuItem::with_id(app, format!("ai-p{i}-w{j}"), text, true, None::<&str>)?)?;
        }
    }
    if !any {
        menu.append(&MenuItem::with_id(app, "ai-none", tx(lang, "no_percent"), false, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&ring_submenu(app, lang, snap)?)?;
    menu.append(&MenuItem::with_id(app, "ai-refresh", tx(lang, "refresh"), true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "ai-open", tx(lang, "open"), true, None::<&str>)?)?;
    Ok(menu)
}

/// "2 giờ 5 phút" / "2h 5m"; days once over 24 hours.
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
        "refresh" => {
            if vi {
                "Làm mới ngay"
            } else {
                "Refresh Now"
            }
        }
        "open" => {
            if vi {
                "Mở trang AI…"
            } else {
                "Open AI Usage…"
            }
        }
        "no_percent" => {
            if vi {
                "Không cung cấp % hạn mức"
            } else {
                "Does not report a quota percentage"
            }
        }
        "resets_in" => {
            if vi {
                "đặt lại sau {left}"
            } else {
                "resets in {left}"
            }
        }
        "ring" => {
            if vi {
                "Vòng tròn hiển thị"
            } else {
                "Ring Shows"
            }
        }
        "ring_auto" => {
            if vi {
                "Tự động (limit 5 giờ cao nhất)"
            } else {
                "Automatic (highest 5-hour limit)"
            }
        }
        _ => "",
    }
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
    fn live_schedule_spaces_requests_and_backs_off_after_rate_limits() {
        let t0 = Instant::now();
        let mut s = LiveSchedule::default();
        assert!(s.due(false, t0), "first request goes out");
        s.record(false, t0);
        assert!(!s.due(false, t0 + Duration::from_secs(120)), "automatic: every 5 minutes");
        assert!(s.due(true, t0 + Duration::from_secs(61)), "manual: after a minute");
        assert!(s.due(false, t0 + LIVE_EVERY));

        s.record(true, t0);
        assert!(!s.due(true, t0 + Duration::from_secs(9 * 60)), "blocked after 429");
        assert!(s.due(false, t0 + BACKOFF_START));
        s.record(true, t0);
        assert_eq!(s.backoff, Some(BACKOFF_START * 2), "doubles");
        s.record(true, t0);
        s.record(true, t0);
        assert_eq!(s.backoff, Some(BACKOFF_MAX), "capped");
        s.record(false, t0);
        assert!(s.blocked_until.is_none() && s.backoff.is_none(), "reset on success");
    }

    #[test]
    fn durations_read_naturally() {
        assert_eq!(fmt_duration(Lang::En, 5 * 60_000), "5m");
        assert_eq!(fmt_duration(Lang::Vi, (2 * 60 + 5) * 60_000), "2 giờ 5 phút");
        assert_eq!(fmt_duration(Lang::En, (3 * 1440 + 4 * 60) * 60_000), "3d 4h");
        assert_eq!(fmt_duration(Lang::Vi, -10), "0 phút");
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
}
