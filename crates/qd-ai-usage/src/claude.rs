//! Claude (Claude Code / claude.ai subscription).
//!
//! Live numbers come from the same endpoint Claude Code's `/usage` uses,
//! authenticated with Claude Code's own OAuth login. We only *read* the
//! token: refreshing it would rotate the refresh token behind Claude Code's
//! back, so an expired login is reported instead ("open Claude Code").
//! The endpoint is not a public API and may change; parsing is lenient.

use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use crate::{now_ms, parse_rfc3339_ms, ProviderUsage, UsageWindow};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

pub struct Claude {
    credentials: PathBuf,
}

impl Claude {
    pub fn detect() -> Option<Self> {
        let dir = std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| crate::home().map(|h| h.join(".claude")))?;
        let credentials = dir.join(".credentials.json");
        credentials.exists().then_some(Claude { credentials })
    }

    pub fn fetch(&self) -> ProviderUsage {
        let mut usage = ProviderUsage::new("claude", "Claude", "live");
        match self.fetch_inner(&mut usage) {
            Ok(()) => usage.as_of = Some(now_ms()),
            Err(e) => usage.error = Some(e),
        }
        usage
    }

    fn fetch_inner(&self, usage: &mut ProviderUsage) -> Result<(), String> {
        let raw = std::fs::read_to_string(&self.credentials).map_err(|e| format!("cannot read Claude login: {e}"))?;
        let cred: Value = serde_json::from_str(&raw).map_err(|_| "Claude login file is not valid JSON".to_owned())?;
        let oauth = cred.get("claudeAiOauth").ok_or("not logged in to Claude Code with a subscription")?;
        usage.plan = oauth.get("subscriptionType").and_then(Value::as_str).map(capitalize);
        let token = oauth.get("accessToken").and_then(Value::as_str).ok_or("no access token in Claude login")?;
        if oauth.get("expiresAt").and_then(Value::as_i64).is_some_and(|exp| exp <= now_ms()) {
            return Err("login_expired".into());
        }
        let resp = ureq::get(USAGE_URL)
            .timeout(Duration::from_secs(15))
            .set("Authorization", &format!("Bearer {token}"))
            .set("anthropic-beta", "oauth-2025-04-20")
            .set("User-Agent", concat!("quickdesk/", env!("CARGO_PKG_VERSION")))
            .call();
        let body: Value = match resp {
            Ok(r) => r.into_json().map_err(|e| format!("unexpected usage response: {e}"))?,
            Err(ureq::Error::Status(401, _)) => return Err("login_expired".into()),
            Err(ureq::Error::Status(429, _)) => return Err("rate_limited".into()),
            Err(ureq::Error::Status(code, _)) => return Err(format!("usage request failed (HTTP {code})")),
            Err(ureq::Error::Transport(_)) => return Err("offline".into()),
        };
        usage.windows = parse(&body);
        Ok(())
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn ms(v: &Value) -> Option<i64> {
    v.as_str().and_then(parse_rfc3339_ms)
}

/// Map the response to windows. Prefers the generic `limits` list (so new
/// kinds show up without code changes) and falls back to the named objects.
pub(crate) fn parse(body: &Value) -> Vec<UsageWindow> {
    let mut windows = Vec::new();
    if let Some(limits) = body.get("limits").and_then(Value::as_array) {
        for l in limits {
            let kind = l.get("kind").and_then(Value::as_str).unwrap_or("limit");
            let group = l.get("group").and_then(Value::as_str).unwrap_or("");
            let id = match (group, kind) {
                ("session", _) => "five_hour".to_owned(),
                ("weekly", "weekly_all") => "weekly".to_owned(),
                ("weekly", k) => k.to_owned(),
                (_, k) => k.to_owned(),
            };
            windows.push(UsageWindow {
                label: kind.replace('_', " "),
                id,
                used_percent: l.get("percent").and_then(Value::as_f64),
                resets_at: l.get("resets_at").and_then(ms),
                detail: None,
            });
        }
    }
    let named = [
        ("five_hour", "five_hour"),
        ("seven_day", "weekly"),
        ("seven_day_opus", "weekly_opus"),
        ("seven_day_sonnet", "weekly_sonnet"),
    ];
    for (key, id) in named {
        let Some(w) = body.get(key).filter(|v| v.is_object()) else { continue };
        if windows.iter().any(|x| x.id == id) {
            continue;
        }
        windows.push(UsageWindow {
            id: id.into(),
            label: key.replace('_', " "),
            used_percent: w.get("utilization").and_then(Value::as_f64),
            resets_at: w.get("resets_at").and_then(ms),
            detail: None,
        });
    }
    if let Some(extra) = body.get("extra_usage").filter(|v| v.is_object()) {
        let enabled = extra.get("is_enabled").and_then(Value::as_bool).unwrap_or(false);
        let decimals = extra.get("decimal_places").and_then(Value::as_u64).unwrap_or(2) as i32;
        let currency = extra.get("currency").and_then(Value::as_str).unwrap_or("USD");
        let money = |v: Option<f64>| v.map(|x| format!("{:.2} {currency}", x / 10f64.powi(decimals)));
        let used = money(extra.get("used_credits").and_then(Value::as_f64));
        let limit = money(extra.get("monthly_limit").and_then(Value::as_f64));
        windows.push(UsageWindow {
            id: if enabled { "extra".into() } else { "extra_off".into() },
            label: "extra usage".into(),
            used_percent: if enabled { extra.get("utilization").and_then(Value::as_f64) } else { None },
            resets_at: None,
            detail: match (enabled, used, limit) {
                (true, Some(u), Some(l)) => Some(format!("{u} / {l}")),
                (true, Some(u), None) => Some(u),
                _ => None,
            },
        });
    }
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape captured from the live endpoint (2026-10-08), trimmed.
    const SAMPLE: &str = r#"{
      "five_hour": {"utilization": 57.0, "resets_at": "2026-10-08T09:20:00.156839+00:00"},
      "seven_day": {"utilization": 50.0, "resets_at": "2026-10-12T22:00:00.156864+00:00"},
      "seven_day_opus": null,
      "extra_usage": {"is_enabled": false, "monthly_limit": null, "used_credits": null, "utilization": null},
      "limits": [
        {"kind": "session", "group": "session", "percent": 57, "resets_at": "2026-10-08T09:20:00.156839+00:00"},
        {"kind": "weekly_all", "group": "weekly", "percent": 50, "resets_at": "2026-10-12T22:00:00.156864+00:00"}
      ]
    }"#;

    #[test]
    fn parses_live_shape() {
        let w = parse(&serde_json::from_str(SAMPLE).unwrap());
        let ids: Vec<&str> = w.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, vec!["five_hour", "weekly", "extra_off"]);
        assert_eq!(w[0].used_percent, Some(57.0));
        assert_eq!(w[0].resets_at, Some(1_791_451_200_156));
        assert_eq!(w[2].used_percent, None);
    }

    #[test]
    fn falls_back_to_named_windows_and_formats_extra_credits() {
        let body = serde_json::json!({
            "five_hour": {"utilization": 12.5, "resets_at": "2026-10-08T09:20:00Z"},
            "seven_day_opus": {"utilization": 3.0, "resets_at": null},
            "extra_usage": {"is_enabled": true, "monthly_limit": 5000, "used_credits": 120, "utilization": 2.4, "currency": "USD", "decimal_places": 2}
        });
        let w = parse(&body);
        let ids: Vec<&str> = w.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, vec!["five_hour", "weekly_opus", "extra"]);
        assert_eq!(w[2].detail.as_deref(), Some("1.20 USD / 50.00 USD"));
        assert_eq!(w[2].used_percent, Some(2.4));
    }
}
