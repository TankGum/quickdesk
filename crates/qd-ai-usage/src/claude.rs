//! Claude (Claude Code / claude.ai subscription).
//!
//! Live numbers come from the same endpoint Claude Code's `/usage` uses,
//! authenticated with Claude Code's own OAuth login. We only *read* the
//! token: refreshing it would rotate the refresh token behind Claude Code's
//! back, so an expired login is reported instead ("open Claude Code").
//! The endpoint is not a public API and may change; parsing is lenient.
//!
//! On macOS Claude Code keeps the login in the Keychain instead of a file.
//! QuickDesk reads it there only after the user turned that on, through
//! `/usr/bin/security` like Claude Code (so macOS does not ask for the
//! password), and like the file it is never stored or sent anywhere but
//! api.anthropic.com. It is kept in memory between refreshes and read again
//! only once it stops working.

use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use crate::{now_ms, parse_rfc3339_ms, ProviderUsage, UsageWindow};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

/// The Keychain item Claude Code saves its login in (macOS).
#[cfg(target_os = "macos")]
const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
/// A login from the Keychain that stopped working (or could not be read) is
/// read again at most this often.
#[cfg(target_os = "macos")]
const KEYCHAIN_REREAD: Duration = Duration::from_secs(10 * 60);

/// The last Keychain read, so the login is not read on every refresh.
#[cfg(target_os = "macos")]
struct KeptLogin {
    result: Result<String, String>,
    read_at: std::time::Instant,
    /// The login was refused or has expired: read it again when allowed.
    stale: bool,
}

#[cfg(target_os = "macos")]
static KEPT_LOGIN: std::sync::Mutex<Option<KeptLogin>> = std::sync::Mutex::new(None);

#[cfg(target_os = "macos")]
fn kept_login() -> std::sync::MutexGuard<'static, Option<KeptLogin>> {
    KEPT_LOGIN.lock().unwrap_or_else(|e| e.into_inner())
}

/// Read the Keychain afresh next time (the user turned the option on again).
pub fn forget_kept_login() {
    #[cfg(target_os = "macos")]
    kept_login().take();
}

/// Where the login is read from.
enum Login {
    File(PathBuf),
    #[cfg(target_os = "macos")]
    Keychain,
}

pub struct Claude {
    login: Login,
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from).or_else(|| crate::home().map(|h| h.join(".claude")))
}

impl Claude {
    /// `keychain`: the user allowed reading the login from the Keychain.
    pub fn detect(keychain: bool) -> Option<Self> {
        let credentials = config_dir()?.join(".credentials.json");
        if credentials.exists() {
            return Some(Claude { login: Login::File(credentials) });
        }
        #[cfg(target_os = "macos")]
        if keychain && keychain_possible() {
            return Some(Claude { login: Login::Keychain });
        }
        let _ = keychain;
        None
    }

    fn read_login(&self) -> Result<String, String> {
        match &self.login {
            Login::File(path) => std::fs::read_to_string(path).map_err(|e| format!("cannot read Claude login: {e}")),
            #[cfg(target_os = "macos")]
            Login::Keychain => keychain_login(),
        }
    }

    pub fn fetch(&self) -> ProviderUsage {
        let mut usage = ProviderUsage::new("claude", "Claude", "live");
        match self.fetch_inner(&mut usage) {
            Ok(()) => usage.as_of = Some(now_ms()),
            Err(e) => usage.error = Some(e),
        }
        #[cfg(target_os = "macos")]
        if matches!(self.login, Login::Keychain) && usage.error.is_some() && !usage.is_transient_error() {
            if let Some(kept) = kept_login().as_mut() {
                kept.stale = true;
            }
        }
        usage
    }

    fn fetch_inner(&self, usage: &mut ProviderUsage) -> Result<(), String> {
        let raw = self.read_login()?;
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

/// macOS: Claude Code is set up here but its login is in the Keychain, so
/// reading it needs the user's go-ahead (the AI tab asks).
pub fn keychain_possible() -> bool {
    cfg!(target_os = "macos") && config_dir().is_some_and(|d| d.is_dir() && !d.join(".credentials.json").exists())
}

/// The login from the Keychain: the one read before while it works, else a
/// fresh read (at most every `KEYCHAIN_REREAD`).
#[cfg(target_os = "macos")]
fn keychain_login() -> Result<String, String> {
    // Held while reading, so two refreshes never read at once.
    let mut kept = kept_login();
    if let Some(k) = kept.as_ref() {
        let works = k.result.is_ok() && !k.stale;
        if works || k.read_at.elapsed() < KEYCHAIN_REREAD {
            return k.result.clone();
        }
    }
    let result = read_keychain();
    *kept = Some(KeptLogin { result: result.clone(), read_at: std::time::Instant::now(), stale: false });
    result
}

/// The login JSON Claude Code saved in the Keychain, read the way Claude Code
/// reads it: with `/usr/bin/security`. Claude Code creates the item with that
/// tool, so the item already trusts it and macOS does not ask for the
/// password. Read by QuickDesk itself, macOS would ask on every read QuickDesk
/// is not trusted for, and Claude Code replaces the item (dropping "Always
/// Allow") whenever it refreshes its login.
#[cfg(target_os = "macos")]
fn read_keychain() -> Result<String, String> {
    let out = std::process::Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Keychain: {e}"))?;
    // 44: errSecItemNotFound.
    if out.status.code() == Some(44) {
        return Err("not logged in to Claude Code".into());
    }
    if !out.status.success() {
        return Err(format!("Keychain: security exited with {}", out.status));
    }
    let text = String::from_utf8(out.stdout).map_err(|_| "Claude login is not text".to_owned())?;
    Ok(unhex(text.trim_end_matches('\n')))
}

/// `security -w` prints a value that is not plain text as hex.
#[cfg(target_os = "macos")]
fn unhex(s: &str) -> String {
    if s.starts_with('{') || s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return s.to_owned();
    }
    let bytes: Option<Vec<u8>> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect();
    bytes.and_then(|b| String::from_utf8(b).ok()).unwrap_or_else(|| s.to_owned())
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
