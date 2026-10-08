//! End-to-end encryption. The bucket only ever sees ciphertext.
//!
//! * DEK: random 256-bit data key, generated once per sync space.
//! * Passphrase KEK: Argon2id(passphrase, salt). Recovery KEK: a random
//!   256-bit recovery key shown to the user once.
//! * `keyring.json` holds the DEK wrapped by each KEK.
//! * Every object is `QD1 | version | nonce(24) | XChaCha20-Poly1305(DEK, AAD = object key)`;
//!   binding the key path stops an attacker from swapping objects around.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use data_encoding::{BASE32_NOPAD, BASE64};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

const MAGIC: &[u8; 3] = b"QD1";
const FORMAT_VERSION: u8 = 1;
const NONCE_LEN: usize = 24;
pub const MIN_PASSPHRASE_CHARS: usize = 8;

/// 256-bit data encryption key.
#[derive(Clone, PartialEq, Eq)]
pub struct Dek([u8; 32]);

impl std::fmt::Debug for Dek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dek(..)")
    }
}

impl Dek {
    pub fn generate() -> Self {
        Dek(XChaCha20Poly1305::generate_key(&mut OsRng).into())
    }

    /// For caching in the OS keyring.
    pub fn to_base64(&self) -> String {
        BASE64.encode(&self.0)
    }

    pub fn from_base64(s: &str) -> Result<Self> {
        let bytes = BASE64.decode(s.as_bytes()).map_err(|_| Error::Crypto("bad cached key".into()))?;
        Ok(Dek(bytes.try_into().map_err(|_| Error::Crypto("bad cached key length".into()))?))
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(Key::from_slice(&self.0))
    }

    /// Encrypt an object stored at `object_key`.
    pub fn seal(&self, object_key: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = self
            .cipher()
            .encrypt(&nonce, Payload { msg: plaintext, aad: object_key.as_bytes() })
            .map_err(|_| Error::Crypto("encryption failed".into()))?;
        let mut out = Vec::with_capacity(4 + NONCE_LEN + ct.len());
        out.extend_from_slice(MAGIC);
        out.push(FORMAT_VERSION);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt an object; fails if it was modified or moved to another key.
    pub fn open(&self, object_key: &str, sealed: &[u8]) -> Result<Vec<u8>> {
        if sealed.len() < 4 + NONCE_LEN || &sealed[..3] != MAGIC {
            return Err(Error::Crypto(format!("{object_key}: not a QuickDesk object")));
        }
        if sealed[3] != FORMAT_VERSION {
            return Err(Error::Crypto(format!("{object_key}: unsupported format v{}", sealed[3])));
        }
        let nonce = XNonce::from_slice(&sealed[4..4 + NONCE_LEN]);
        self.cipher()
            .decrypt(nonce, Payload { msg: &sealed[4 + NONCE_LEN..], aad: object_key.as_bytes() })
            .map_err(|_| Error::Crypto(format!("{object_key}: decryption failed (wrong key or tampered)")))
    }
}

/// Argon2id cost parameters, stored alongside the salt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        KdfParams { memory_kib: 64 * 1024, iterations: 3, parallelism: 1 }
    }
}

/// Contents of `keyring.json`. Contains no secret in the clear.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyring {
    pub version: u32,
    pub kdf: KdfParams,
    pub salt: String,
    pub wrapped_by_passphrase: String,
    pub wrapped_by_recovery: String,
}

const AAD_PASSPHRASE: &[u8] = b"quickdesk/keyring/v1/passphrase";
const AAD_RECOVERY: &[u8] = b"quickdesk/keyring/v1/recovery";

fn derive_kek(passphrase: &str, salt: &[u8], p: KdfParams) -> Result<[u8; 32]> {
    let params = Params::new(p.memory_kib, p.iterations, p.parallelism, Some(32))
        .map_err(|e| Error::Crypto(format!("kdf params: {e}")))?;
    let mut out = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| Error::Crypto(format!("kdf: {e}")))?;
    Ok(out)
}

fn wrap(kek: &[u8; 32], aad: &[u8], dek: &Dek) -> Result<String> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cipher.encrypt(&nonce, Payload { msg: &dek.0, aad }).map_err(|_| Error::Crypto("wrap failed".into()))?;
    Ok(BASE64.encode(&[nonce.as_slice(), &ct].concat()))
}

fn unwrap(kek: &[u8; 32], aad: &[u8], wrapped: &str) -> Option<Dek> {
    let raw = BASE64.decode(wrapped.as_bytes()).ok()?;
    if raw.len() <= NONCE_LEN {
        return None;
    }
    let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
    let pt = cipher.decrypt(XNonce::from_slice(&raw[..NONCE_LEN]), Payload { msg: &raw[NONCE_LEN..], aad }).ok()?;
    Some(Dek(pt.try_into().ok()?))
}

/// Human-friendly recovery key: base32 in dash-separated groups of four.
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryKey([u8; 32]);

impl RecoveryKey {
    pub fn generate() -> Self {
        RecoveryKey(XChaCha20Poly1305::generate_key(&mut OsRng).into())
    }

    pub fn display(&self) -> String {
        let s = BASE32_NOPAD.encode(&self.0);
        s.as_bytes().chunks(4).map(|c| std::str::from_utf8(c).unwrap_or_default()).collect::<Vec<_>>().join("-")
    }

    /// Accepts any case, with or without dashes/spaces.
    pub fn parse(input: &str) -> Option<Self> {
        let cleaned: String =
            input.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
        let bytes = BASE32_NOPAD.decode(cleaned.as_bytes()).ok()?;
        Some(RecoveryKey(bytes.try_into().ok()?))
    }
}

impl Keyring {
    /// Create a keyring for `dek`, protected by `passphrase` and `recovery`.
    pub fn create(dek: &Dek, passphrase: &str, recovery: &RecoveryKey, kdf: KdfParams) -> Result<Self> {
        if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
            return Err(Error::InvalidInput(format!("passphrase must be at least {MIN_PASSPHRASE_CHARS} characters")));
        }
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&XChaCha20Poly1305::generate_nonce(&mut OsRng)[..16]);
        let kek = derive_kek(passphrase, &salt, kdf)?;
        Ok(Keyring {
            version: 1,
            kdf,
            salt: BASE64.encode(&salt),
            wrapped_by_passphrase: wrap(&kek, AAD_PASSPHRASE, dek)?,
            wrapped_by_recovery: wrap(&recovery.0, AAD_RECOVERY, dek)?,
        })
    }

    /// Unlock with either the passphrase or the recovery key.
    pub fn unlock(&self, secret: &str) -> Result<Dek> {
        if let Some(rk) = RecoveryKey::parse(secret) {
            if let Some(dek) = unwrap(&rk.0, AAD_RECOVERY, &self.wrapped_by_recovery) {
                return Ok(dek);
            }
        }
        let salt = BASE64.decode(self.salt.as_bytes()).map_err(|_| Error::Crypto("bad salt".into()))?;
        let kek = derive_kek(secret, &salt, self.kdf)?;
        unwrap(&kek, AAD_PASSPHRASE, &self.wrapped_by_passphrase).ok_or(Error::WrongSecret)
    }

    /// Re-wrap the same DEK under a new passphrase (recovery key unchanged).
    pub fn change_passphrase(&self, dek: &Dek, new_passphrase: &str) -> Result<Self> {
        if new_passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
            return Err(Error::InvalidInput(format!("passphrase must be at least {MIN_PASSPHRASE_CHARS} characters")));
        }
        let salt = BASE64.decode(self.salt.as_bytes()).map_err(|_| Error::Crypto("bad salt".into()))?;
        let kek = derive_kek(new_passphrase, &salt, self.kdf)?;
        Ok(Keyring { wrapped_by_passphrase: wrap(&kek, AAD_PASSPHRASE, dek)?, ..self.clone() })
    }
}

#[cfg(test)]
pub(crate) const TEST_KDF: KdfParams = KdfParams { memory_kib: 64, iterations: 1, parallelism: 1 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip_and_tamper_detection() {
        let dek = Dek::generate();
        let sealed = dek.seal("v1/log/a/1.bin", b"hello").unwrap();
        assert_eq!(dek.open("v1/log/a/1.bin", &sealed).unwrap(), b"hello");
        assert!(dek.open("v1/log/b/1.bin", &sealed).is_err(), "moved object must not decrypt");
        let mut flipped = sealed.clone();
        *flipped.last_mut().unwrap() ^= 1;
        assert!(dek.open("v1/log/a/1.bin", &flipped).is_err());
        assert!(Dek::generate().open("v1/log/a/1.bin", &sealed).is_err());
        assert!(dek.open("k", b"garbage").is_err());
    }

    #[test]
    fn keyring_unlocks_with_passphrase_or_recovery_key() {
        let dek = Dek::generate();
        let rk = RecoveryKey::generate();
        let kr = Keyring::create(&dek, "correct horse", &rk, TEST_KDF).unwrap();
        let json = serde_json::to_string(&kr).unwrap();
        assert!(!json.contains(&dek.to_base64()));
        let kr: Keyring = serde_json::from_str(&json).unwrap();

        assert_eq!(kr.unlock("correct horse").unwrap(), dek);
        assert_eq!(kr.unlock(&rk.display()).unwrap(), dek);
        assert_eq!(kr.unlock(&rk.display().to_lowercase().replace('-', " ")).unwrap(), dek);
        assert!(matches!(kr.unlock("wrong horse"), Err(Error::WrongSecret)));
    }

    #[test]
    fn passphrase_rules_and_change() {
        let dek = Dek::generate();
        let rk = RecoveryKey::generate();
        assert!(matches!(Keyring::create(&dek, "short", &rk, TEST_KDF), Err(Error::InvalidInput(_))));
        let kr = Keyring::create(&dek, "first passphrase", &rk, TEST_KDF).unwrap();
        let kr2 = kr.change_passphrase(&dek, "second passphrase").unwrap();
        assert_eq!(kr2.unlock("second passphrase").unwrap(), dek);
        assert!(kr2.unlock("first passphrase").is_err());
        assert_eq!(kr2.unlock(&rk.display()).unwrap(), dek, "recovery key still works");
    }

    #[test]
    fn recovery_key_format() {
        let rk = RecoveryKey::generate();
        let shown = rk.display();
        assert_eq!(shown.split('-').count(), 13);
        assert!(RecoveryKey::parse(&shown) == Some(rk));
        assert!(RecoveryKey::parse("not a key").is_none());
    }

    #[test]
    fn dek_cache_roundtrip() {
        let dek = Dek::generate();
        assert_eq!(Dek::from_base64(&dek.to_base64()).unwrap(), dek);
        assert!(Dek::from_base64("AAAA").is_err());
    }
}
