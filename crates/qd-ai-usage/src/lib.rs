//! Usage limits of the AI coding tools installed on this machine.
//!
//! Each provider detects itself from files the tool keeps in the home
//! directory and reports its limit windows (5-hour, weekly, monthly, extra
//! credits). Nothing is configured by hand: a tool that is not installed is
//! simply not shown.

mod antigravity;
mod claude;
mod codex;

use std::path::PathBuf;

use serde::Serialize;

/// One limit window, e.g. Claude's 5-hour session or Codex's weekly limit.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    /// Stable id the UI translates: `five_hour`, `weekly`, `weekly_opus`,
    /// `monthly`, `extra`, `quota_hit`, or a provider-specific kind.
    pub id: String,
    /// Fallback label when the UI has no translation for `id`.
    pub label: String,
    /// 0–100; `None` when the tool does not report a percentage.
    pub used_percent: Option<f64>,
    /// Unix ms.
    pub resets_at: Option<i64>,
    /// Extra text, e.g. "$1.20 of $50.00".
    pub detail: Option<String>,
}

impl UsageWindow {
    pub fn is_five_hour(&self) -> bool {
        self.id == "five_hour"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    /// `claude`, `codex`, `antigravity`.
    pub provider: String,
    pub name: String,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    /// `live` (asked the vendor just now) or `local` (read from the tool's logs).
    pub source: &'static str,
    /// When the numbers were true (unix ms): now for live data, the log time for local data.
    pub as_of: Option<i64>,
    pub error: Option<String>,
}

impl ProviderUsage {
    fn new(provider: &str, name: &str, source: &'static str) -> Self {
        ProviderUsage {
            provider: provider.into(),
            name: name.into(),
            plan: None,
            windows: Vec::new(),
            source,
            as_of: None,
            error: None,
        }
    }

    pub fn five_hour_percent(&self) -> Option<f64> {
        self.windows.iter().find(|w| w.is_five_hour()).and_then(|w| w.used_percent)
    }
}

pub(crate) fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Every detected tool with its current usage. Blocking (network for Claude).
pub fn collect() -> Vec<ProviderUsage> {
    let mut out = Vec::new();
    if let Some(p) = claude::Claude::detect() {
        out.push(p.fetch());
    }
    if let Some(p) = codex::Codex::detect() {
        out.push(p.fetch());
    }
    if let Some(p) = antigravity::Antigravity::detect() {
        out.push(p.fetch());
    }
    out
}

/// The 5-hour usage to show in the top bar: the most used one across tools.
pub fn headline_percent(usage: &[ProviderUsage]) -> Option<f64> {
    usage
        .iter()
        .filter_map(ProviderUsage::five_hour_percent)
        .fold(None, |acc, p| Some(acc.map_or(p, |a: f64| a.max(p))))
}

/// `2026-10-08T09:20:00.156839+00:00` → unix ms (UTC offsets supported).
pub(crate) fn parse_rfc3339_ms(s: &str) -> Option<i64> {
    let s = s.trim();
    let (date, rest) = s.split_once('T')?;
    let mut d = date.split('-').map(|p| p.parse::<i64>());
    let (y, m, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
    // Split off the zone: Z, +hh:mm or -hh:mm.
    let (time, offset_min) = if let Some(t) = rest.strip_suffix('Z') {
        (t, 0)
    } else if let Some(i) = rest.rfind(['+', '-']) {
        let (t, z) = rest.split_at(i);
        let sign = if z.starts_with('-') { -1 } else { 1 };
        let mut hm = z[1..].split(':').map(|p| p.parse::<i64>());
        let (h, mi) = (hm.next()?.ok()?, hm.next().unwrap_or(Ok(0)).ok()?);
        (t, sign * (h * 60 + mi))
    } else {
        (rest, 0)
    };
    let (hms, frac) = time.split_once('.').unwrap_or((time, "0"));
    let mut t = hms.split(':').map(|p| p.parse::<i64>());
    let (hh, mm, ss) = (t.next()?.ok()?, t.next()?.ok()?, t.next()?.ok()?);
    let ms: i64 = format!("{:0<3}", &frac[..frac.len().min(3)]).parse().ok()?;
    Some((days_from_civil(y, m, day) * 86_400 + hh * 3600 + mm * 60 + ss - offset_min * 60) * 1000 + ms)
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm).
pub(crate) fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_with_offsets() {
        assert_eq!(parse_rfc3339_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339_ms("2026-10-08T09:20:00.156839+00:00"), Some(1_791_451_200_156));
        assert_eq!(parse_rfc3339_ms("2026-10-08T16:20:00+07:00"), Some(1_791_451_200_000));
        assert_eq!(parse_rfc3339_ms("nope"), None);
    }

    #[test]
    fn headline_is_the_highest_five_hour_usage() {
        let mk = |p: Option<f64>| {
            let mut u = ProviderUsage::new("x", "X", "live");
            if let Some(p) = p {
                u.windows.push(UsageWindow {
                    id: "five_hour".into(),
                    label: String::new(),
                    used_percent: Some(p),
                    resets_at: None,
                    detail: None,
                });
            }
            u
        };
        assert_eq!(headline_percent(&[mk(Some(20.0)), mk(None), mk(Some(57.0))]), Some(57.0));
        assert_eq!(headline_percent(&[mk(None)]), None);
    }
}
