//! Secrets in the OS keyring (GNOME Keyring / KWallet via Secret Service,
//! Windows Credential Manager, macOS Keychain). Never stored in SQLite.

/// Keyring service name. Kept from the original identifier on purpose:
/// changing it would orphan secrets users already stored.
const SERVICE: &str = "dev.quickdesk.app";

pub const S3_SECRET: &str = "sync-s3-secret";
pub const SYNC_DEK: &str = "sync-dek";

fn entry(name: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, name).map_err(|e| format!("OS keyring unavailable: {e}"))
}

pub fn set(name: &str, value: &str) -> Result<(), String> {
    entry(name)?.set_password(value).map_err(|e| format!("could not save to OS keyring: {e}"))
}

pub fn get(name: &str) -> Result<Option<String>, String> {
    match entry(name)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("could not read OS keyring: {e}")),
    }
}

pub fn delete(name: &str) -> Result<(), String> {
    match entry(name)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
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
