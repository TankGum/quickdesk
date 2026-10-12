//! Secrets in the OS keyring (GNOME Keyring / KWallet via Secret Service,
//! Windows Credential Manager, macOS Keychain). Never stored in SQLite.
//!
//! Each secret is read from the keyring at most once per run and then kept in
//! memory: sync needs them every round, and macOS asks for the login password
//! on each read it does not trust the app for (an ad hoc signed app loses
//! "Always Allow" with every update).

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

/// Keyring service name. Kept from the original identifier on purpose:
/// changing it would orphan secrets users already stored.
const SERVICE: &str = "dev.quickdesk.app";

/// QuickDesk Cloud sync account token (with the account id, it is the sync code).
pub const SYNC_TOKEN: &str = "sync-cloud-token";
/// Secret of the own-bucket setup of 0.3.0 and earlier; only ever deleted now.
pub const S3_SECRET: &str = "sync-s3-secret";
pub const SYNC_DEK: &str = "sync-dek";

/// What this run read or wrote; `None` = not in the keyring.
static CACHE: Mutex<BTreeMap<String, Option<String>>> = Mutex::new(BTreeMap::new());

fn cache() -> MutexGuard<'static, BTreeMap<String, Option<String>>> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

fn entry(name: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, name).map_err(|e| format!("OS keyring unavailable: {e}"))
}

pub fn set(name: &str, value: &str) -> Result<(), String> {
    let mut cache = cache();
    entry(name)?.set_password(value).map_err(|e| format!("could not save to OS keyring: {e}"))?;
    cache.insert(name.to_owned(), Some(value.to_owned()));
    Ok(())
}

pub fn get(name: &str) -> Result<Option<String>, String> {
    // Held while reading, so two threads never both make macOS ask.
    let mut cache = cache();
    if let Some(known) = cache.get(name) {
        return Ok(known.clone());
    }
    let value = match entry(name)?.get_password() {
        Ok(v) => Some(v),
        Err(keyring::Error::NoEntry) => None,
        Err(e) => return Err(format!("could not read OS keyring: {e}")),
    };
    cache.insert(name.to_owned(), value.clone());
    Ok(value)
}

pub fn delete(name: &str) -> Result<(), String> {
    let mut cache = cache();
    match entry(name)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {
            cache.insert(name.to_owned(), None);
            Ok(())
        }
        Err(e) => Err(format!("could not delete from OS keyring: {e}")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "touches the real OS keyring"]
    fn roundtrip_in_os_keyring() {
        let name = "test-roundtrip";
        super::set(name, "s3cr3t").unwrap();
        assert_eq!(super::get(name).unwrap().as_deref(), Some("s3cr3t"));
        super::delete(name).unwrap();
        assert_eq!(super::get(name).unwrap(), None);
    }
}
