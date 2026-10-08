//! OpenAI Codex CLI. Every response Codex receives includes the account's
//! rate limits, and the CLI writes them into its session logs
//! (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`). We read the newest one,
//! so the numbers are as fresh as the last Codex turn (shown as "as of").

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::Value;

use crate::{ProviderUsage, UsageWindow};

/// Only the tail of a session file is scanned; rate limits repeat every turn.
const TAIL_BYTES: u64 = 512 * 1024;
const MAX_FILES: usize = 20;

pub struct Codex {
    sessions: PathBuf,
}

impl Codex {
    pub fn detect() -> Option<Self> {
        let dir =
            std::env::var_os("CODEX_HOME").map(PathBuf::from).or_else(|| crate::home().map(|h| h.join(".codex")))?;
        let sessions = dir.join("sessions");
        sessions.is_dir().then_some(Codex { sessions })
    }

    pub fn fetch(&self) -> ProviderUsage {
        let mut usage = ProviderUsage::new("codex", "Codex", "local");
        let mut files = Vec::new();
        collect_jsonl(&self.sessions, &mut files, 0);
        files.sort_by_key(|(_, mtime)| std::cmp::Reverse(*mtime));
        for (path, mtime) in files.into_iter().take(MAX_FILES) {
            if let Some((limits, ts)) = tail(&path).and_then(|t| last_rate_limits(&t)) {
                usage.as_of = ts.or(Some(mtime));
                usage.plan = limits.get("plan_type").and_then(Value::as_str).map(str::to_owned);
                usage.windows = windows(&limits, usage.as_of.unwrap_or(mtime));
                return usage;
            }
        }
        usage.error = Some("no_data".into());
        usage
    }
}

fn collect_jsonl(dir: &Path, out: &mut Vec<(PathBuf, i64)>, depth: u8) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() && depth < 4 {
            collect_jsonl(&path, out, depth + 1);
        } else if path.extension().is_some_and(|x| x == "jsonl") {
            let mtime = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            out.push((path, mtime));
        }
    }
}

fn tail(path: &Path) -> Option<String> {
    let mut f = fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    f.seek(SeekFrom::Start(len.saturating_sub(TAIL_BYTES))).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn find_key<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(m) => m.get(key).or_else(|| m.values().find_map(|x| find_key(x, key))),
        Value::Array(a) => a.iter().find_map(|x| find_key(x, key)),
        _ => None,
    }
}

/// The newest `rate_limits` object and its event time (unix ms).
pub(crate) fn last_rate_limits(text: &str) -> Option<(Value, Option<i64>)> {
    text.lines().rev().filter(|l| l.contains("\"rate_limits\"")).find_map(|line| {
        let v: Value = serde_json::from_str(line).ok()?;
        let limits = find_key(&v, "rate_limits").filter(|l| l.is_object())?.clone();
        let ts = v.get("timestamp").and_then(Value::as_str).and_then(crate::parse_rfc3339_ms);
        Some((limits, ts))
    })
}

fn window_id(minutes: i64) -> (&'static str, String) {
    match minutes {
        300 => ("five_hour", "5h".into()),
        10080 => ("weekly", "weekly".into()),
        43200 | 44640 | 40320 => ("monthly", "monthly".into()),
        m => ("window", format!("{} h", m / 60)),
    }
}

pub(crate) fn windows(limits: &Value, as_of: i64) -> Vec<UsageWindow> {
    let mut out = Vec::new();
    for key in ["primary", "secondary"] {
        let Some(w) = limits.get(key).filter(|w| w.is_object()) else { continue };
        let minutes = w.get("window_minutes").and_then(Value::as_i64).unwrap_or(0);
        let (id, label) = window_id(minutes);
        let resets_at = w
            .get("resets_at")
            .and_then(Value::as_i64)
            .map(|s| s * 1000)
            .or_else(|| w.get("resets_in_seconds").and_then(Value::as_i64).map(|s| as_of + s * 1000));
        out.push(UsageWindow {
            id: id.into(),
            label,
            used_percent: w.get("used_percent").and_then(Value::as_f64),
            resets_at,
            detail: None,
        });
    }
    if let Some(c) = limits.get("credits").filter(|c| c.get("has_credits").and_then(Value::as_bool) == Some(true)) {
        let balance = c.get("balance").and_then(|b| b.as_f64().or_else(|| b.as_str().and_then(|s| s.parse().ok())));
        out.push(UsageWindow {
            id: "extra".into(),
            label: "credits".into(),
            used_percent: None,
            resets_at: None,
            detail: balance.map(|b| format!("{b:.2}")),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_newest_rate_limits_from_session_log() {
        let log = [
            r#"{"timestamp":"2026-09-28T08:00:00Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":10.0,"window_minutes":300,"resets_at":1790000000},"secondary":{"used_percent":3.0,"window_minutes":10080,"resets_at":1790500000},"plan_type":"plus"}}}"#,
            r#"{"timestamp":"2026-09-28T08:40:51Z","type":"response_item","payload":{"text":"no limits here"}}"#,
            r#"{"timestamp":"2026-09-28T09:00:00Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":42.0,"window_minutes":300,"resets_at":1790001000},"secondary":null,"credits":{"has_credits":true,"balance":"12.5"},"plan_type":"plus"}}}"#,
        ]
        .join("\n");
        let (limits, ts) = last_rate_limits(&log).unwrap();
        assert_eq!(ts, crate::parse_rfc3339_ms("2026-09-28T09:00:00Z"));
        let w = windows(&limits, ts.unwrap());
        let ids: Vec<&str> = w.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, vec!["five_hour", "extra"]);
        assert_eq!(w[0].used_percent, Some(42.0));
        assert_eq!(w[0].resets_at, Some(1_790_001_000_000));
        assert_eq!(w[1].detail.as_deref(), Some("12.50"));
    }

    #[test]
    fn free_plan_monthly_window_and_relative_reset() {
        let limits: Value = serde_json::from_str(
            r#"{"primary":{"used_percent":76.0,"window_minutes":43200,"resets_in_seconds":3600},"secondary":null,"credits":{"has_credits":false},"plan_type":"free"}"#,
        )
        .unwrap();
        let w = windows(&limits, 1_000_000);
        assert_eq!(w.len(), 1);
        assert_eq!((w[0].id.as_str(), w[0].resets_at), ("monthly", Some(1_000_000 + 3_600_000)));
    }
}
