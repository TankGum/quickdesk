//! Hybrid logical clock. Timestamps serialize to strings whose lexicographic
//! order equals causal order: `{physical_ms:013}-{counter:06}-{node}`.

use std::fmt;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc {
    pub physical_ms: u64,
    pub counter: u32,
    pub node: String,
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:013}-{:06}-{}", self.physical_ms, self.counter, self.node)
    }
}

impl FromStr for Hlc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let invalid = || Error::InvalidHlc(s.to_owned());
        let mut parts = s.splitn(3, '-');
        let physical_ms = parts.next().and_then(|p| p.parse().ok()).ok_or_else(invalid)?;
        let counter = parts.next().and_then(|p| p.parse().ok()).ok_or_else(invalid)?;
        let node = parts.next().filter(|n| !n.is_empty()).ok_or_else(invalid)?;
        Ok(Hlc { physical_ms, counter, node: node.to_owned() })
    }
}

/// Per-device clock. `now` for local events, `observe` when receiving remote ones.
pub struct Clock {
    node: String,
    last: Mutex<(u64, u32)>,
    wall: fn() -> u64,
}

fn system_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

impl Clock {
    pub fn new(node: impl Into<String>) -> Self {
        Self::with_wall(node, system_ms)
    }

    pub fn with_wall(node: impl Into<String>, wall: fn() -> u64) -> Self {
        Clock { node: node.into(), last: Mutex::new((0, 0)), wall }
    }

    pub fn now(&self) -> Hlc {
        let wall = (self.wall)();
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        *last = if wall > last.0 { (wall, 0) } else { (last.0, last.1 + 1) };
        Hlc { physical_ms: last.0, counter: last.1, node: self.node.clone() }
    }

    /// Advance past a remote timestamp so later local events sort after it.
    pub fn observe(&self, remote: &Hlc) {
        let wall = (self.wall)();
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        let pt = wall.max(last.0).max(remote.physical_ms);
        let counter = match (pt == last.0, pt == remote.physical_ms) {
            (true, true) => last.1.max(remote.counter) + 1,
            (true, false) => last.1 + 1,
            (false, true) => remote.counter + 1,
            (false, false) => 0,
        };
        *last = (pt, counter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frozen() -> u64 {
        1_000
    }

    #[test]
    fn roundtrips_through_string() {
        let h = Hlc { physical_ms: 1_759_800_000_000, counter: 7, node: "dev-a-1".into() };
        let s = h.to_string();
        assert_eq!(s, "1759800000000-000007-dev-a-1");
        assert_eq!(s.parse::<Hlc>().unwrap(), h);
    }

    #[test]
    fn rejects_malformed() {
        for bad in ["", "abc", "1-2", "1-x-node", "1-2-"] {
            assert!(bad.parse::<Hlc>().is_err(), "{bad:?} should fail");
        }
    }

    #[test]
    fn string_order_matches_struct_order() {
        let a = Hlc { physical_ms: 999, counter: 999_999, node: "z".into() };
        let b = Hlc { physical_ms: 1_000, counter: 0, node: "a".into() };
        assert!(a < b);
        assert!(a.to_string() < b.to_string());
    }

    #[test]
    fn now_is_strictly_monotonic_when_wall_clock_stalls() {
        let clock = Clock::with_wall("a", frozen);
        let t1 = clock.now();
        let t2 = clock.now();
        assert!(t2 > t1);
        assert_eq!(t2.counter, t1.counter + 1);
    }

    #[test]
    fn observe_moves_past_remote_clock_from_the_future() {
        let clock = Clock::with_wall("a", frozen);
        let remote = Hlc { physical_ms: 5_000, counter: 3, node: "b".into() };
        clock.observe(&remote);
        let next = clock.now();
        assert!(next > remote, "{next} should sort after {remote}");
        assert_eq!(next.physical_ms, 5_000);
    }
}
