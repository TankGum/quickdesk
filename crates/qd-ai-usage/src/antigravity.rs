//! Google Antigravity CLI (Gemini). It keeps no quota percentage on disk,
//! but its logs record when a request was refused because the quota ran out
//! (`RESOURCE_EXHAUSTED … quota reached`). We report the latest such event.

use std::fs;
use std::path::PathBuf;

use crate::{days_from_civil, ProviderUsage, UsageWindow};

pub struct Antigravity {
    logs: PathBuf,
}

impl Antigravity {
    pub fn detect() -> Option<Self> {
        let dir = crate::home()?.join(".gemini").join("antigravity-cli");
        dir.is_dir().then(|| Antigravity { logs: dir.join("log") })
    }

    pub fn fetch(&self) -> ProviderUsage {
        let mut usage = ProviderUsage::new("antigravity", "Gemini (Antigravity)", "local");
        let mut files: Vec<PathBuf> = fs::read_dir(&self.logs)
            .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "log")).collect())
            .unwrap_or_default();
        files.sort();
        let last_hit = files.iter().rev().find_map(|p| {
            let year = year_from_name(p.file_name()?.to_str()?)?;
            let text = fs::read_to_string(p).ok()?;
            text.lines().rev().find(|l| l.contains("RESOURCE_EXHAUSTED")).and_then(|l| glog_time_ms(l, year))
        });
        match last_hit {
            Some(at) => {
                usage.as_of = Some(at);
                usage.windows.push(UsageWindow {
                    id: "quota_hit".into(),
                    label: "quota reached".into(),
                    used_percent: None,
                    resets_at: None,
                    detail: None,
                });
            }
            None => usage.error = Some("no_percent".into()),
        }
        usage
    }
}

/// `cli-20260713_152303.log` → 2026
fn year_from_name(name: &str) -> Option<i64> {
    name.strip_prefix("cli-")?.get(..4)?.parse().ok()
}

/// glog prefix `E0713 15:48:55.956336 …` (local time) → unix ms.
pub(crate) fn glog_time_ms(line: &str, year: i64) -> Option<i64> {
    let head = line.get(1..)?;
    let (md, rest) = head.split_once(' ')?;
    let (month, day) = (md.get(..2)?.parse::<i64>().ok()?, md.get(2..4)?.parse::<i64>().ok()?);
    let time = rest.split_whitespace().next()?;
    let mut t = time.split(['.', ':']).map(|p| p.parse::<i64>());
    let (h, m, s) = (t.next()?.ok()?, t.next()?.ok()?, t.next()?.ok()?);
    let local = (days_from_civil(year, month, day) * 86_400 + h * 3600 + m * 60 + s) * 1000;
    Some(local - local_offset_ms())
}

/// The machine's current UTC offset (logs are written in local time).
#[cfg(unix)]
fn local_offset_ms() -> i64 {
    // SAFETY: `time` and `localtime_r` only write into values we own.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&now, &mut tm).is_null() {
            return 0;
        }
        tm.tm_gmtoff as i64 * 1000
    }
}

#[cfg(not(unix))]
fn local_offset_ms() -> i64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_glog_lines_and_file_years() {
        assert_eq!(year_from_name("cli-20260713_152303.log"), Some(2026));
        let line = "E0713 15:48:55.956336 779444 log.go:398] model unreachable: RESOURCE_EXHAUSTED (code 429)";
        let ms = glog_time_ms(line, 2026).unwrap();
        let utc_naive = (days_from_civil(2026, 7, 13) * 86_400 + 15 * 3600 + 48 * 60 + 55) * 1000;
        assert_eq!(ms + local_offset_ms(), utc_naive);
        assert!(glog_time_ms("garbage", 2026).is_none());
    }
}
