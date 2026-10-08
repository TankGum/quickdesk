//! Where encrypted objects live. Blocking I/O: the sync engine runs on its own thread.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use rusty_s3::actions::list_objects_v2::ListObjectsV2;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use serde::{Deserialize, Serialize};

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
}

/// In-memory store for tests and simulations.
#[derive(Default)]
pub struct MemoryTransport {
    objects: Mutex<BTreeMap<String, (Vec<u8>, i64)>>,
    /// When set, every call fails (simulates being offline).
    pub offline: std::sync::atomic::AtomicBool,
}

impl MemoryTransport {
    fn check(&self) -> Result<()> {
        if self.offline.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::Network("offline".into()));
        }
        Ok(())
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
        Ok(PutOutcome::Created)
    }
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        self.check()?;
        self.objects.lock().unwrap().insert(key.to_owned(), (bytes.to_vec(), now_ms()));
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
        Ok(())
    }
}

/// Non-secret part of the S3 configuration (the secret lives in the OS keyring).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    /// e.g. `https://<account>.r2.cloudflarestorage.com`, `https://s3.amazonaws.com`, `http://localhost:9000`
    pub endpoint: String,
    pub bucket: String,
    /// `auto` for Cloudflare R2.
    pub region: String,
    pub access_key_id: String,
    /// Object key prefix, e.g. `quickdesk/`.
    pub prefix: String,
}

/// Any S3-compatible store (Cloudflare R2, AWS S3, MinIO) via presigned URLs.
pub struct S3Transport {
    bucket: Bucket,
    credentials: Credentials,
    agent: ureq::Agent,
}

const SIGN_TTL: Duration = Duration::from_secs(300);

impl S3Transport {
    pub fn new(cfg: &S3Config, secret_access_key: &str) -> Result<Self> {
        let endpoint =
            url::Url::parse(cfg.endpoint.trim()).map_err(|e| Error::InvalidInput(format!("endpoint: {e}")))?;
        let bucket = Bucket::new(endpoint, UrlStyle::Path, cfg.bucket.trim().to_owned(), cfg.region.trim().to_owned())
            .map_err(|e| Error::InvalidInput(format!("bucket: {e}")))?;
        let agent =
            ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(10)).timeout(Duration::from_secs(60)).build();
        Ok(S3Transport {
            bucket,
            credentials: Credentials::new(cfg.access_key_id.trim(), secret_access_key.trim()),
            agent,
        })
    }

    fn call(req: std::result::Result<ureq::Response, ureq::Error>) -> Result<std::result::Result<ureq::Response, u16>> {
        match req {
            Ok(r) => Ok(Ok(r)),
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                match code {
                    401 | 403 => Err(Error::Auth(s3_message(&body).unwrap_or_else(|| format!("HTTP {code}")))),
                    404 | 409 | 412 => Ok(Err(code)),
                    _ => Err(Error::Remote(format!("HTTP {code}: {}", s3_message(&body).unwrap_or(body)))),
                }
            }
            Err(e) => Err(Error::Network(e.to_string())),
        }
    }
}

/// Extract `<Message>` from an S3 XML error body.
fn s3_message(body: &str) -> Option<String> {
    let start = body.find("<Message>")? + "<Message>".len();
    let end = body[start..].find("</Message>")? + start;
    Some(body[start..end].to_owned())
}

impl BlobTransport for S3Transport {
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<PutOutcome> {
        let mut action = self.bucket.put_object(Some(&self.credentials), key);
        action.headers_mut().insert("if-none-match", "*");
        let url = action.sign(SIGN_TTL);
        match Self::call(self.agent.put(url.as_str()).set("if-none-match", "*").send_bytes(bytes))? {
            Ok(_) => Ok(PutOutcome::Created),
            // 412: exists. 409: a concurrent conditional write won.
            Err(412 | 409) => Ok(PutOutcome::AlreadyExists),
            Err(code) => Err(Error::Remote(format!("conditional PUT {key}: HTTP {code}"))),
        }
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let url = self.bucket.put_object(Some(&self.credentials), key).sign(SIGN_TTL);
        match Self::call(self.agent.put(url.as_str()).send_bytes(bytes))? {
            Ok(_) => Ok(()),
            Err(code) => Err(Error::Remote(format!("PUT {key}: HTTP {code}"))),
        }
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let url = self.bucket.get_object(Some(&self.credentials), key).sign(SIGN_TTL);
        match Self::call(self.agent.get(url.as_str()).call())? {
            Ok(resp) => {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf)
                    .map_err(|e| Error::Network(e.to_string()))?;
                Ok(Some(buf))
            }
            Err(404) => Ok(None),
            Err(code) => Err(Error::Remote(format!("GET {key}: HTTP {code}"))),
        }
    }

    fn list(&self, prefix: &str, start_after: Option<&str>) -> Result<Vec<ObjectInfo>> {
        let mut out = Vec::new();
        let mut token: Option<String> = None;
        loop {
            let mut action = self.bucket.list_objects_v2(Some(&self.credentials));
            action.with_prefix(prefix);
            if let Some(sa) = start_after {
                action.with_start_after(sa);
            }
            if let Some(t) = &token {
                action.with_continuation_token(t.as_str());
            }
            let url = action.sign(SIGN_TTL);
            let body = match Self::call(self.agent.get(url.as_str()).call())? {
                Ok(resp) => resp.into_string().map_err(|e| Error::Network(e.to_string()))?,
                Err(404) => return Err(Error::Remote(format!("bucket {:?} not found", self.bucket.name()))),
                Err(code) => return Err(Error::Remote(format!("LIST {prefix}: HTTP {code}"))),
            };
            let parsed =
                ListObjectsV2::parse_response(&body).map_err(|e| Error::Remote(format!("bad LIST response: {e}")))?;
            out.extend(
                parsed.contents.into_iter().map(|c| ObjectInfo {
                    last_modified_ms: parse_rfc3339_ms(&c.last_modified).unwrap_or(0),
                    key: c.key,
                }),
            );
            match parsed.next_continuation_token {
                Some(t) => token = Some(t),
                None => break,
            }
        }
        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    fn delete(&self, key: &str) -> Result<()> {
        let url = self.bucket.delete_object(Some(&self.credentials), key).sign(SIGN_TTL);
        match Self::call(self.agent.delete(url.as_str()).call())? {
            Ok(_) | Err(404) => Ok(()),
            Err(code) => Err(Error::Remote(format!("DELETE {key}: HTTP {code}"))),
        }
    }
}

/// `2026-10-08T01:48:24.000Z` → unix ms (UTC only, as S3 returns).
fn parse_rfc3339_ms(s: &str) -> Option<i64> {
    let (date, time) = s.trim_end_matches('Z').split_once('T')?;
    let mut d = date.split('-').map(|p| p.parse::<i64>());
    let (y, m, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
    let (hms, frac) = time.split_once('.').unwrap_or((time, "0"));
    let mut t = hms.split(':').map(|p| p.parse::<i64>());
    let (hh, mm, ss) = (t.next()?.ok()?, t.next()?.ok()?, t.next()?.ok()?);
    let ms: i64 = format!("{:0<3}", &frac[..frac.len().min(3)]).parse().ok()?;
    // Days from civil (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(((days * 86_400 + hh * 3600 + mm * 60 + ss) * 1000) + ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339() {
        assert_eq!(parse_rfc3339_ms("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(parse_rfc3339_ms("2026-10-08T01:48:24.5Z"), Some(1_791_424_104_500));
        assert_eq!(parse_rfc3339_ms("2000-03-01T00:00:00Z"), Some(951_868_800_000));
        assert_eq!(parse_rfc3339_ms("garbage"), None);
    }

    #[test]
    fn s3_error_message() {
        let body = "<Error><Code>AccessDenied</Code><Message>Access Denied</Message></Error>";
        assert_eq!(s3_message(body).as_deref(), Some("Access Denied"));
    }

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
        t.delete("p/a").unwrap();
        assert_eq!(t.get("p/a").unwrap(), None);
    }
}
