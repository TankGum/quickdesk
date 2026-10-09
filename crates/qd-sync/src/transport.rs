//! Where encrypted objects live. Blocking I/O: the sync engine runs on its own thread.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub key: String,
    /// Unix ms; 0 when unknown.
    pub last_modified_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOutcome {
    Created,
    AlreadyExists,
}

pub trait BlobTransport: Send + Sync {
    /// Create `key` only if it does not exist yet (conditional PUT).
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<PutOutcome>;
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    /// All keys under `prefix` sorted ascending, strictly after `start_after`.
    fn list(&self, prefix: &str, start_after: Option<&str>) -> Result<Vec<ObjectInfo>>;
    fn delete(&self, key: &str) -> Result<()>;
    /// A value that changes whenever anything in the store is written or
    /// deleted, read in one cheap call. Lets a round end early when nothing
    /// changed. `None`: the store cannot tell, so every round looks.
    fn revision(&self) -> Result<Option<String>> {
        Ok(None)
    }
}

/// In-memory store for tests and simulations.
#[derive(Default)]
pub struct MemoryTransport {
    objects: Mutex<BTreeMap<String, (Vec<u8>, i64)>>,
    /// When set, every call fails (simulates being offline).
    pub offline: std::sync::atomic::AtomicBool,
    writes: std::sync::atomic::AtomicU64,
}

impl MemoryTransport {
    fn check(&self) -> Result<()> {
        if self.offline.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::Network("offline".into()));
        }
        Ok(())
    }

    fn wrote(&self) {
        self.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn keys(&self) -> Vec<String> {
        self.objects.lock().unwrap().keys().cloned().collect()
    }

    /// Test hook: pretend an object was written at `ms`.
    pub fn set_modified(&self, key: &str, ms: i64) {
        if let Some(o) = self.objects.lock().unwrap().get_mut(key) {
            o.1 = ms;
        }
    }

    /// Test hook: raw bytes as stored.
    pub fn raw(&self, key: &str) -> Option<Vec<u8>> {
        self.objects.lock().unwrap().get(key).map(|o| o.0.clone())
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

impl BlobTransport for MemoryTransport {
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<PutOutcome> {
        self.check()?;
        let mut o = self.objects.lock().unwrap();
        if o.contains_key(key) {
            return Ok(PutOutcome::AlreadyExists);
        }
        o.insert(key.to_owned(), (bytes.to_vec(), now_ms()));
        self.wrote();
        Ok(PutOutcome::Created)
    }
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        self.check()?;
        self.objects.lock().unwrap().insert(key.to_owned(), (bytes.to_vec(), now_ms()));
        self.wrote();
        Ok(())
    }
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.check()?;
        Ok(self.objects.lock().unwrap().get(key).map(|o| o.0.clone()))
    }
    fn list(&self, prefix: &str, start_after: Option<&str>) -> Result<Vec<ObjectInfo>> {
        self.check()?;
        Ok(self
            .objects
            .lock()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.starts_with(prefix) && start_after.is_none_or(|s| k.as_str() > s))
            .map(|(k, (_, m))| ObjectInfo { key: k.clone(), last_modified_ms: *m })
            .collect())
    }
    fn delete(&self, key: &str) -> Result<()> {
        self.check()?;
        self.objects.lock().unwrap().remove(key);
        self.wrote();
        Ok(())
    }
    fn revision(&self) -> Result<Option<String>> {
        self.check()?;
        Ok(Some(self.writes.load(std::sync::atomic::Ordering::Relaxed).to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_transport_semantics() {
        let t = MemoryTransport::default();
        assert_eq!(t.put_if_absent("p/a", b"1").unwrap(), PutOutcome::Created);
        assert_eq!(t.put_if_absent("p/a", b"2").unwrap(), PutOutcome::AlreadyExists);
        t.put("p/b", b"3").unwrap();
        t.put("q/c", b"4").unwrap();
        let keys = |v: Vec<ObjectInfo>| v.into_iter().map(|o| o.key).collect::<Vec<_>>();
        assert_eq!(keys(t.list("p/", None).unwrap()), vec!["p/a", "p/b"]);
        assert_eq!(keys(t.list("p/", Some("p/a")).unwrap()), vec!["p/b"]);
        assert_eq!(t.get("p/a").unwrap().as_deref(), Some(&b"1"[..]));
        let rev = t.revision().unwrap();
        t.delete("p/a").unwrap();
        assert_eq!(t.get("p/a").unwrap(), None);
        assert_ne!(t.revision().unwrap(), rev);
    }
}
