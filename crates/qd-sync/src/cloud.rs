//! QuickDesk Cloud: the hosted store behind `sync.quickdesk.click`
//! (sync-server/ in this repository). The server only keeps what the engine
//! uploads, which is already end-to-end encrypted.
//!
//! An account is a random id plus a random token, created anonymously. The app
//! shows both as one **sync code**; another device enters the code (to reach
//! the account) and the passphrase (to decrypt it).

use std::time::Duration;

use data_encoding::BASE32_NOPAD;
use serde::Deserialize;

use crate::transport::{BlobTransport, ObjectInfo, PutOutcome};
use crate::{Error, Result};

/// Production endpoint; `QD_SYNC_URL` overrides it (local `wrangler dev`, tests).
pub const DEFAULT_ENDPOINT: &str = "https://sync.quickdesk.click";

pub fn endpoint() -> String {
    std::env::var("QD_SYNC_URL").ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| DEFAULT_ENDPOINT.into())
}

/// Credentials of one sync account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Account {
    pub account: String,
    pub token: String,
}

const CODE_PREFIX: &str = "QD1";
const ACCOUNT_LEN: usize = 16;

impl Account {
    /// `QD1-XXXXXX-…`: the account id and the token in one string, grouped for reading.
    pub fn sync_code(&self) -> String {
        let token = hex_decode(&self.token).map(|b| BASE32_NOPAD.encode(&b)).unwrap_or_default();
        let body = format!("{}{}", self.account.to_ascii_uppercase(), token);
        let groups: Vec<String> = body.as_bytes().chunks(6).map(|c| String::from_utf8_lossy(c).into_owned()).collect();
        format!("{CODE_PREFIX}-{}", groups.join("-"))
    }

    /// Accepts the code as shown, with or without dashes, spaces or the prefix, any case.
    pub fn from_sync_code(code: &str) -> Result<Self> {
        let bad = || Error::InvalidInput("that is not a QuickDesk sync code".into());
        let mut s: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
        if let Some(rest) = s.strip_prefix(CODE_PREFIX) {
            s = rest.to_owned();
        }
        if s.len() <= ACCOUNT_LEN {
            return Err(bad());
        }
        let (account, token) = s.split_at(ACCOUNT_LEN);
        let token = BASE32_NOPAD.decode(token.as_bytes()).map_err(|_| bad())?;
        if token.len() != 16 || !account.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(bad());
        }
        Ok(Account { account: account.to_ascii_lowercase(), token: token.iter().map(|b| format!("{b:02x}")).collect() })
    }
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(10)).timeout(Duration::from_secs(60)).build()
}

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

/// Map an HTTP failure to our error kinds; `Ok(code)` for statuses the caller handles.
fn call(req: std::result::Result<ureq::Response, ureq::Error>) -> Result<std::result::Result<ureq::Response, u16>> {
    match req {
        Ok(r) => Ok(Ok(r)),
        Err(ureq::Error::Status(code, r)) => {
            let message = r
                .into_string()
                .ok()
                .and_then(|b| serde_json::from_str::<ErrorBody>(&b).ok())
                .map(|b| b.error)
                .unwrap_or_else(|| format!("HTTP {code}"));
            match code {
                401 | 403 => Err(Error::Auth(message)),
                404 | 412 => Ok(Err(code)),
                413 | 507 => Err(Error::Remote(message)),
                429 | 500..=599 => Err(Error::Network(message)),
                _ => Err(Error::Remote(message)),
            }
        }
        Err(e) => Err(Error::Network(e.to_string())),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(resp: ureq::Response) -> Result<T> {
    let body = resp.into_string().map_err(|e| Error::Network(e.to_string()))?;
    serde_json::from_str(&body).map_err(|e| Error::Remote(format!("bad response from the sync server: {e}")))
}

/// Create a new, empty sync account.
pub fn create_account(endpoint: &str) -> Result<Account> {
    let url = format!("{}/v1/accounts", endpoint.trim_end_matches('/'));
    match call(agent().post(&url).send_bytes(&[]))? {
        Ok(r) => read_json(r),
        Err(code) => Err(Error::Remote(format!("could not create a sync account: HTTP {code}"))),
    }
}

/// Delete the account and everything stored in it.
pub fn delete_account(endpoint: &str, account: &Account) -> Result<()> {
    let url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), account.account);
    match call(agent().delete(&url).set("authorization", &format!("Bearer {}", account.token)).call())? {
        Ok(_) | Err(404) => Ok(()),
        Err(code) => Err(Error::Remote(format!("HTTP {code}"))),
    }
}

pub struct CloudTransport {
    /// `…/v1/<account>`
    account_url: String,
    /// `…/v1/<account>/objects`
    base: String,
    auth: String,
    agent: ureq::Agent,
}

impl CloudTransport {
    pub fn new(endpoint: &str, account: &Account) -> Self {
        let account_url = format!("{}/v1/{}", endpoint.trim_end_matches('/'), account.account);
        CloudTransport {
            base: format!("{account_url}/objects"),
            account_url,
            auth: format!("Bearer {}", account.token),
            agent: agent(),
        }
    }

    fn object_url(&self, key: &str) -> String {
        let path: Vec<String> =
            key.split('/').map(|seg| url::form_urlencoded::byte_serialize(seg.as_bytes()).collect()).collect();
        format!("{}/{}", self.base, path.join("/"))
    }
}

#[derive(Deserialize)]
struct AccountInfo {
    rev: String,
}

#[derive(Deserialize)]
struct Listing {
    objects: Vec<Listed>,
}

#[derive(Deserialize)]
struct Listed {
    key: String,
    uploaded: i64,
}

impl BlobTransport for CloudTransport {
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<PutOutcome> {
        let req = self.agent.put(&self.object_url(key)).set("authorization", &self.auth).set("if-none-match", "*");
        match call(req.send_bytes(bytes))? {
            Ok(_) => Ok(PutOutcome::Created),
            Err(412) => Ok(PutOutcome::AlreadyExists),
            Err(code) => Err(Error::Remote(format!("conditional PUT {key}: HTTP {code}"))),
        }
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        match call(self.agent.put(&self.object_url(key)).set("authorization", &self.auth).send_bytes(bytes))? {
            Ok(_) => Ok(()),
            Err(code) => Err(Error::Remote(format!("PUT {key}: HTTP {code}"))),
        }
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match call(self.agent.get(&self.object_url(key)).set("authorization", &self.auth).call())? {
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
        let mut req = self.agent.get(&self.base).set("authorization", &self.auth).query("prefix", prefix);
        if let Some(sa) = start_after {
            req = req.query("start_after", sa);
        }
        match call(req.call())? {
            Ok(resp) => {
                let listing: Listing = read_json(resp)?;
                Ok(listing
                    .objects
                    .into_iter()
                    .map(|o| ObjectInfo { key: o.key, last_modified_ms: o.uploaded })
                    .collect())
            }
            Err(404) => Err(Error::Auth("this sync account no longer exists".into())),
            Err(code) => Err(Error::Remote(format!("LIST {prefix}: HTTP {code}"))),
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        match call(self.agent.delete(&self.object_url(key)).set("authorization", &self.auth).call())? {
            Ok(_) | Err(404) => Ok(()),
            Err(code) => Err(Error::Remote(format!("DELETE {key}: HTTP {code}"))),
        }
    }

    fn revision(&self) -> Result<Option<String>> {
        match call(self.agent.get(&self.account_url).set("authorization", &self.auth).call())? {
            Ok(resp) => Ok(Some(read_json::<AccountInfo>(resp)?.rev)),
            Err(404) => Err(Error::Auth("this sync account no longer exists".into())),
            Err(code) => Err(Error::Remote(format!("GET account: HTTP {code}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_code_roundtrips_and_tolerates_formatting() {
        let a = Account { account: "ua6aulg2mx7nyleo".into(), token: "5fa5e2223e229401a4d895d2a3172356".into() };
        let code = a.sync_code();
        assert!(code.starts_with("QD1-"), "{code}");
        assert_eq!(Account::from_sync_code(&code).unwrap(), a);
        assert_eq!(Account::from_sync_code(&code.to_lowercase().replace('-', " ")).unwrap(), a);
        assert_eq!(Account::from_sync_code(&code.trim_start_matches("QD1-").replace('-', "")).unwrap(), a);
        assert!(Account::from_sync_code("QD1-NOPE").is_err());
        assert!(Account::from_sync_code("").is_err());
    }

    /// Against a running server: `cd sync-server && npx wrangler dev`, then
    /// `QD_SYNC_URL=http://127.0.0.1:8787 cargo test -p qd-sync cloud -- --ignored`.
    #[test]
    #[ignore = "needs a running sync server (QD_SYNC_URL)"]
    fn against_a_live_server() {
        let ep = endpoint();
        let acct = create_account(&ep).unwrap();
        let t = CloudTransport::new(&ep, &acct);
        let empty = t.revision().unwrap();
        assert_eq!(t.put_if_absent("v1/keyring.json", b"1").unwrap(), PutOutcome::Created);
        let rev = t.revision().unwrap();
        assert_ne!(rev, empty);
        assert_eq!(t.revision().unwrap(), rev, "reads leave the revision alone");
        assert_eq!(t.put_if_absent("v1/keyring.json", b"2").unwrap(), PutOutcome::AlreadyExists);
        assert_eq!(t.get("v1/keyring.json").unwrap().as_deref(), Some(&b"1"[..]));
        t.put("v1/log/d/1.bin", b"a").unwrap();
        t.put("v1/log/d/2.bin", b"b").unwrap();
        let keys: Vec<String> = t.list("v1/log/", Some("v1/log/d/1.bin")).unwrap().into_iter().map(|o| o.key).collect();
        assert_eq!(keys, ["v1/log/d/2.bin"]);
        let before = t.revision().unwrap();
        t.delete("v1/log/d/2.bin").unwrap();
        assert_eq!(t.get("v1/log/d/2.bin").unwrap(), None);
        assert_ne!(t.revision().unwrap(), before);
        let wrong = CloudTransport::new(&ep, &Account { token: "00".repeat(16), ..acct.clone() });
        assert!(matches!(wrong.list("v1/", None), Err(Error::Auth(_))));
        assert!(matches!(wrong.revision(), Err(Error::Auth(_))));
        delete_account(&ep, &acct).unwrap();
        assert!(matches!(t.list("v1/", None), Err(Error::Auth(_))));
        assert!(matches!(t.revision(), Err(Error::Auth(_))));
    }
}
